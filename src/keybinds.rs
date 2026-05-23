use crate::{
    app::{App, AppAction},
    app_helper_structs::{ActiveBlock, BrowseCategory, CurrentView, TitleLanguage},
};
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use std::sync::mpsc::Sender;

pub fn handle_sidebar_events(
    app: &mut App,
    key: KeyEvent,
    client: crate::anilist::AnilistClient,
    tx: Sender<AppAction>,
) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_sidebar_item(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_sidebar_item(),
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            if let Some(selected_idx) = app.sidebar_state.selected() {
                let new_view = app.sidebar_items[selected_idx];

                if new_view != app.current_view {
                    app.current_view = new_view;
                    app.browse_state.current_category = BrowseCategory::CategoryOne;
                    app.browse_state.media = None;
                }
            }
            app.active_block = ActiveBlock::Center;

            match app.current_view {
                CurrentView::UserAnime | CurrentView::UserManga => app.fetch_user_media(client, tx),
                CurrentView::BrowseAnime | CurrentView::BrowseManga => app.fetch_browse(client, tx),
            }
        }
        _ => {}
    }
}

pub fn handle_center_events(
    app: &mut App,
    key: KeyEvent,
    client: crate::anilist::AnilistClient,
    tx: Sender<AppAction>,
) {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => {
            app.active_block = ActiveBlock::Sidebar;
            app.error_message = None;
        }

        KeyCode::Char('[') | KeyCode::Char(']') | KeyCode::BackTab | KeyCode::Tab => {
            if key.code == KeyCode::Char('[') || key.code == KeyCode::BackTab {
                app.browse_state.current_category = app.browse_state.current_category.previous();
            } else {
                app.browse_state.current_category = app.browse_state.current_category.next();
            }

            app.browse_state.media = None;

            let tx_clone = tx.clone();
            match app.current_view {
                CurrentView::UserAnime | CurrentView::UserManga => {
                    app.fetch_user_media(client, tx_clone)
                }
                CurrentView::BrowseAnime | CurrentView::BrowseManga => {
                    app.fetch_browse(client, tx_clone)
                }
            }
        }

        KeyCode::Char('j') | KeyCode::Down => app.next_center_item(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_center_item(),
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if let Some(selected_index) = app.browse_state.state.selected() {
                let current_items = app.get_current_center_items();

                if selected_index < current_items.len() {
                    app.fetch_media_details(client, tx);
                    app.active_block = ActiveBlock::Details;
                }
            }
        }
        KeyCode::Char('n') => {
            app.next_center_page();
            match app.current_view {
                CurrentView::BrowseAnime | CurrentView::BrowseManga => app.fetch_browse(client, tx),
                CurrentView::UserAnime | CurrentView::UserManga => app.fetch_user_media(client, tx),
            }
        }
        KeyCode::Char('p') => {
            app.previous_center_page();
            match app.current_view {
                CurrentView::BrowseAnime | CurrentView::BrowseManga => app.fetch_browse(client, tx),
                CurrentView::UserAnime | CurrentView::UserManga => app.fetch_user_media(client, tx),
            }
        }
        KeyCode::Char('t') => {
            app.show_language_popup = true;
            app.language_popup_index = TitleLanguage::ALL
                .iter()
                .position(|l| l == &app.title_language)
                .unwrap_or(0);
        }
        _ => {}
    }
}

pub fn handle_details_events(
    app: &mut App,
    key: KeyEvent,
    client: crate::anilist::AnilistClient,
    tx: Sender<AppAction>,
) {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            app.active_block = ActiveBlock::Center;
            app.media_details = None;
        }
        _ => {}
    }
}

pub fn handle_language_popup_events(app: &mut App, key: KeyEvent) {
    if app.show_language_popup {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => app.show_language_popup = false,

            KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                app.language_popup_index = (app.language_popup_index + 1) % 4;
            }

            KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                app.language_popup_index = (app.language_popup_index + 3) % 4;
            }

            KeyCode::Enter => {
                app.title_language =
                    crate::app_helper_structs::TitleLanguage::ALL[app.language_popup_index];
                app.show_language_popup = false;
            }
            _ => {}
        }
    }
}
