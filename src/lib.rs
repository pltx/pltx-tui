use color_eyre::{eyre::WrapErr, Result};
use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::Frame;

pub mod config;
pub mod errors;
pub mod tui;
pub mod ui;

use ui::render;

use crate::config::get_config;

pub struct App {
    exit: bool,
    config: Config,
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
        render(frame, &self.config);
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
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true
    }
}
