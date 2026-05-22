use crate::{app::App, ui::centered_rect};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let popup_area = centered_rect(30, 6, frame.area());

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title(" Titles lang ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let items: Vec<ListItem> = crate::app_helper_structs::TitleLanguage::ALL
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            if i == app.language_popup_index {
                ListItem::new(format!(" > {} ", lang.to_string())).style(
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(format!("   {} ", lang.to_string()))
                    .style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let list = List::new(items).block(popup_block);
    frame.render_widget(list, popup_area);
}
