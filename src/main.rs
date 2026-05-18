use crate::anilist::AnilistClient;
use std::error::Error;

use crate::app::{App, run_app};
use dotenv::dotenv;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::{env, io};

mod anilist;
mod app;
mod app_helper_structs;
mod keybinds;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal: Terminal<CrosstermBackend<io::Stderr>> = Terminal::new(backend)?;

    let mut app = App::new();

    dotenv().ok();
    let anilist_token = env::var("ANILIST_TOKEN").ok();
    let client = AnilistClient::new(anilist_token.as_deref())?;

    let (tx, rx) = std::sync::mpsc::channel();

    let _res = run_app(&mut terminal, &mut app, client, tx, &rx);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
