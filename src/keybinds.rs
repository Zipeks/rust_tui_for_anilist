use crate::app::{ActiveBlock, App, AppAction, CurrentView, MediaTab};
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

pub fn handle_center_events(app: &mut App, key: KeyEvent) {
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
