use crate::app::{ActiveBlock, App};
use ratatui::{prelude::*, widgets::*};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let sidebar_block = Block::default();

    let inner_sidebar_area = sidebar_block.inner(area);
    frame.render_widget(sidebar_block, area);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(inner_sidebar_area);

    let items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|view| {
            // let text = if *view == app.current_view {
            //     format!("● {}", view.to_string())
            // } else {
            //     format!("● {}", view.to_string())
            // };
            let text = Line::from(vec![
                Span::styled("●", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(" {}", view.to_string()),
                    Style::default().fg(Color::White),
                ),
            ]);

            ListItem::new(text).style(Style::default().fg(Color::Cyan))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(match app.active_block {
                    ActiveBlock::Sidebar => Style::default().fg(Color::Cyan),
                    _ => Style::default().fg(Color::DarkGray),
                })
                .border_type(BorderType::Rounded),
        )
        .highlight_symbol("> ".red())
        .highlight_style(Style::default().yellow())
        .repeat_highlight_symbol(true)
        .scroll_padding(1);

    frame.render_stateful_widget(list, sidebar_layout[0], &mut app.sidebar_state);
}
