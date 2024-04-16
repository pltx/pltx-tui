use color_eyre::{eyre::WrapErr, Result};
use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{layout::Rect, Frame};

pub mod config;
pub mod errors;
pub mod screens;
pub mod state;
pub mod tui;
pub mod ui;

use ui::render;

use crate::config::get_config;
use state::{Mode, Screen, State};

type ScreenRenderFn = &'static dyn Fn(&mut Frame, &App, Rect);

pub struct App {
    exit: bool,
    config: Config,
    state: State,
    screen_list: Vec<(Screen, &'static str, ScreenRenderFn)>,
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
            state: State {
                mode: Mode::Navigation,
                screen: Screen::Dashboard,
            },
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
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    // Makes keybinds by handing key events
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let screen_index = self
            .screen_list
            .iter()
            .position(|s| s.0 == self.state.screen)
            .unwrap();

        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            KeyCode::Char('j') => {
                if self.state.mode == Mode::Navigation {
                    if screen_index == self.screen_list.len() - 1 {
                        self.state.screen = self.screen_list[0].0.clone();
                    } else {
                        self.state.screen = self.screen_list[screen_index + 1].0.clone();
                    }
                };
            }
            KeyCode::Char('k') => {
                if self.state.mode == Mode::Navigation {
                    if screen_index == 0 {
                        self.state.screen = self.screen_list[self.screen_list.len() - 1].0.clone();
                    } else {
                        self.state.screen = self.screen_list[screen_index - 1].0.clone();
                    }
                };
            }
            _ => {}
        }
        Ok(())
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
