use color_eyre::{eyre::WrapErr, Result};
use config::Config;
use crossterm::event::{self, Event, KeyEventKind};
use database::database_connection;
use keybinds::handle_key_event;
use popups::Help;
use ratatui::{layout::Rect, style::Color, Frame};

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

use rusqlite::Connection;
use state::{Mode, Popup, Screen, State, Window};
use ui::render;
use utils::PopupKeyEventHandler;

use crate::{config::get_config, utils::RenderScreen};

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
                (Screen::Dashboard, "Dashboard", &screens::Dashboard::render),
                (
                    Screen::ProjectManagement,
                    "Project Management",
                    &screens::ProjectManagement::render,
                ),
                (Screen::Sleep, "Sleep", &screens::Sleep::render),
                (Screen::Settings, "Settings", &screens::Settings::render),
            ],
            state: State {
                mode: Mode::Navigation,
                screen: Screen::Dashboard,
                window: Window::Navigation,
                popup: Popup::None,
            },
        }
    }

    /// Runs the application's main loop until the user quits.
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

    /// Updates the application's state based on user input.
    fn handle_events(&mut self) -> Result<()> {
        let e = event::read()?;
        // A copy of the application state at the start of the event. Since there are
        // seperate global and component-specific functions that handle events,
        // they can conflict if not using the original state of the application
        // at the start of the event. For example, if the global handler runs
        // first and opens the help menu when pressing `?`, but the help popup
        // event handler sees that the current mode state is set to `Mode::Popup`,
        // then it will immediately close the popup when the user presses `?`, thus the
        // popup will never open.
        let event_state = &self.state.clone();
        // Pass the event for the popup to handle
        // if self.state.mode == Mode::Popup {
        //     match self.state.popup {
        //         _ => {}
        //     }
        // }
        match e {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if self.state.mode == Mode::Popup {
                    match self.state.popup {
                        Popup::Help => Help::key_event_handler(self, key_event, event_state),
                        Popup::None => {}
                    }
                }
                handle_key_event(self, key_event, event_state)
                    .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
            }
            _ => Ok(()),
        }
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
}
