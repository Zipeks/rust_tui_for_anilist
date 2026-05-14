use crate::anilist::get_user_media_list::{self, MediaListSort, MediaListStatus};
use crate::app::MediaType::Anime;
use crate::app::Season::ANY;
use crate::keybinds;
use crate::ui::ui;
use chrono::{Datelike, Utc};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self, KeyEvent};
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::widgets::{ListItem, ListState, TableState};
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

#[derive(PartialEq)]
pub enum ActiveBlock {
    Sidebar,
    Center,
    Details,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CurrentView {
    Home,
    Browse,
    Profile,
}

impl CurrentView {
    pub const ALL: [CurrentView; 3] =
        [CurrentView::Home, CurrentView::Browse, CurrentView::Profile];

    pub fn to_string(&self) -> &'static str {
        match &self {
            CurrentView::Home => "Home",
            CurrentView::Browse => "Browse",
            CurrentView::Profile => "Profile",
        }
    }
}

#[derive(Clone)]
pub struct User {
    id: i64,
    name: String,
    allows_nsfw: Option<bool>,
}

impl User {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum MediaTab {
    Anime,
    Manga,
}

impl MediaTab {
    pub fn next(&self) -> Self {
        match self {
            MediaTab::Anime => MediaTab::Manga,
            MediaTab::Manga => MediaTab::Anime,
        }
    }

    pub fn previous(&self) -> Self {
        self.next()
    }
}
pub struct PageInfo {
    pub current_page: i64,
    pub per_page: i64,
    pub total: Option<i64>,
    pub last_page: Option<i64>,
    pub has_next_page: Option<bool>,
}
pub struct UserMediaList {
    pub page_info: PageInfo,
    pub user_id: i64,
    pub media_list_status: Option<Vec<MediaListStatus>>,
    pub type_: MediaType,
    pub items: Option<Vec<MediaListItem>>,
}

#[derive(Clone, Debug)]
pub struct NextAiringEpisode {
    pub episode: i64,
    pub airing_at: i64,
}
#[derive(Clone, Debug)]
pub struct MediaListItem {
    pub id: i64,
    pub title: String,
    pub progress: Option<i64>,
    pub total: Option<i64>,
    pub status: Option<i64>,
    pub next_airing_episode: Option<NextAiringEpisode>,
}

pub struct App {
    pub active_block: ActiveBlock,
    pub current_view: CurrentView,
    pub search_settings: SearchSettings,
    pub user: Option<User>,
    pub sidebar_items: Vec<CurrentView>,
    pub sidebar_state: ListState,

    pub is_loading: bool,
    pub error_message: Option<String>,

    pub user_anime: Option<UserMediaList>,
    pub anime_state: TableState,

    pub user_manga: Option<UserMediaList>,
    pub manga_state: TableState,

    pub active_tab: MediaTab,
}

impl App {
    pub fn new() -> App {
        let mut state = ListState::default();
        state.select(Some(0));

        App {
            active_block: ActiveBlock::Sidebar,
            current_view: CurrentView::Home,
            search_settings: SearchSettings {
                search_input: String::from(""),
                media_year: Utc::now().year(),
                media_season: ANY,
                media_type: Anime,
            },
            sidebar_items: CurrentView::ALL.to_vec(),
            sidebar_state: state,
            is_loading: false,
            user: None,

            error_message: None,
            user_anime: None,
            user_manga: None,
            active_tab: MediaTab::Anime,
            anime_state: TableState::default(),
            manga_state: TableState::default(),
        }
    }

    pub fn get_current_tab_items(&self) -> &[MediaListItem] {
        let active_list = match self.active_tab {
            MediaTab::Anime => &self.user_anime,
            MediaTab::Manga => &self.user_manga,
        };

        active_list
            .as_ref()
            .and_then(|l| l.items.as_deref())
            .unwrap_or(&[])
    }

