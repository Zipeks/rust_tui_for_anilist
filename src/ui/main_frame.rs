use crate::{
    app::App,
    app_helper_structs::ActiveBlock,
    ui::{
        content::{browse, details},
        sidebar,
    },
};
use ratatui::{prelude::*, widgets::*};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_main_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints({
            match app.active_block {
                ActiveBlock::Sidebar => [
                    Constraint::Length(20),
                    Constraint::Fill(6),
                    Constraint::Fill(2),
                ],
                ActiveBlock::Center => [
                    Constraint::Length(20),
                    Constraint::Fill(6),
                    Constraint::Fill(2),
                ],
                ActiveBlock::Details => [
                    Constraint::Length(0),
                    Constraint::Fill(3),
                    Constraint::Fill(5),
                ],
            }
        })
        .split(inner_main_area);

    sidebar::draw(frame, center[0], app);

    browse::draw(frame, center[1], app);

    details::draw(frame, center[2], app);
}
