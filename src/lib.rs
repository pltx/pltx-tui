use color_eyre::{eyre::WrapErr, Result};
use ratatui::style::Color;
use rusqlite::Connection;
use tui_scrollview::ScrollViewState;

pub mod components;
pub mod popups;
pub mod screens;
pub mod utils;

pub mod config;
pub mod database;
pub mod errors;
pub mod keybinds;
pub mod state;
pub mod tui;
pub mod ui;

use config::Config;
use database::database_connection;
use keybinds::EventHandler;
use state::{Mode, Pane, Popup, Screen, State};
use ui::Interface;

use crate::config::get_config;

pub struct App {
    exit: bool,
    config: Config,
    db: Connection,
    state: State,
    scroll_view_state: ScrollViewState,
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
            state: State {
                mode: Mode::Navigation,
                screen: Screen::Dashboard,
                pane: Pane::Navigation,
                popup: Popup::None,
            },
            scroll_view_state: ScrollViewState::new(),
        }
    }

    /// Runs the application's main loop until the user quits.
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        let mut interface = Interface::init();
        let mut event_handler = EventHandler::init();

        while !self.exit {
            terminal.draw(|frame| {
                interface.render(frame, self);
            })?;

            event_handler
                .handle_events(self, &mut interface)
                .wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true
    }

    /// Returns (text, fg, bg).
    fn get_mode(&self) -> (&str, Color, Color) {
        let colors = &self.config.colors;
        match self.state.mode {
            Mode::Navigation => (
                "Navigation",
                colors.status_bar_navigation_mode_fg,
                colors.status_bar_navigation_mode_bg,
            ),
            Mode::Popup => (
                "Popup",
                colors.status_bar_popup_mode_fg,
                colors.status_bar_popup_mode_bg,
            ),
        }
    }

    pub fn get_screen_list<'a>(&self) -> Vec<(Screen, &'a str)> {
        vec![
            (Screen::Dashboard, "Dashboard"),
            (Screen::ProjectManagement, "Project Management"),
            (Screen::Sleep, "Sleep"),
            (Screen::Settings, "Settings"),
        ]
    }
}
