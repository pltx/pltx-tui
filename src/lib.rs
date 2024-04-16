use color_eyre::{eyre::WrapErr, Result};
use config::Config;
use crossterm::event::{self, Event, KeyEventKind};
use database::database_connection;
use keybinds::handle_key_event;
use ratatui::{layout::Rect, Frame};

pub mod config;
pub mod database;
pub mod errors;
pub mod keybinds;
pub mod screens;
pub mod state;
pub mod tui;
pub mod ui;

use rusqlite::Connection;
use state::{Screen, Mode, State, Window};
use ui::render;

use crate::config::get_config;

type ScreenRenderFn = &'static dyn Fn(&mut Frame, &App, Rect);

pub struct App {
    exit: bool,
    config: Config,
    db: Connection,
    screen_list: Vec<(Screen, &'static str, ScreenRenderFn)>,
    state: State,
}

impl Default for App {
    fn default() -> App {
        App::new()
    }
}

impl App {
    // Create a new instance App
    pub fn new() -> App {
        App {
            exit: false,
            config: get_config(),
            db: database_connection(),
            screen_list: vec![
                (
                    Screen::Dashboard,
                    "Dashboard",
                    &screens::dashboard::render_dashboard,
                ),
                (Screen::Sleep, "Sleep", &screens::sleep::render_sleep),
                (
                    Screen::Settings,
                    "Settings",
                    &screens::settings::render_settings,
                ),
            ],
            state: State {
                mode: Mode::Navigation,
                screen: Screen::Dashboard,
                window: Window::Navigation,
            },
        }
    }

    /// Runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        render(frame, self);
    }

    /// Updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                handle_key_event(self, key_event)
                    .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
            }
            _ => Ok(()),
        }
    }

    fn exit(&mut self) {
        self.exit = true
    }

    fn get_mode(&self) -> &str {
        match self.state.mode {
            Mode::Navigation => "Navigation",
        }
    }
}
