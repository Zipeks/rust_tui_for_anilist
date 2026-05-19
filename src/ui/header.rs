use crate::app::App;
use ratatui::{prelude::*, widgets::*};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_header_area = header_block.inner(area);
    frame.render_widget(header_block, area);

    let header = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(2),
            Constraint::Percentage(25),
            Constraint::Percentage(1),
            Constraint::Percentage(44),
            Constraint::Percentage(1),
            Constraint::Percentage(25),
            Constraint::Percentage(2),
        ])
        .split(inner_header_area);

    let header_info = Paragraph::new(Line::from("Rust tui for AniList")).left_aligned();
    frame.render_widget(header_info, header[1]);

    let spacer = Paragraph::new("│").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(spacer, header[2]);

    let header_current_view = Paragraph::new(app.current_view.to_string()).centered();
    frame.render_widget(header_current_view, header[3]);

    let spacer = Paragraph::new("│").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(spacer, header[4]);

    let header_user_info = Paragraph::new({
        if let Some(user) = &app.user {
            format!("{}", user.get_name())
        } else {
            "Not logged in.".to_string()
        }
    })
    .right_aligned();

    frame.render_widget(header_user_info, header[5]);
}
