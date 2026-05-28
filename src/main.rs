use crate::anilist::AnilistClient;
use crate::app::{App, run_app};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::error::Error;
use std::io;
use std::ops::Deref;
use tracing_subscriber::EnvFilter;

mod anilist;
mod app;
mod app_helper_structs;
mod auth;
mod keybinds;
mod ui;
mod utils;

const CLIENT_ID: &str = "40678";
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _guard = init_tracing();

    let token = auth::load_user_token();

    let anilist_token = match token {
        Some(t) => t,
        None => {
            println!("Authorization token not found.");
            println!("Logging in with your browser...");

            let new_token = auth::login_with_browser(CLIENT_ID).await;

            match new_token {
                Ok(s) => {
                    let _ = auth::save_user_token(&s);
                    s
                }
                Err(e) => {
                    println!("Something went wrong during authorization: {}", e);
                    return Ok(());
                }
            }
        }
    };

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal: Terminal<CrosstermBackend<io::Stderr>> = Terminal::new(backend)?;

    let mut app = App::new();

    let client = AnilistClient::new(Some(anilist_token.deref()))?;

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
