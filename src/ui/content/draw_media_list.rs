use crate::app_helper_structs::{CurrentView, MediaListItem, TitleLanguage};
use ratatui::{prelude::*, widgets::*};

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    items: &[MediaListItem],
    is_active: bool,
    state: &mut TableState,
    title_spans: Vec<Span>,
    page_info: Span,
    current_view: CurrentView,
    title_language: &TitleLanguage,
) {
    let now = chrono::Utc::now().timestamp();

    let rows: Vec<Row> = items
        .iter()
        .map(|item| {
            let progress = item.progress.unwrap_or(0);
            let total = match item.total {
                None => "?".to_string(),
                Some(x) => x.to_string(),
            };
            let progress_str = match current_view {
                CurrentView::UserAnime | CurrentView::UserManga => {
                    format!("{}/{} ", progress, total)
                }
                CurrentView::BrowseAnime | CurrentView::BrowseManga => {
                    format!("{} ", {
                        match item.average_score {
                            Some(score) => score.to_string(),
                            _ => "".to_string(),
                        }
                    })
                }
            };

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
            let display_title = item.titles.get_title(&title_language).to_string();
            let airing_cell = Cell::from(airing_str).style(Style::default().fg(Color::Magenta));

            Row::new(vec![
                Cell::from(Span::styled(" ● ", Style::default().fg(Color::Cyan))),
                Cell::from(display_title),
                airing_cell,
                Cell::from(Line::from(progress_str).alignment(Alignment::Right))
                    .style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();
    let header_row = Row::new(vec![
        Cell::from(""),
        Cell::from("Title"),
        Cell::from(match current_view {
            CurrentView::UserAnime | CurrentView::BrowseAnime => "Next Episode",
            CurrentView::UserManga | CurrentView::BrowseManga => "",
        }),
        Cell::from(
            Line::from(match current_view {
                CurrentView::UserAnime | CurrentView::UserManga => "Progress ",
                CurrentView::BrowseAnime => "Avg score ",
                CurrentView::BrowseManga => "Avg score ",
            })
            .right_aligned(),
        ),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let table_widget = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(30),
            Constraint::Max(21),
            Constraint::Length(12),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if is_active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .border_type(BorderType::Rounded)
            .title(Line::from(title_spans).centered())
            .title_bottom(Line::from(page_info).centered()),
    )
    // .highlight_symbol(">> ")
    .row_highlight_style(Style::default().yellow());

    frame.render_stateful_widget(table_widget, area, state);
}
