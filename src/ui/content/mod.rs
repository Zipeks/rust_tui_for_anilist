use crate::{app::App};
use ratatui::prelude::*;

mod browse;
mod draw_media_list;
pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    browse::draw(frame, area, app);
}
