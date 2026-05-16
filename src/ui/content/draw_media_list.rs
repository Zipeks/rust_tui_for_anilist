use crate::app_helper_structs::MediaListItem;
use ratatui::{prelude::*, widgets::*};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    items: &[MediaListItem],
    is_active: bool,
    state: &mut TableState,
    title_spans: Vec<Span>,
) {
    let now = chrono::Utc::now().timestamp();

    let rows: Vec<Row> = items
        .iter()
        .map(|item| {
            let progress = item.progress.unwrap_or(0);
            let total = item
                .total
                .map(|t| t.to_string())
                .unwrap_or_else(|| "?".to_string());
            let progress_str = format!("{}/{} ", progress, total);

            let airing_str = if let Some(ref airing) = item.next_airing_episode {
                let diff_seconds = airing.airing_at - now;

                if diff_seconds > 0 {
                    let days = diff_seconds / 86400;
                    let hours = (diff_seconds % 86400) / 3600;
                    let mins = (diff_seconds % 3600) / 60;

                    if days > 0 {
                        format!("Ep {}: {}d {}h {}min", airing.episode, days, hours, mins)
                    } else {
                        format!("Ep {}: in {}h {}min", airing.episode, hours, mins)
                    }
                } else {
                    format!("Ep {} is out! ", airing.episode)
                }
            } else {
                String::new()
            };

            let airing_cell = Cell::from(airing_str).style(Style::default().fg(Color::Magenta));

            Row::new(vec![
                Cell::from(Span::styled(" ● ", Style::default().fg(Color::Cyan))),
                Cell::from(item.title.clone()),
                airing_cell,
                Cell::from(Line::from(progress_str).alignment(Alignment::Right))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let table_widget = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(20),
            Constraint::Length(21),
            Constraint::Length(11),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if is_active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .border_type(BorderType::Rounded)
            .title(Line::from(title_spans).centered()),
    )
    .highlight_symbol(">> ")
    .row_highlight_style(Style::default().yellow());

    frame.render_stateful_widget(table_widget, area, state);
}
