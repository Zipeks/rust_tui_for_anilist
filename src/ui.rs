use std::vec;

use crate::app::{App, CurrentScreen};
use ratatui::Frame;
use ratatui::layout::Direction;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap};

pub fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(15), Constraint::Length(1)])
        .split(frame.area());

    if let Some(status) = &app.status {
        let status_paragraph = Paragraph::new(Span::from(status)).centered();
        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(status_paragraph,area);
        return;
    }

    // let main_block = Block::default()
    //     .title(
    //         Line::from({
    //             let active_style = Style::default().fg(Color::DarkGray);
    //             let active_style = Style::default();
    //
    //             let mut home = Span::styled(" Home", Style::default().fg(Color::White));
    //             let mut search = Span::styled("Search", Style::default().fg(Color::White));
    //             let mut profile = Span::styled("Profile ", Style::default().fg(Color::White));
    //
    //             match app.current_screen {
    //                 CurrentScreen::Main => home = home.style(active_style),
    //                 CurrentScreen::Search => search = search.style(active_style),
    //                 CurrentScreen::Profile => profile = profile.style(active_style),
    //                 _ => {}
    //             }
    //             vec![home, Span::raw(" - "), search, Span::raw(" - "), profile]
    //         })
    //             .centered(),
    //     )
    //     .border_style(Style::default().fg(Color::Magenta))
    //     .border_type(BorderType::Rounded)
    //     .borders(Borders::ALL);

    let inactive_style = Style::default().fg(Color::DarkGray);
    let active_style = Style::default().fg(Color::White);

    let mut home = Span::styled(" Home ", inactive_style);
    let mut search = Span::styled(" Search ", inactive_style);
    let mut profile = Span::styled(" Profile ", inactive_style);

    match app.current_screen {
        CurrentScreen::Main => home = home.style(active_style),
        CurrentScreen::Search => search = search.style(active_style),
        CurrentScreen::Profile => profile = profile.style(active_style),
        _ => {}
    }

    let main_block = Block::default()
        .title(
            Line::from(vec![
                home,
                Span::raw("|"),
                search,
                Span::raw("|"),
                profile
            ]).centered()
        )
        .border_style(Style::default().fg(Color::Magenta))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL);


    let inner_area = main_block.inner(chunks[0]);

    frame.render_widget(main_block, chunks[0]);

    match app.current_screen {
        CurrentScreen::Main => {
            let text = if let Some(ref user) = app.user {
                format!("Witaj {}!", user.get_name())
            } else {
                "Ekran domowy. Brak zalogowanego użytkownika.".to_string()
            };

            let p = Paragraph::new(text);
            frame.render_widget(p, inner_area);
        }
        CurrentScreen::Search => {
            let p = Paragraph::new("Tutaj będzie wyszukiwarka...");
            frame.render_widget(p, inner_area);
        }
        CurrentScreen::Profile => {
            let p = Paragraph::new("Ustawienia profilu...");
            frame.render_widget(p, inner_area);
        }
        _ => {}
    }


    let current_keys_hint = {
        Line::from(Span::raw("Keybinds: ? "))
    };

    let footer = Block::new().borders(Borders::NONE);

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(chunks[1]);

    frame.render_widget(current_keys_hint, footer_chunks[0]);

    // if let Some(editing) = &app.currently_editing {
    //     let popup_block = Block::default()
    //         .title("Enter a new key-value pair.")
    //         .borders(Borders::NONE)
    //         .style(Style::default().bg(Color::DarkGray));
    //     let area = centered_rect(60, 25, frame.area());
    //     frame.render_widget(popup_block, area);
    //
    //     let popup_chunks = Layout::default()
    //         .direction(Direction::Horizontal)
    //         .margin(1)
    //         .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    //         .split(area);
    //
    //     let mut key_block = Block::default().title("Key").borders(Borders::all());
    //     let mut value_block = Block::default().title("Value").borders(Borders::all());
    //
    //     let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);
    //
    //     match editing {
    //         CurrentlyEditing::Key => key_block = key_block.style(active_style),
    //         CurrentlyEditing::Value => value_block = value_block.style(active_style),
    //     }
    //
    //     let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
    //     frame.render_widget(key_text, popup_chunks[0]);
    //
    //     let value_text = Paragraph::new(app.value_input.clone()).block(value_block);
    //     frame.render_widget(value_text, popup_chunks[1]);
    // }

    if let CurrentScreen::Exiting = app.current_screen {
        frame.render_widget(Clear, frame.area());

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta));

        let exit_text = Text::styled(
            "Would you like to exit?\n Y\\N",
            Style::default().fg(Color::Red),
        );

        let exit_paragraph = Paragraph::new(exit_text)
            .centered()
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(exit_paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
