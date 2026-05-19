use crate::{
    app::{App, AppAction},
    app_helper_structs::{ActiveBlock, BrowseCategory, CurrentView},
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
        KeyCode::Enter => {
            let current_state = app.browse_state.state;
            if let Some(selected_index) = current_state.selected() {
                let current_items = app.get_current_center_items();

                if selected_index < current_items.len() {
                    let selected_id = current_items[selected_index].id;
                    let selected_title = &current_items[selected_index].title;
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
        _ => {}
    }
}
