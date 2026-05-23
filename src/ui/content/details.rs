use std::vec;

use crate::{
    app::App,
    app_helper_structs::{ActiveBlock, BrowseCategory, CurrentView, MediaDetails},
    ui::content::draw_media_list,
};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};
use ratatui_image::StatefulImage;
// use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_active = app.active_block == ActiveBlock::Details;

    let details_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .border_type(BorderType::Rounded);

    let inner_details_area = details_block.inner(area);
    frame.render_widget(details_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Fill(1)])
        .split(inner_details_area);

    if let Some(ref err) = app.error_message {
        let p = Paragraph::new(format!("❌ API error: {}", err))
            .style(Style::default().fg(Color::Red))
            .centered();
        frame.render_widget(p, chunks[1]);
        return;
    } else if let Some(media_details) = &app.media_details {
        let media_id = {
            let mut id = 0;
            if let Some(idx) = app.browse_state.state.selected() {
                let items = app.get_current_center_items();
                if idx < items.len() {
                    id = items[idx].id;
                }
            }
            id
        };

        let title = Line::from(media_details.titles.get_title(&app.title_language)).style(Style::default().bold()).centered();

        frame.render_widget(title, chunks[0]);

        let clean_desc = media_details
            .description
            .replace("<br>\n", "")
            .replace("<br>", "")
            .replace("<i>", "")
            .replace("</i>", "")
            .replace("<b>", "")
            .replace("</b>", "");
        let description: Vec<Line> = clean_desc.lines().map(|t| Line::from(t)).collect();

        let p = Paragraph::new(description)
            .scroll((0, 0))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, chunks[1]);
    }
}
