use crate::ui::ui;
use crate::{app_helper_structs, keybinds};
use chrono::{Datelike, Month};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self};
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::widgets::{ListState, TableState};
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use crate::anilist::{get_media, get_user_media_list};
use crate::app_helper_structs::{
    ActiveBlock, BrowseCategory, BrowseState, CurrentView, MediaListItem, MediaType,
    NextAiringEpisode, PageInfo, Season, User, UserMediaList,
};

pub struct App {
    pub active_block: ActiveBlock,
    pub current_view: CurrentView,
    pub user: Option<User>,
    pub sidebar_items: Vec<CurrentView>,
    pub sidebar_state: ListState,

    pub is_loading: bool,
    pub error_message: Option<String>,

    pub browse_state: BrowseState,
}

impl App {
    pub fn new() -> App {
        let mut state = ListState::default();
        state.select(Some(0));

        App {
            active_block: ActiveBlock::Sidebar,
            current_view: CurrentView::UserAnime,
            sidebar_items: CurrentView::ALL.to_vec(),
            sidebar_state: state,
            is_loading: false,
            user: None,

            error_message: None,

            browse_state: BrowseState {
                loaded_view: CurrentView::UserAnime,
                media: None,
                state: TableState::default(),
                current_category: BrowseCategory::CategoryOne,
            },
        }
    }

    pub fn next_sidebar_item(&mut self) {
        let len = self.sidebar_items.len();
        if len == 0 {
            return;
        }
        let current = self.sidebar_state.selected().unwrap_or(0);
        self.sidebar_state.select(Some((current + 1) % len));
    }

    pub fn previous_sidebar_item(&mut self) {
        let len = self.sidebar_items.len();
        if len == 0 {
            return;
        }
        let current = self.sidebar_state.selected().unwrap_or(0);
        self.sidebar_state.select(Some((current + len - 1) % len));
    }

    pub fn authenticated(&mut self, id: i64, name: String, allows_nsfw: Option<bool>) {
        self.user = Some(User {
            id,
            name,
            allows_nsfw,
        })
    }
    pub fn get_current_center_items(&self) -> &[MediaListItem] {
        self.browse_state
            .media
            .as_ref()
            .and_then(|l| l.items.as_deref())
            .unwrap_or(&[])
    }

    pub fn next_center_item(&mut self) {
        let count = self.get_current_center_items().len();
        if count == 0 {
            return;
        }

        let current = self.browse_state.state.selected().unwrap_or(0);
        self.browse_state.state.select(Some((current + 1) % count));
    }

    pub fn previous_center_item(&mut self) {
        let count = self.get_current_center_items().len();
        if count == 0 {
            return;
        }

        let current = self.browse_state.state.selected().unwrap_or(0);
        self.browse_state
            .state
            .select(Some((current + count - 1) % count));
    }

    pub fn fetch_user_media(
        &mut self,
        client: crate::anilist::AnilistClient,
        tx: Sender<AppAction>,
    ) {
        self.fetch_user_media_list(
            client,
            tx,
            match self.browse_state.current_category {
                BrowseCategory::CategoryOne => Some(get_user_media_list::MediaListStatus::CURRENT),
                BrowseCategory::CategoryTwo => {
                    Some(get_user_media_list::MediaListStatus::COMPLETED)
                }
                BrowseCategory::CategoryThree => {
                    Some(get_user_media_list::MediaListStatus::PLANNING)
                }
                _ => None,
            },
            match self.browse_state.current_category {
                BrowseCategory::CategoryTwo => {
                    Some(vec![get_user_media_list::MediaListSort::SCORE_DESC])
                }
                _ => None,
            },
            Some(
                self.browse_state
                    .media
                    .as_ref()
                    .map_or(1, |m| m.page_info.current_page),
            ),
            Some(
                self.browse_state
                    .media
                    .as_ref()
                    .map_or(25, |m| m.page_info.per_page),
            ),
            match self.current_view {
                CurrentView::UserAnime => get_user_media_list::MediaType::ANIME,
                CurrentView::UserManga => get_user_media_list::MediaType::MANGA,
                _ => unimplemented!(),
            },
        );
    }

    pub fn fetch_user_media_list(
        &mut self,
        client: crate::anilist::AnilistClient,
        tx: Sender<AppAction>,
        status: Option<get_user_media_list::MediaListStatus>,
        sort: Option<Vec<get_user_media_list::MediaListSort>>,
        page: Option<i64>,
        per_page: Option<i64>,
        graphql_type: get_user_media_list::MediaType,
    ) {
        let user_id = self.user.as_ref().map(|u| u.id).unwrap_or(0);
        self.is_loading = true;
        self.error_message = None;

        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = client_clone
                .get_user_media_list(user_id, status, sort, page, per_page, graphql_type)
                .await;

            let action: AppAction = Box::new(move |app: &mut App| {
                app.is_loading = false;
                match result {
                    Ok(data) => {
                        let clean_list = App::to_user_media_list(data, user_id);

                        app.browse_state.media = Some(clean_list);
                        app.browse_state.state.select_first();
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            });
            let _ = tx_clone.send(action);
        });
    }

