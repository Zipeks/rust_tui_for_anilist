use crate::anilist::get_anime::MediaSeason;
use crate::app::MediaType::Anime;
use crate::app::Season::ANY;
use chrono::{Datelike, Utc};

use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::{EnableMouseCapture, Event, KeyCode};
use ratatui::crossterm::{event, execute};
use std::io;
use crate::ui::ui;
pub enum CurrentScreen {
    Main,
    Profile,
    Search,
    Media,
    Exiting,
    Status
}
pub struct User {
    name: String,
    allows_nsfw: Option<bool>,
}
impl User {
    pub fn get_name(&self) -> &String {
        &self.name
    }
}
pub struct App {
    pub current_screen: CurrentScreen,
    pub search_settings: SearchSettings,
    pub previous_state: Box<Option<App>>,
    pub user: Option<User>,
    pub status: Option<String>
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            search_settings: SearchSettings {
                search_input: String::from(""),
                media_year: Utc::now().year(),
                media_season: ANY,
                media_type: Anime,

            },
            previous_state: Box::new(None),
            user: None,
            status: None
        }
    }
    pub fn authenticated(&mut self, name: String, allows_nsfw: Option<bool>) {
        self.user = Some(User { name, allows_nsfw })
    }
}
pub enum MediaType {
    Anime,
    Manga,
}
pub enum Season {
    WINTER,
    SPRING,
    SUMMER,
    FALL,
    ANY,
}
pub struct SearchSettings {
    search_input: String,
    media_type: MediaType,
    media_year: i32,
    media_season: Season,
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool>
where
    io::Error: From<B::Error>,
{
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    // KeyCode::Char('e') => {
                    //     // app.current_screen = CurrentScreen::Editing;
                    //     // app.currently_editing = Some(CurrentlyEditing::Key);
                    // }
                    // // KeyCode::Char('q') => {
                    //     app.current_screen = CurrentScreen::Exiting;
                    // }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('q') | KeyCode::Enter => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },

                CurrentScreen::Status => {
                    match key.code {
                        _ => return Ok(true)
                    }   
                }
                _ => {}
            }

            // Universal keybinds
            match key.code {
                KeyCode::Char(']') => match app.current_screen {
                    CurrentScreen::Main => app.current_screen = CurrentScreen::Search,
                    CurrentScreen::Search => app.current_screen = CurrentScreen::Profile,
                    CurrentScreen::Profile => app.current_screen = CurrentScreen::Main,
                    _ => {}
                },
                KeyCode::Char('[') => match app.current_screen {
                    CurrentScreen::Main => app.current_screen = CurrentScreen::Profile,
                    CurrentScreen::Search => app.current_screen = CurrentScreen::Main,
                    CurrentScreen::Profile => app.current_screen = CurrentScreen::Search,
                    _ => {}
                },
                KeyCode::Char('q') => {
                    app.current_screen = CurrentScreen::Exiting;
                }
                _ => {}
            }
        }
    }
}
