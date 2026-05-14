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

    let media_items = app.get_current_tab_items();

    let titles: Vec<String> = media_items.iter().map(|item| item.title.clone()).collect();

    let is_center_active = app.active_block == ActiveBlock::Center;

    let active_state = match app.active_tab {
        MediaTab::Anime => &mut app.anime_state,
        MediaTab::Manga => &mut app.manga_state,
    };

    draw_media_list::draw(
        frame,
        chunks[0],
        &titles,
        is_center_active,
        active_state,
        app.active_tab,
    );
}