    pub fn next_center_page(&mut self) {
        if let Some(media) = &mut self.browse_state.media {
            if media.page_info.has_next_page.unwrap_or(false) {
                media.page_info.current_page = media.page_info.current_page + 1;
            }
        }
    }

    pub fn previous_center_page(&mut self) {
        if let Some(media) = &mut self.browse_state.media {
            if media.page_info.has_next_page.unwrap_or(false) {
                media.page_info.current_page = media.page_info.current_page - 1;
            }
        }
    }

    pub fn fetch_browse(&mut self, client: crate::anilist::AnilistClient, tx: Sender<AppAction>) {
        self.fetch_media(
            client,
            tx,
            None,
            match self.current_view {
                CurrentView::BrowseAnime => match self.browse_state.current_category {
                    BrowseCategory::CategoryOne => Some(vec![get_media::MediaSort::TRENDING_DESC]),
                    BrowseCategory::CategoryTwo | BrowseCategory::CategoryThree => {
                        Some(vec![get_media::MediaSort::POPULARITY_DESC])
                    }
                    _ => unimplemented!(),
                },
                CurrentView::BrowseManga => match self.browse_state.current_category {
                    BrowseCategory::CategoryOne => Some(vec![get_media::MediaSort::TRENDING_DESC]),
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
            Some(
                self.browse_state
                    .media
                    .as_ref()
                    .map_or(1, |m| m.page_info.current_page),
            ),
            Some(
                self.browse_state
                    .media
                    .as_ref()
                    .map_or(25, |m| m.page_info.per_page),
            ),
            match self.current_view {
                CurrentView::BrowseAnime => get_media::MediaType::ANIME,
                CurrentView::BrowseManga => get_media::MediaType::MANGA,
                _ => get_media::MediaType::ANIME,
            },
            {
                match self.current_view {
                    CurrentView::BrowseAnime => MediaType::Anime,
                    CurrentView::BrowseManga => MediaType::Manga,
                    _ => MediaType::Anime,
                }
            },
            match self.browse_state.current_category {
                BrowseCategory::CategoryTwo => Some(App::get_season().to_get_media_media_season()),
                BrowseCategory::CategoryThree => {
                    Some(App::get_season().next().to_get_media_media_season())
                }
                _ => None,
            },
            match self.browse_state.current_category {
                BrowseCategory::CategoryTwo | BrowseCategory::CategoryThree => {
                    Some(App::get_year())
                }
                _ => None,
            },
            None,
            None,
        );
    }

    pub fn get_year() -> i64 {
        chrono::Utc::now().year() as i64
    }
    pub fn get_season() -> Season {
        let current_date = chrono::Utc::now();
        let month = current_date.month();
        match month {
            1 | 2 | 3 => Season::WINTER,
            4 | 5 | 6 => Season::SPRING,
            7 | 8 | 9 => Season::SUMMER,
            10 | 11 | 12 => Season::FALL,
            _ => unimplemented!(),
        }
    }

    pub fn fetch_media(
        &mut self,
        client: crate::anilist::AnilistClient,
        tx: Sender<AppAction>,
        status: Option<Vec<get_media::MediaStatus>>,
        sort: Option<Vec<get_media::MediaSort>>,
        page: Option<i64>,
        per_page: Option<i64>,
        graphql_type: get_media::MediaType,
        app_type: MediaType,
        season: Option<get_media::MediaSeason>,
        season_year: Option<i64>,
        search: Option<String>,
        format: Option<get_media::MediaFormat>,
    ) {
        self.is_loading = true;
        self.error_message = None;

        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = client_clone
                .get_media(
                    graphql_type,
                    season,
                    season_year,
                    status,
                    sort,
                    page,
                    per_page,
                    search,
                    format,
                )
                .await;

            let action: AppAction = Box::new(move |app: &mut App| {
                app.is_loading = false;
                match result {
                    Ok(data) => {
                        let clean_list = App::browse_media_to_user_list(data, app_type);

                        app.browse_state.media = Some(clean_list);
                        app.browse_state.state.select_first();
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            });
            let _ = tx_clone.send(action);
        });
    }

    pub fn to_user_media_list(
        data: get_user_media_list::ResponseData,
        user_id: i64,
    ) -> UserMediaList {
        let mut page_info = PageInfo {
            current_page: 1,
            per_page: 50,
            total: None,
            last_page: None,
            has_next_page: None,
        };
        let mut items = Vec::new();

        if let Some(page) = data.page {
            if let Some(pi) = page.page_info {
                page_info.current_page = pi.current_page.unwrap_or(1);
                page_info.per_page = pi.per_page.unwrap_or(50);
                page_info.total = pi.total;
                page_info.last_page = pi.last_page;
                page_info.has_next_page = pi.has_next_page;
            }

            if let Some(media_list) = page.media_list {
                for m in media_list.into_iter().flatten() {
                    let id = m.media.as_ref().map(|x| x.id).unwrap_or(0);
                    let title = m
                        .media
                        .as_ref()
                        .and_then(|x| x.title.as_ref())
                        .and_then(|t| t.user_preferred.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let total = m.media.as_ref().and_then(|x| {
                        Some(x.episodes.unwrap_or_else(|| 0) + x.chapters.unwrap_or_else(|| 0))
                    });

                    let next_episode = m
                        .media
                        .as_ref()
                        .and_then(|next| next.next_airing_episode.clone())
                        .map(|airing| NextAiringEpisode {
                            airing_at: airing.airing_at,
                            episode: airing.episode,
                        });
                    let mapped_status: Option<app_helper_structs::MediaStatus> =
                        m.status.map(|s| s.into());

                    items.push(MediaListItem {
                        id,
                        title,
                        progress: m.progress,
                        total,
                        status: mapped_status,
                        next_airing_episode: next_episode,
                    });
                }
            }
        }

        UserMediaList {
            page_info,
            user_id,
            items: Some(items),
        }
    }
    pub fn browse_media_to_user_list(
        data: crate::anilist::get_media::ResponseData,
        type_: MediaType,
    ) -> UserMediaList {
        let mut page_info = PageInfo {
            current_page: 1,
            per_page: 50,
            total: None,
            last_page: None,
            has_next_page: None,
        };
        let mut items = Vec::new();

        if let Some(page) = data.page {
            if let Some(pi) = page.page_info {
                page_info.current_page = pi.current_page.unwrap_or(1);
                page_info.per_page = pi.per_page.unwrap_or(50);
                page_info.total = pi.total;
                page_info.last_page = pi.last_page;
                page_info.has_next_page = pi.has_next_page;
            }

            if let Some(media_array) = page.media {
                for m in media_array.into_iter().flatten() {
                    let id = m.id;
                    let title = m
                        .title
                        .as_ref()
                        .and_then(|t| t.user_preferred.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let total = match type_ {
                        MediaType::Anime => m.episodes,
                        MediaType::Manga => m.chapters,
                    };

                    let next_episode =
                        m.next_airing_episode
                            .as_ref()
                            .map(|airing| NextAiringEpisode {
                                airing_at: airing.airing_at,
                                episode: airing.episode,
                            });

                    items.push(MediaListItem {
                        id,
                        title,
                        progress: None,
                        total,
                        status: None,
                        next_airing_episode: next_episode,
                    });
                }
            }
        }

        UserMediaList {
            page_info,
            user_id: 0,
            items: Some(items),
        }
    }
}

pub type AppAction = Box<dyn FnOnce(&mut App) + Send>;

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    client: crate::anilist::AnilistClient,
    tx: Sender<AppAction>,
    rx: &Receiver<AppAction>,
) -> io::Result<bool>
where
    std::io::Error: From<<B as Backend>::Error>,
{
    spawn_initial_viewer_fetch(client.clone(), tx.clone());

    loop {
        terminal.draw(|f| ui(f, app))?;

        while let Ok(action) = rx.try_recv() {
            action(app);
        }

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            if key.code == KeyCode::Char('q') {
                return Ok(true);
            }

            match app.active_block {
                ActiveBlock::Sidebar => {
                    keybinds::handle_sidebar_events(app, key, client.clone(), tx.clone())
                }
                ActiveBlock::Center => {
                    keybinds::handle_center_events(app, key, client.clone(), tx.clone())
                }
                _ => {}
            }
        }
    }
}

fn spawn_initial_viewer_fetch(client: crate::anilist::AnilistClient, tx: Sender<AppAction>) {
    let client_clone = client.clone();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let result = client_clone.get_basic_viewer().await;

        let action: AppAction = Box::new(move |app: &mut App| {
            if let Ok(data) = result {
                if let Some(viewer) = data.viewer {
                    let allows_nsfw = viewer.options.and_then(|o| o.display_adult_content);

                    app.authenticated(viewer.id, viewer.name, allows_nsfw);
                }
            }
        });

        let _ = tx_clone.send(action);
    });
}
