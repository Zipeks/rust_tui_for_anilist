use crate::{
    app::{App, AppAction},
    app_helper_structs::{ActiveBlock, CurrentView, MediaTab},
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
                app.current_view = app.sidebar_items[selected_idx];
                // app.current_media_state.select(Some(0));
            }
            app.active_block = ActiveBlock::Center;

            match app.current_view {
                CurrentView::Home => app.fetch_home_data(client, tx),
                CurrentView::BrowseAnime => app.fetch_browse(client, tx),
                CurrentView::BrowseManga => app.fetch_browse(client, tx),
                _ => {}
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

        KeyCode::Char('[') => {
            match app.current_view {
                CurrentView::Home => app.active_tab = app.active_tab.previous(),
                CurrentView::BrowseAnime => app.browse_anime.current_category = app.browse_anime.current_category.previous(),
                CurrentView::BrowseManga => app.browse_manga.current_category = app.browse_manga.current_category.previous(),
                _ => todo!()
            }
        }
        KeyCode::Char(']') => {
            match app.current_view {
                CurrentView::Home => app.active_tab = app.active_tab.next(),
                CurrentView::BrowseAnime => app.browse_anime.current_category = app.browse_anime.current_category.next(),
                CurrentView::BrowseManga => app.browse_manga.current_category = app.browse_manga.current_category.next(),
                _ => todo!()
            }
        }

        KeyCode::Char('j') | KeyCode::Down => app.next_center_item(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_center_item(),
        KeyCode::Enter => {
            let current_state = match app.current_view {
                CurrentView::Home => match app.active_tab {
                    MediaTab::Anime => &app.user_anime_state,
                    MediaTab::Manga => &app.user_manga_state,
                },
                CurrentView::BrowseAnime => &app.browse_anime.state,
                CurrentView::BrowseManga => &app.browse_manga.state,
                _ => return,
            };

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
                CurrentView::Home => app.fetch_home_data(client, tx),
                _ => {}
            }
        }
        KeyCode::Char('p') => {
            app.previous_center_page();
            match app.current_view {
                CurrentView::BrowseAnime | CurrentView::BrowseManga => app.fetch_browse(client, tx),
                CurrentView::Home => app.fetch_home_data(client, tx),
                _ => {}
            }
        }

        _ => {}
    }
}
