mod content;
mod footer;
mod header;
mod main_frame;
mod sidebar;

use crate::app::App;
use ratatui::prelude::*;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    header::draw(frame, chunks[0], app);

    main_frame::draw(frame, chunks[1], app);

    footer::draw(frame, chunks[2], app);
}
