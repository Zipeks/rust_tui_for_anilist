use crate::{
    app::App,
    app_helper_structs::{ActiveBlock, BrowseCategory, CurrentView},
    ui::content::draw_media_list,
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    // if app.is_loading {
    //     let p = Paragraph::new("⏳ Waiting for AniList...").centered();
    //     frame.render_widget(p, area);
    //     return;
    // }
    if let Some(ref err) = app.error_message {
        let p = Paragraph::new(format!("❌ API error: {}", err))
            .style(Style::default().fg(Color::Red))
            .centered();
        frame.render_widget(p, area);
        return;
    }

    let is_center_active = app.active_block == ActiveBlock::Center;

    let active_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    let (media_items, active_state, current_category) = {
        let i = app
            .browse_state
            .media
            .as_ref()
            .and_then(|l| l.items.as_deref())
            .unwrap_or(&[]);
        (
            i,
            &mut app.browse_state.state,
            app.browse_state.current_category,
        )
    };

    let categories = BrowseCategory::ALL;

    let mut title_spans = Vec::new();
    for (i, cat) in categories.iter().enumerate() {
        let style = if *cat == current_category {
            active_style
        } else {
            inactive_style
        };
        title_spans.push(Span::styled(
            format!(" {} ", {
                match app.current_view {
                    CurrentView::UserAnime => cat.to_string_user_anime(),
                    CurrentView::UserManga => cat.to_string_user_manga(),
                    CurrentView::BrowseAnime => cat.to_string_browse_anime(),
                    CurrentView::BrowseManga => cat.to_string_browse_manga(),
                }
            }),
            style,
        ));

        if i < categories.len() - 1 {
            title_spans.push(Span::raw("│"));
        }
    }
    let page_text = app
        .browse_state
        .media
        .as_ref()
        .map_or("1".to_string(), |media| {
            let current = media.page_info.current_page;
            // Api doesn't give accurate info about number of pages

            // let last = media
            //     .page_info
            //     .last_page
            //     .map(|p| p.to_string())
            //     .unwrap_or_else(|| "?".to_string());

            format!("{}", current)
        });

    let page_info = Span::raw(page_text);

    draw_media_list::draw(
        frame,
        chunks[0],
        media_items,
        is_center_active,
        active_state,
        title_spans,
        page_info,
        app.current_view,
        &app.title_language,
    );
}
