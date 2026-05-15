use crate::{
    app::App,
    app_helper_structs::{ActiveBlock, BrowseCategory, CurrentView, MediaTab},
    ui::content::draw_media_list,
};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    if app.is_loading {
        let p = Paragraph::new("⏳ Waiting for AniList...").centered();
        frame.render_widget(p, area);
        return;
    }
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

    let (media_items, active_state, title_spans) = match app.current_view {
        
        CurrentView::Home => {
            let (items, state) = match app.active_tab {
                MediaTab::Anime => (
                    app.user_anime.as_ref().and_then(|l| l.items.as_deref()).unwrap_or(&[]),
                    &mut app.user_anime_state,
                ),
                MediaTab::Manga => (
                    app.user_manga.as_ref().and_then(|l| l.items.as_deref()).unwrap_or(&[]),
                    &mut app.user_manga_state,
                ),
            };

            let (a_s, m_s) = if app.active_tab == MediaTab::Anime {
                (active_style, inactive_style)
            } else {
                (inactive_style, active_style)
            };

            let spans = vec![
                Span::styled(" Anime ", a_s),
                Span::raw("│"),
                Span::styled(" Manga ", m_s),
            ];

            (items, state, spans)
        }

        CurrentView::BrowseAnime | CurrentView::BrowseManga => {
            let (items, state, current_category) = if app.current_view == CurrentView::BrowseAnime {
                let i = app.browse_anime.media.as_ref().and_then(|l| l.items.as_deref()).unwrap_or(&[]);
                (i, &mut app.browse_anime.state, app.browse_anime.current_category)
            } else {
                let i = app.browse_manga.media.as_ref().and_then(|l| l.items.as_deref()).unwrap_or(&[]);
                (i, &mut app.browse_manga.state, app.browse_manga.current_category)
            };

            let categories = [
                BrowseCategory::Trending,
                BrowseCategory::ThisSeason,
                BrowseCategory::NextSeason,
                BrowseCategory::SearchResults,
            ];

            let mut spans = Vec::new();
            for (i, cat) in categories.iter().enumerate() {
                let style = if *cat == current_category { active_style } else { inactive_style };
                spans.push(Span::styled(format!(" {} ", cat.to_string()), style));
                
                if i < categories.len() - 1 {
                    spans.push(Span::raw("│")); 
                }
            }

            (items, state, spans)
        }
        _ => return, 
    };


    draw_media_list::draw(
        frame,
        chunks[0],
        media_items,
        is_center_active,
        active_state,
        title_spans,
    );
}
