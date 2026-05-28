mod content;
mod footer;
mod header;
mod main_frame;
mod sidebar;
use crate::{app::App, ui::content::error_popup};
use content::language_popup;
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

    if app.show_language_popup {
        language_popup::draw(frame, app);
    }

    if let Some(error_message) = &app.error_message {
        error_popup::draw(frame, app, error_message.clone());
    }
}

pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(r.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(r.height.saturating_sub(height) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(r.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(r.width.saturating_sub(width) / 2),
        ])
        .split(popup_layout[1])[1]
}
