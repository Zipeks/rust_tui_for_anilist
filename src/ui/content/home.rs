use crate::{
    app::{ActiveBlock, App, MediaTab},
    ui::content::draw_media_list,
};

use ratatui::{prelude::*, widgets::*};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let is_center_active = app.active_block == ActiveBlock::Center;

    let (media_items, active_state) = match app.active_tab {
        MediaTab::Anime => {
            let items = app
                .user_anime
                .as_ref()
                .and_then(|l| l.items.as_deref())
                .unwrap_or(&[]);
            (items, &mut app.anime_state)
        }
        MediaTab::Manga => {
            let items = app
                .user_manga
                .as_ref()
                .and_then(|l| l.items.as_deref())
                .unwrap_or(&[]);
            (items, &mut app.manga_state)
        }
    };

    draw_media_list::draw(
        frame,
        chunks[0],
        media_items,
        is_center_active,
        active_state,
        app.active_tab,
    );
}
