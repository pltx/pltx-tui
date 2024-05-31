use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use color_eyre::{eyre::Context, Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use pltx_app::{
    state::{AppModule, View},
    App, Module, Popup,
};

use crate::{command_handler::CommandHandler, ui::Interface};

pub enum Event {
    Tick,
    Key(KeyEvent),
    // Mouse(MouseEvent),
    // Resize(u16, u16),
    FocusGained,
    FocusLost,
    // Paste(String),
}

pub struct EventHandler {
    // sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    // handler: thread::JoinHandle<()>,
}

const TICK_RATE: u64 = 500;

impl EventHandler {
    pub fn init() -> Self {
        let tick_rate = Duration::from_millis(TICK_RATE);
        let (sender, receiver) = mpsc::channel();

        // handler
        {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("no events available") {
                        match event::read().expect("enable to read event") {
                            CrosstermEvent::Key(e) => {
                                sender
                                    .send(Event::Key(e))
                                    .expect("failed to send key event");
                            }
                            // CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                            // CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                            CrosstermEvent::FocusGained => {
                                sender
                                    .send(Event::FocusGained)
                                    .expect("failed to send focus gained event");
                            }
                            CrosstermEvent::FocusLost => {
                                sender
                                    .send(Event::FocusLost)
                                    .expect("failed to send focus lost event");
                            }
                            // CrosstermEvent::Paste(s) => sender.send(Event::Paste(s)),
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            // sender,
            receiver,
            // handler,
        }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    pub fn key_events(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        command_handler: &mut CommandHandler,
        key_event: KeyEvent,
    ) -> Result<()> {
        match key_event.kind {
            KeyEventKind::Press => self
                .key_event_handler(app, interface, command_handler, key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn key_event_handler(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        command_handler: &mut CommandHandler,
        key_event: KeyEvent,
    ) -> Result<()> {
        if app.mode.is_normal() {
            match key_event.code {
                KeyCode::Char('`') => app.debug.toggle(),
                KeyCode::Char('!') => app.debug.toggle_min_preview(),
                KeyCode::Char('~') => app.debug.next_position(),
                _ => {}
            }
        }

        match app.view {
            View::Default => {
                if app.mode.is_normal() && key_event.code == KeyCode::Char(':') {
                    app.mode.insert();
                    app.view.command();
                }
            }
            View::Popup => {
                if app.mode.is_normal() && key_event.code == KeyCode::Char(':') {
                    app.mode.insert();
                    app.view.command();
                }
            }
            View::Command => {
                command_handler.key_event_handler(app, key_event);
                return Ok(());
            }
        }

        match app.module {
            AppModule::Home => interface.modules.home.key_event_handler(app, key_event),
            AppModule::ProjectManagement => interface
                .modules
                .project_management
                .key_event_handler(app, key_event)?,
            _ => {}
        }

        Ok(())
    }
}