    pub fn next_sidebar_item(&mut self) {
        let i = match self.sidebar_state.selected() {
            Some(i) => {
                if i >= self.sidebar_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.sidebar_state.select(Some(i));
    }

    pub fn previous_sidebar_item(&mut self) {
        let i = match self.sidebar_state.selected() {
            Some(i) => {
                if i <= 0 {
                    self.sidebar_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.sidebar_state.select(Some(i));
    }

    pub fn next_center_item(&mut self) {
        let count = self.get_current_tab_items().len();
        if count == 0 {
            return;
        }

        let state = match self.active_tab {
            MediaTab::Anime => &mut self.anime_state,
            MediaTab::Manga => &mut self.manga_state,
        };

        let i = match state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        state.select(Some(i));
    }

    pub fn previous_center_item(&mut self) {
        let count = self.get_current_tab_items().len();
        if count == 0 {
            return;
        }

        let state = match self.active_tab {
            MediaTab::Anime => &mut self.anime_state,
            MediaTab::Manga => &mut self.manga_state,
        };

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        state.select(Some(i));
    }

    pub fn authenticated(&mut self, id: i64, name: String, allows_nsfw: Option<bool>) {
        self.user = Some(User {
            id,
            name,
            allows_nsfw,
        })
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
                                app.anime_state.select_first();
                            }
                            MediaType::Manga => {
                                app.user_manga = Some(clean_list);
                                app.manga_state.select_first();
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
                        .and_then(|next_episode| next_episode.next_airing_episode.clone())
                        .and_then(|next_airing_episode| {
                            Some(NextAiringEpisode {
                                airing_at: next_airing_episode.airing_at,
                                episode: next_airing_episode.episode,
                            })
                        });

                    items.push(MediaListItem {
                        id,
                        title,
                        progress: m.progress,
                        total,
                        status: None,
                        next_airing_episode: next_episode,
                    });
                }
            }
        }

        UserMediaList {
            page_info,
            user_id,
            media_list_status: None,
            type_,
            items: Some(items),
        }
    }
}

#[derive(Clone, Copy)]
pub enum MediaType {
    Anime,
    Manga,
}
pub enum Season {
    WINTER,
    SPRING,
    SUMMER,
    FALL,
    ANY,
}
pub struct SearchSettings {
    search_input: String,
    media_type: MediaType,
    media_year: i32,
    media_season: Season,
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
                ActiveBlock::Center => keybinds::handle_center_events(app, key),
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
//     }

//     loop {
//         terminal.draw(|f| ui(f, app))?;

//         while let Ok(action) = rx.try_recv() {
//             action(app);
//         }

//         if event::poll(Duration::from_millis(50))? {
//             if let Event::Key(key) = event::read()? {
//                 if key.kind == event::KeyEventKind::Release {
//                     continue;
//                 }

//                 match app.active_block {
//                     ActiveBlock::Sidebar => sidebar_actions(terminal, app, client, &tx, &rx, key),
//                     ActiveBlock::Center => match key.code {
//                         KeyCode::Char('h') | KeyCode::BackTab | KeyCode::Esc => {
//                             app.active_block = ActiveBlock::Sidebar
//                         }
//                         KeyCode::Char('j') | KeyCode::Down => app.next_center_item(),
//                         KeyCode::Char('k') | KeyCode::Up => app.previous_center_item(),
//                         _ => {}
//                     },

//                     _ => {}
//                 }

//                 if let KeyCode::Char('q') = key.code {
//                     return Ok(true);
//                 }
//             }
//         }
//     }
// }

// fn sidebar_actions<B: Backend>(
//     terminal: &mut Terminal<B>,
//     app: &mut App,
//     client: crate::anilist::AnilistClient,
//     tx: &Sender<AppAction>,
//     rx: &Receiver<AppAction>,
//     key: KeyEvent,
// ) where
//     io::Error: From<B::Error>,
// {
//     match key.code {
//         KeyCode::Char('l') | KeyCode::Enter => {
//             if let Some(selected_idx) = app.sidebar_state.selected() {
//                 app.current_view = app.sidebar_items[selected_idx];
//                 app.current_media_state.select(Some(0));
//             }

//             app.active_block = ActiveBlock::Center;

//             match app.current_view {
//                 CurrentView::Home => {
//                     if app.current_media.is_none() && !app.is_loading {
//                         let user_id = app.user.as_ref().map(|u| u.id).unwrap_or(0);

//                         app.is_loading = true;
//                         app.error_message = None;

//                         let client_clone = client.clone();
//                         let tx_clone = tx.clone();

//                         tokio::spawn(async move {
//                             let result = client_clone.get_current_media(user_id).await;

//                             let action: AppAction = Box::new(move |app: &mut App| {
//                                 app.is_loading = false;
//                                 match result {
//                                     Ok(data) => {
//                                         app.current_media = Some(data);
//                                         let selectables = app.get_selectable_indices();
//                                         if let Some(&first) = selectables.first() {
//                                             app.current_media_state.select(Some(first));
//                                         }
//                                     }
//                                     Err(e) => app.error_message = Some(e.to_string()),
//                                 }
//                             });

//                             let _ = tx_clone.send(action);
//                         });
//                     }
//                 }
//                 KeyCode::Char('j') | KeyCode::Down => app.next_sidebar_item(),
//                 KeyCode::Char('k') | KeyCode::Up => app.previous_sidebar_item(),
//                 _ => {}
//             }
//         }
//     }
// }
