use crate::app::{App, ActiveBlock};

use ratatui::{prelude::*, widgets::*};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let keybinds_info = Paragraph::new(Line::from({
        let keybinds = match app.active_block {
            ActiveBlock::Sidebar => vec![
                Span::raw("  "),
                Span::raw("Up: k"),
                Span::raw(" | "),
                Span::raw("Down: j"),
                Span::raw(" | "),
                Span::raw("Right: l"),
            ],
            ActiveBlock::Center => vec![
                Span::raw("  "),
                Span::raw("Up: k"),
                Span::raw(" | "),
                Span::raw("Down: j"),
                Span::raw(" | "),
                Span::raw("Sidebar: h"),
                Span::raw(" | "),
                Span::raw("Details: l"),
                Span::raw(" | "),
                Span::raw("Change anime/manga: ]"),
            ],
            _ => vec![],
        };
        keybinds
    }      
        )).left_aligned();

    frame.render_widget(keybinds_info, area);

}
