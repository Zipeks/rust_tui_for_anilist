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
use tracing::info;
use tracing_subscriber::EnvFilter;
mod anilist;
mod app;
mod app_helper_structs;
mod keybinds;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _guard = init_tracing();

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
fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("logs", "anilist_tui.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    guard
}
