use crate::ui::ui;
use crate::{app_helper_structs, keybinds};
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
    ActiveBlock, BrowseCategory, BrowseState, CurrentView, MediaListItem, MediaTab, MediaType,
    NextAiringEpisode, PageInfo, User, UserMediaList,
};

pub struct App {
    pub active_block: ActiveBlock,
    pub current_view: CurrentView,
    pub user: Option<User>,
    pub sidebar_items: Vec<CurrentView>,
    pub sidebar_state: ListState,

    pub is_loading: bool,
    pub error_message: Option<String>,

    pub user_anime: Option<UserMediaList>,
    pub user_anime_state: TableState,

    pub user_manga: Option<UserMediaList>,
    pub user_manga_state: TableState,

    pub active_tab: MediaTab,

    pub browse_anime: BrowseState,
    pub browse_manga: BrowseState,
}

impl App {
    pub fn new() -> App {
        let mut state = ListState::default();
        state.select(Some(0));

        App {
            active_block: ActiveBlock::Sidebar,
            current_view: CurrentView::Home,
            sidebar_items: CurrentView::ALL.to_vec(),
            sidebar_state: state,
            is_loading: false,
            user: None,

            error_message: None,
            user_anime: None,
            user_manga: None,
            user_anime_state: TableState::default(),
            user_manga_state: TableState::default(),
            active_tab: MediaTab::Anime,

            browse_anime: BrowseState {
                media: None,
                state: TableState::default(),
                current_category: BrowseCategory::CategoryOne,
            },
            browse_manga: BrowseState {
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
        match self.current_view {
            CurrentView::Home => match self.active_tab {
                MediaTab::Anime => self
                    .user_anime
                    .as_ref()
                    .and_then(|l| l.items.as_deref())
                    .unwrap_or(&[]),
                MediaTab::Manga => self
                    .user_manga
                    .as_ref()
                    .and_then(|l| l.items.as_deref())
                    .unwrap_or(&[]),
            },
            CurrentView::BrowseAnime => self
                .browse_anime
                .media
                .as_ref()
                .and_then(|l| l.items.as_deref())
                .unwrap_or(&[]),
            CurrentView::BrowseManga => self
                .browse_manga
                .media
                .as_ref()
                .and_then(|l| l.items.as_deref())
                .unwrap_or(&[]),
            _ => &[],
        }
    }
    pub fn next_center_item(&mut self) {
        let count = self.get_current_center_items().len();
        if count == 0 {
            return;
        }

        let state = match self.current_view {
            CurrentView::Home => match self.active_tab {
                MediaTab::Anime => &mut self.user_anime_state,
                MediaTab::Manga => &mut self.user_manga_state,
            },
            CurrentView::BrowseAnime => &mut self.browse_anime.state,
            CurrentView::BrowseManga => &mut self.browse_manga.state,
            _ => return,
        };

        let current = state.selected().unwrap_or(0);
        state.select(Some((current + 1) % count));
    }

    pub fn previous_center_item(&mut self) {
        let count = self.get_current_center_items().len();
        if count == 0 {
            return;
        }

        let state = match self.current_view {
            CurrentView::Home => match self.active_tab {
                MediaTab::Anime => &mut self.user_anime_state,
                MediaTab::Manga => &mut self.user_manga_state,
            },
            CurrentView::BrowseAnime => &mut self.browse_anime.state,
            CurrentView::BrowseManga => &mut self.browse_manga.state,
            _ => return,
        };

        let current = state.selected().unwrap_or(0);
        state.select(Some((current + count - 1) % count));
    }
    pub fn fetch_home_data(
        &mut self,
        client: crate::anilist::AnilistClient,
        tx: Sender<AppAction>,
    ) {
        if self.is_loading || self.user_anime.is_some() || self.user_manga.is_some() {
            return;
        }

        use crate::anilist::get_user_media_list::{MediaListStatus, MediaType as GraphQlMediaType};

        self.fetch_user_media_list(
            client.clone(),
            tx.clone(),
            Some(MediaListStatus::CURRENT),
            None,
            None,
            None,
            GraphQlMediaType::ANIME,
            MediaType::Anime,
        );

        self.fetch_user_media_list(
            client,
            tx,
            Some(MediaListStatus::CURRENT),
            None,
            None,
            None,
            GraphQlMediaType::MANGA,
            MediaType::Manga,
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
        app_type: MediaType,
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
                        let clean_list = App::to_user_media_list(data, user_id, app_type);

                        match app_type {
                            MediaType::Anime => {
                                app.user_anime = Some(clean_list);
                                app.user_anime_state.select_first();
                            }
                            MediaType::Manga => {
                                app.user_manga = Some(clean_list);
                                app.user_manga_state.select_first();
                            }
                        }
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            });
            let _ = tx_clone.send(action);
        });
    }

    pub fn next_center_page(&mut self) {
        match self.current_view {
            CurrentView::Home => match self.active_tab {
                MediaTab::Anime => {
                    if let Some(media) = &mut self.user_anime {
                        if media.page_info.has_next_page.unwrap_or(false) {
                            media.page_info.current_page = media.page_info.current_page + 1;
                        }
                    }
                }
                MediaTab::Manga => {
                    if let Some(media) = &mut self.user_manga {
                        if media.page_info.has_next_page.unwrap_or(false) {
                            media.page_info.current_page = media.page_info.current_page + 1;
                        }
                    }
                }
            },
            CurrentView::BrowseAnime => {
                if let Some(media) = &mut self.browse_anime.media {
                    if media.page_info.has_next_page.unwrap_or(false) {
                        media.page_info.current_page = media.page_info.current_page + 1;
                    }
                }
            }
            CurrentView::BrowseManga => {
                if let Some(media) = &mut self.browse_manga.media {
                    if media.page_info.has_next_page.unwrap_or(false) {
                        media.page_info.current_page = media.page_info.current_page + 1;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn previous_center_page(&mut self) {
        match self.current_view {
            CurrentView::Home => match self.active_tab {
                MediaTab::Anime => {
                    if let Some(media) = &mut self.user_anime {
                        if media.page_info.current_page > 1 {
                            media.page_info.current_page = media.page_info.current_page - 1;
                        }
                    }
                }
                MediaTab::Manga => {
                    if let Some(media) = &mut self.user_manga {
                        if media.page_info.current_page > 1 {
                            media.page_info.current_page = media.page_info.current_page - 1;
                        }
                    }
                }
            },
            CurrentView::BrowseAnime => {
                if let Some(media) = &mut self.browse_anime.media {
                    if media.page_info.current_page > 1 {
                        media.page_info.current_page = media.page_info.current_page - 1
                    }
                }
            }
            CurrentView::BrowseManga => {
                if let Some(media) = &mut self.browse_manga.media {
                    if media.page_info.current_page > 1 {
                        media.page_info.current_page = media.page_info.current_page - 1
                    }
                }
            }
            _ => {}
        }
    }

    pub fn fetch_browse(&mut self, client: crate::anilist::AnilistClient, tx: Sender<AppAction>) {
        self.fetch_media(
            client,
            tx,
            None,
            match self.current_view {
                CurrentView::BrowseAnime => match self.browse_anime.current_category {
                    BrowseCategory::CategoryOne => Some(vec![get_media::MediaSort::TRENDING_DESC]),
                    BrowseCategory::CategoryTwo | BrowseCategory::CategoryThree => {
                        Some(vec![get_media::MediaSort::POPULARITY_DESC])
                    }
                    _ => todo!(),
                },
                CurrentView::BrowseManga => match self.browse_anime.current_category {
                    BrowseCategory::CategoryOne => Some(vec![get_media::MediaSort::TRENDING_DESC]),
                    _ => todo!(),
                },
                _ => todo!(),
            },
            Some({
                match self.current_view {
                    CurrentView::BrowseAnime => {
                        if let Some(media) = &self.browse_anime.media {
                            media.page_info.current_page
                        } else {
                            1
                        }
                    }
                    CurrentView::BrowseManga => {
                        if let Some(media) = &self.browse_manga.media {
                            media.page_info.current_page
                        } else {
                            1
                        }
                    }
                    _ => 1,
                }
            }),
            Some(25),
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
            None,
            None,
            None,
            None,
        );
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

                        match app_type {
                            MediaType::Anime => {
                                app.browse_anime.media = Some(clean_list);
                                app.browse_anime.state.select_first();
                            }
                            MediaType::Manga => {
                                app.browse_manga.media = Some(clean_list);
                                app.browse_manga.state.select_first();
                            }
                        }
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

            if let Some(media_list) = page.media_list {
                for m in media_list.into_iter().flatten() {
                    let id = m.media.as_ref().map(|x| x.id).unwrap_or(0);
                    let title = m
                        .media
                        .as_ref()
                        .and_then(|x| x.title.as_ref())
                        .and_then(|t| t.user_preferred.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let total = m.media.as_ref().and_then(|x| match type_ {
                        MediaType::Anime => x.episodes,
                        MediaType::Manga => x.chapters,
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
            type_,
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
            type_,
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
