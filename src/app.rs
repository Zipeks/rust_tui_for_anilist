use crate::anilist::get_anime::{self, MediaSeason};
use crate::anilist::get_current_media::{self, MediaListStatus};
use crate::app::MediaType::Anime;
use crate::app::Season::ANY;
use crate::ui::ui;
use chrono::{Datelike, Utc};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self, KeyEvent};
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::widgets::{ListItem, ListState};
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
#[derive(Clone, Debug)]
pub struct MediaItem {
    pub id: i64,
    pub title: String,
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

    pub current_media: Option<get_current_media::ResponseData>,
    pub active_tab: MediaTab,
    pub anime_state: ListState,
    pub manga_state: ListState,
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
            current_media: None,
            active_tab: MediaTab::Anime,
            anime_state: ListState::default(),
            manga_state: ListState::default(),
        }
    }

    pub fn get_current_tab_items(&self) -> Vec<MediaItem> {
        let Some(ref data) = self.current_media else {
            return vec![];
        };

        let raw_list = data
            .page
            .as_ref()
            .and_then(|p| p.media_list.as_ref())
            .map(|l| l.iter().flatten().collect::<Vec<_>>())
            .unwrap_or_default();

        let target_type = match self.active_tab {
            MediaTab::Anime => get_current_media::MediaType::ANIME,
            MediaTab::Manga => get_current_media::MediaType::MANGA,
        };

        raw_list
            .into_iter()
            .filter(|m| {
                m.media.as_ref().map_or(
                    false,
                    |med| matches!(med.type_, Some(ref t) if *t == target_type),
                )
            })
            .map(|m| {
                let title = m
                    .media
                    .as_ref()
                    .and_then(|med| med.title.as_ref())
                    .and_then(|t| t.user_preferred.clone())
                    .unwrap_or_else(|| "Unknown".into());

                let id = m.media.as_ref().map(|med| med.id).unwrap_or(0);

                MediaItem { id, title }
            })
            .collect()
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
        if self.is_loading || self.current_media.is_some() {
            return;
        }

        let user_id = self.user.as_ref().map(|u| u.id).unwrap_or(0);
        self.is_loading = true;
        self.error_message = None;

        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = client_clone.get_current_media(user_id).await;
            let action: AppAction = Box::new(move |app: &mut App| {
                app.is_loading = false;
                match result {
                    Ok(data) => {
                        app.current_media = Some(data);
                        app.anime_state.select_first();
                        app.manga_state.select_first();
                    }
                    Err(e) => app.error_message = Some(e.to_string()),
                }
            });
            let _ = tx_clone.send(action);
        });
    }
}
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
                ActiveBlock::Sidebar => handle_sidebar_events(app, key, client.clone(), tx.clone()),
                ActiveBlock::Center => handle_center_events(app, key),
                _ => {}
            }
        }
    }
}

fn handle_sidebar_events(
    app: &mut App,
    key: KeyEvent,
    client: crate::anilist::AnilistClient,
    tx: Sender<AppAction>,
) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_sidebar_item(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_sidebar_item(),

        KeyCode::Char('l') | KeyCode::Enter => {
            if let Some(selected_idx) = app.sidebar_state.selected() {
                app.current_view = app.sidebar_items[selected_idx];
                // app.current_media_state.select(Some(0));
            }
            app.active_block = ActiveBlock::Center;

            match app.current_view {
                CurrentView::Home => app.fetch_home_data(client, tx),
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_center_events(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('h') | KeyCode::Esc => app.active_block = ActiveBlock::Sidebar,

        KeyCode::Char('[') => {
            app.active_tab = app.active_tab.previous();
        }
        KeyCode::Char(']') => {
            app.active_tab = app.active_tab.next();
        }

        KeyCode::Char('j') | KeyCode::Down => app.next_center_item(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_center_item(),
        KeyCode::Enter => {
            let current_state = match app.active_tab {
                MediaTab::Anime => &app.anime_state,
                MediaTab::Manga => &app.manga_state,
            };

            if let Some(selected_index) = current_state.selected() {
                let current_items = app.get_current_tab_items();

                if selected_index < current_items.len() {
                    let selected_id = current_items[selected_index].id;
                    let selected_title = &current_items[selected_index].title;

                    // Teraz możesz zmienić ekran na Details i wysłać zapytanie o to ID
                    // np. app.fetch_anime_details(selected_id, tx);
                    // app.active_block = ActiveBlock::Details;
                }
            }
        }
        _ => {}
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
