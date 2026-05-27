use crate::{
    app::App,
    app_helper_structs::{ActiveBlock, MediaType},
};
use ratatui::{prelude::*, widgets::*};
use ratatui_image::StatefulImage;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_active = app.active_block == ActiveBlock::Details;

    let details_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .border_type(BorderType::Rounded)
        .title(" Details ")
        .padding(Padding::proportional(1));

    let inner_details_area = details_block.inner(area);
    frame.render_widget(details_block, area);

    if let Some(ref err) = app.error_message {
        let p = Paragraph::new(format!("❌ API error: {}", err))
            .style(Style::default().fg(Color::Red))
            .centered();
        frame.render_widget(p, inner_details_area);
        return;
    } else if let Some(media_details) = &app.media_details {
        let media_type = media_details.type_;
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(14),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(inner_details_area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .spacing(2)
            .constraints([Constraint::Length(25), Constraint::Fill(1)])
            .split(vertical_chunks[0]);

        let image_area = top_chunks[0];
        let media_id = app
            .browse_state
            .state
            .selected()
            .and_then(|idx| app.get_current_center_items().get(idx))
            .map(|item| item.id)
            .unwrap_or(0);

        if let Some(image_protocol) = app.image_cache.get_mut(&media_id) {
            let image_widget = StatefulImage::default();
            frame.render_stateful_widget(image_widget, image_area, image_protocol);
        } else if app.currently_fetching_image == Some(media_id) {
            frame.render_widget(Paragraph::new("⏳").centered(), image_area);
        } else {
            frame.render_widget(Paragraph::new("").centered(), image_area);
        }

        let right_panel_area = top_chunks[1];

        let right_panel_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Fill(1)])
            .split(right_panel_area);

        let main_title = media_details.titles.get_title(&app.title_language);
        let mut alt_titles = Vec::new();
        let candidates = [
            &media_details.titles.native,
            &media_details.titles.romaji,
            &media_details.titles.english,
        ];
        for t in candidates.iter() {
            let t_str = t.as_str();
            if !t_str.is_empty() && t_str != main_title && !alt_titles.contains(&t_str) {
                alt_titles.push(t_str);
            }
        }
        let mut title_spans = Vec::new();

        if media_details.is_favourite {
            title_spans.push(Span::styled("❤️ ", Style::default().fg(Color::Red)));
            title_spans.push(Span::styled("• ", Style::default().fg(Color::DarkGray))); 
        }

        title_spans.push(Span::styled(
            main_title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

        let title_lines = vec![
            Line::from(title_spans), 
            Line::from(Span::styled(
                alt_titles.join(" • "),
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let titles_paragraph = Paragraph::new(title_lines).wrap(Wrap { trim: true });
        frame.render_widget(titles_paragraph, right_panel_chunks[0]);

        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(right_panel_chunks[1]);

        let label_style = Style::default().fg(Color::DarkGray);
        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let total_str = media_details
            .total
            .map(|t| t.to_string())
            .unwrap_or_else(|| "?".to_string());
        let info_lines = vec![
            Line::from("Information").style(header_style),
            Line::from(vec![
                Span::styled("Status:     ", label_style),
                Span::raw(media_details.media_status.to_string()),
            ]),
            // Line::from(vec![
            //     Span::styled("Season:    ", label_style),
            //     Span::raw(format!(
            //         "{} {}",
            //         media_details.season.to_string(),
            //         media_details.season_year
            //     )),
            // ]),
            Line::from(vec![
                Span::styled(
                    {
                        let media_type: MediaType = media_details.type_;
                        match media_type {
                            MediaType::Anime => "Episodes:   ",
                            MediaType::Manga => "Chapters:   ",
                            MediaType::Unknown => " ",
                        }
                    },
                    label_style,
                ),
                Span::raw(total_str),
            ]),
            Line::from(vec![
                Span::styled("Avg Score:  ", label_style),
                Span::raw(format!("{} / 100", media_details.average_score)),
            ]),
            Line::from(vec![
                Span::styled("Start Date: ", label_style),
                Span::raw(media_details.start_date.to_string()),
            ]),
            Line::from(vec![
                Span::styled("End Date:   ", label_style),
                Span::raw(media_details.end_date.to_string()),
            ]),
        ];
        frame.render_widget(Paragraph::new(info_lines), stats_chunks[0]);

        if let Some(user_media_details) = &media_details.user_media_details {
            let user_info_lines = vec![
                Line::from("Your List").style(header_style),
                Line::from(vec![
                    Span::styled("Status:   ", label_style),
                    Span::raw(user_media_details.status.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Progress: ", label_style),
                    Span::raw(user_media_details.progress.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Score:    ", label_style),
                    Span::raw(user_media_details.score.to_string()),
                ]),
            ];
            frame.render_widget(Paragraph::new(user_info_lines), stats_chunks[1]);
        }

        let desc_area = vertical_chunks[2];

        let clean_desc = media_details
            .description
            .replace("<br>\n", "\n")
            .replace("<br>", "\n")
            .replace("<i>", "")
            .replace("</i>", "")
            .replace("<b>", "")
            .replace("</b>", "");

        let description: Vec<Line> = clean_desc.lines().map(|t| Line::from(t)).collect();

        let desc_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Description ");

        let p = Paragraph::new(description)
            .block(desc_block)
            .scroll((0, 0))
            .wrap(Wrap { trim: false });

        frame.render_widget(p, desc_area);
    }
}
