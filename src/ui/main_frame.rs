use crate::{
    app::{ActiveBlock, App},
    ui::{content, sidebar},
};
use ratatui::{layout::Spacing, prelude::*, widgets::*};

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
                    Constraint::Min(15),
                    Constraint::Fill(3),
                    Constraint::Fill(3),
                ],
                ActiveBlock::Center => [
                    Constraint::Length(7),
                    Constraint::Fill(6),
                    Constraint::Fill(2),
                ],
                ActiveBlock::Details => [
                    Constraint::Length(7),
                    Constraint::Fill(3),
                    Constraint::Fill(5),
                ],
            }
        })
        .split(inner_main_area);

    sidebar::draw(frame, center[0], app);

    content::draw(frame, center[1], app);
}
