use ratatui::{prelude::*, widgets::*};

use crate::app::MediaTab;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    items: &[String],
    is_active: bool,
    state: &mut ListState,
    active_media: MediaTab,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|title| {
            ListItem::new(Line::from(vec![
                Span::styled(" ● ", Style::default().fg(Color::Cyan)),
                Span::raw(title.clone()),
            ]))
        })
        .collect();

    let active_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    let list_widget = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if is_active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                }).border_type(BorderType::Rounded)
                .title(
                    Line::from({
                        let (anime_style, manga_style) = match active_media {
                            MediaTab::Anime => (active_style, inactive_style),
                            MediaTab::Manga => (inactive_style, active_style),
                        };
                        vec![
                            Span::styled(" Anime ", anime_style),
                            Span::raw("│"),
                            Span::styled(" Manga ", manga_style),
                        ]
                    })
                    .centered(),
                ),
        )
        .highlight_symbol(">> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list_widget, area, state);
}
