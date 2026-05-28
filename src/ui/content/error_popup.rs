use crate::{app::App, ui::centered_rect};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, app: &mut App, error_message: String) {
    let popup_area = centered_rect(60, 18, frame.area());

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let error_p = Paragraph::new(error_message.clone())
        .block(popup_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(error_p, popup_area);
}
