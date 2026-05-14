use crate::app::{ActiveBlock, App, CurrentView};
use ratatui::{prelude::*, widgets::*};

mod draw_media_list;
mod home;
pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    // let block = Block::default().borders(Borders::NONE);

    // let inner_area = block.inner(area);

    // frame.render_widget(block, area);

    match app.current_view {
        CurrentView::Home => {
            home::draw(frame, area, app);
        }
        _ => {}
    }
}
