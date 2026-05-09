use crate::anilist::AnilistClient;
use std::error::Error;

use crate::app::{App, run_app, CurrentScreen};
use dotenv::dotenv;
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::{EnableMouseCapture, Event, KeyCode};
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use ratatui::crossterm::{event, execute};
use std::{env, io};

mod anilist;
mod app;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal: Terminal<CrosstermBackend<io::Stderr>> = Terminal::new(backend)?;

    let mut app = App::new();

    dotenv().ok();
    let anilist_token = env::var("ANILIST_TOKEN").ok();
    let client = AnilistClient::new(anilist_token.as_deref())?;

    let viewer_result = client.get_basic_viewer().await;

    if let Ok(viewer_data) = viewer_result {
        if let Some(viewer) = viewer_data.viewer {
            if let Some(options) = viewer.options {
                app.authenticated(viewer.name, options.display_adult_content);
            }
        }
    } else {
        app.status = Some(String::from("Login error"));
        app.current_screen = CurrentScreen::Status;
    }

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
