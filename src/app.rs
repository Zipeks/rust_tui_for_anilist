use crate::keybinds;
use crate::ui::ui;
use crate::utils::Utils;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{self};
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::widgets::{ListState, TableState};
use ratatui_image::FontSize;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{picker::Picker, protocol::Protocol};
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use crate::anilist::{get_media, get_user_media_list};
use crate::app_helper_structs::{
    ActiveBlock, BrowseCategory, BrowseState, CurrentView, MediaDetails, MediaListItem, MediaType, NextAiringEpisode, PageInfo, Season, TitleLanguage, User, UserMediaList
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
    pub image_picker: Picker,
    pub image_cache: HashMap<i64, StatefulProtocol>,
    pub currently_fetching_image: Option<i64>,

    pub media_details: Option<MediaDetails>,
    pub title_language: TitleLanguage,

    pub show_language_popup: bool,
    pub language_popup_index: usize,
}

impl App {
    pub fn new() -> App {
        let mut state = ListState::default();
        state.select(Some(0));
        let mut picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
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
            image_picker: picker,
            image_cache: HashMap::new(),
            currently_fetching_image: None,
            media_details: None,
            title_language: TitleLanguage::UserPreferred,
            show_language_popup: false,
            language_popup_index: 0,

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
        if self.is_loading {
            return;
        }
        let user_id = self.user.as_ref().map(|u| u.id).unwrap_or(0);
        self.is_loading = true;
        self.error_message = None;

        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let timeout_duration = Duration::from_secs(5);
            let fetch_future = client_clone.get_user_media_list(
                user_id,
                status,
                sort,
                page,
                per_page,
                graphql_type,
            );

            let timeout_result = tokio::time::timeout(timeout_duration, fetch_future).await;
            let action: AppAction = Box::new(move |app: &mut App| {
                app.is_loading = false;
                match timeout_result {
                    Ok(Ok(data)) => {
                        let clean_list= UserMediaList::from(data);
                        app.browse_state.media = Some(clean_list);
                        app.browse_state.state.select_first();
                    }
                    Ok(Err(api_error)) => {
                        app.error_message = Some(format!("API error: {}", api_error));
                    }
                    Err(_) => {
                        app.error_message = Some("Server timout".to_string());
                    }
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
            if media.page_info.current_page > 1 {
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
                    _ => None,
                },
                CurrentView::BrowseManga => match self.browse_state.current_category {
                    BrowseCategory::CategoryOne => Some(vec![get_media::MediaSort::TRENDING_DESC]),
                    BrowseCategory::CategoryTwo => {
                        Some(vec![get_media::MediaSort::POPULARITY_DESC])
                    }
                    BrowseCategory::CategoryThree => Some(vec![get_media::MediaSort::SCORE_DESC]),
                    _ => None,
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
                BrowseCategory::CategoryTwo => {
                    Some(Utils::get_season().to_get_media_media_season())
                }
                BrowseCategory::CategoryThree => {
                    Some(Utils::get_season().next().to_get_media_media_season())
                }
                _ => None,
            },
            match self.browse_state.current_category {
                BrowseCategory::CategoryTwo | BrowseCategory::CategoryThree => {
                    Some(Utils::get_year())
                }
                _ => None,
            },
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
        if self.is_loading {
            return;
        }
        self.is_loading = true;
        self.error_message = None;

        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let timeout_duration = Duration::from_secs(5);
            let fetch_future = client_clone.get_media(
                graphql_type,
                season,
                season_year,
                status,
                sort,
                page,
                per_page,
                search,
                format,
            );
            let timeout_result = tokio::time::timeout(timeout_duration, fetch_future).await;

            let action: AppAction = Box::new(move |app: &mut App| {
                app.is_loading = false;

                match timeout_result {
                    Ok(Ok(data)) => {
                        let clean_list = UserMediaList::from(data);
                        app.browse_state.media = Some(clean_list);
                        app.browse_state.state.select_first();
                    }
                    Ok(Err(api_error)) => {
                        app.error_message = Some(format!("API error: {}", api_error));
                    }
                    Err(_) => {
                        app.error_message = Some("Server timout".to_string());
                    }
                }
            });

            let _ = tx_clone.send(action);
        });
    }
    pub fn fetch_media_details(
        &mut self,
        client: crate::anilist::AnilistClient,
        tx: Sender<AppAction>,
    ) {
        if self.is_loading {
            return;
        }
        let selected_index = self.browse_state.state.selected();
        let current_items = self.get_current_center_items();

        let Some(idx) = selected_index else {
            return;
        };
        if idx >= current_items.len() {
            return;
        }

        let media_id = current_items[idx].id;

        let media_type = match self.current_view {
            CurrentView::UserAnime | CurrentView::BrowseAnime => MediaType::Anime,
            CurrentView::UserManga | CurrentView::BrowseManga => MediaType::Manga,
        };

        self.is_loading = true;
        self.error_message = None;

        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let timeout_duration = Duration::from_secs(5);
            let fetch_future = client_clone.get_media_details(media_id, media_type);
            let timeout_result = tokio::time::timeout(timeout_duration, fetch_future).await;

            let action: AppAction = Box::new(move |app: &mut App| {
                app.is_loading = false;

                match timeout_result {
                    Ok(Ok(data)) => {
                        let media_details = MediaDetails::from(data);
                        app.media_details = Some(media_details);
                    }
                    Ok(Err(api_error)) => {
                        app.error_message = Some(format!("API error: {}", api_error));
                    }
                    Err(_) => {
                        app.error_message = Some("Server timeout".to_string());
                    }
                }
            });
            let _ = tx_clone.send(action);
        });
    }
    pub fn fetch_cover(&mut self, media_id: i64, url: String, tx: Sender<AppAction>) {
        if self.image_cache.contains_key(&media_id)
            || self.currently_fetching_image == Some(media_id)
        {
            return;
        }

        self.currently_fetching_image = Some(media_id);
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            if let Ok(response) = reqwest::get(&url).await {
                if let Ok(bytes) = response.bytes().await {
                    if let Ok(dyn_image) = image::load_from_memory(&bytes) {
                        let action: AppAction = Box::new(move |app: &mut App| {
                            let protocol = app.image_picker.new_resize_protocol(dyn_image);

                            app.image_cache.insert(media_id, protocol);
                            app.currently_fetching_image = None;
                        });

                        let _ = tx_clone.send(action);
                        return;
                    }
                }
            }

            let action: AppAction = Box::new(move |app: &mut App| {
                app.currently_fetching_image = None;
            });
            let _ = tx_clone.send(action);
        });
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
            if app.show_language_popup {
                keybinds::handle_language_popup_events(app, key);
                continue;
            }

            match app.active_block {
                ActiveBlock::Sidebar => {
                    keybinds::handle_sidebar_events(app, key, client.clone(), tx.clone())
                }
                ActiveBlock::Center => {
                    keybinds::handle_center_events(app, key, client.clone(), tx.clone())
                }
                ActiveBlock::Details => {
                    keybinds::handle_details_events(app, key, client.clone(), tx.clone())
                }
            }
        }
    }
}

fn spawn_initial_viewer_fetch(client: crate::anilist::AnilistClient, tx: Sender<AppAction>) {
    let client_clone = client.clone();
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let timeout_duration = Duration::from_secs(5);
        let fetch_future = client_clone.get_basic_viewer();

        let timeout_result = tokio::time::timeout(timeout_duration, fetch_future).await;
        let action: AppAction = Box::new(move |app: &mut App| {
            if let Ok(Ok(data)) = timeout_result {
                if let Some(viewer) = data.viewer {
                    let allows_nsfw = viewer.options.and_then(|o| o.display_adult_content);

                    app.authenticated(viewer.id, viewer.name, allows_nsfw);
                }
            }
        });

        let _ = tx_clone.send(action);
    });
}
