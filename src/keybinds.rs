use color_eyre::eyre::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use pltx_app::{
    state::{GlobalPopup, Mode, Pane, Screen, State},
    App,
};
use pltx_utils::KeyEventHandler;

use crate::{command_handler::CommandHandler, ui::Interface};

/// Global event handler
pub struct EventHandler;

impl EventHandler {
    pub fn init() -> EventHandler {
        EventHandler {}
    }

    /// Updates the application's state based on user input.
    pub fn handle_events(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        command_handler: &mut CommandHandler,
    ) -> color_eyre::eyre::Result<()> {
        let e = event::read()?;
        // A copy of the application state at the start of the event. Since there are
        // seperate global and widget-specific functions that handle events,
        // they can conflict if not using the original state of the application
        // at the start of the event. For example, if the global handler runs
        // first and opens the help menu when pressing `?`, but the help popup
        // event handler sees that the current mode state is set to `Mode::Popup`,
        // then it will immediately close the popup when the user presses `?`, thus the
        // popup will never open.
        let event_state = &app.state.clone();
        match e {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .key_event_handler(app, interface, command_handler, key_event, event_state)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    // Makes keybinds by handing key events
    fn key_event_handler(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        command_handler: &mut CommandHandler,
        key_event: KeyEvent,
        event_state: &State,
    ) -> color_eyre::eyre::Result<()> {
        let screen_list = &app.get_screen_list();
        let screen_index = screen_list
            .iter()
            .position(|s| s.0 == event_state.screen)
            .unwrap();

        // TODO: Convert to match statements when adding more poups
        match app.state.mode {
            Mode::Popup => {
                if app.state.popup == GlobalPopup::Help {
                    interface
                        .popups
                        .help
                        .key_event_handler(app, key_event, event_state);
                }
            }
            Mode::Command | Mode::CommandInsert => {
                command_handler.key_event_handler(app, key_event, event_state)
            }
            _ => {}
        }

        match event_state.mode {
            Mode::Navigation | Mode::Popup => {
                if key_event.code == KeyCode::Char(':') {
                    app.state.mode = Mode::CommandInsert;
                }
            }
            _ => {}
        }

        if event_state.mode == Mode::Navigation {
            match key_event.code {
                KeyCode::Char('?') => {
                    app.state.mode = Mode::Popup;
                    app.state.popup = GlobalPopup::Help;
                }
                KeyCode::Enter | KeyCode::Char('L') => {
                    if event_state.pane == Pane::Navigation {
                        app.state.pane = Pane::Screen;
                    }
                }
                _ => {}
            }
            if event_state.pane == Pane::Navigation {
                match key_event.code {
                    // Go down an option
                    KeyCode::Char('j') => {
                        if screen_index == screen_list.len().saturating_sub(1) {
                            app.state.screen = screen_list[0].0.clone();
                        } else {
                            app.state.screen = screen_list[screen_index + 1].0.clone();
                        }
                    }
                    // Go up an option
                    KeyCode::Char('k') => {
                        if screen_index == 0 {
                            app.state.screen =
                                screen_list[screen_list.len().saturating_sub(1)].0.clone();
                        } else {
                            app.state.screen =
                                screen_list[screen_index.saturating_sub(1)].0.clone();
                        }
                    }
                    _ => {}
                }
            }
        }
        if event_state.mode == Mode::Insert && key_event.code == KeyCode::Esc {
            app.state.mode = Mode::Navigation;
        } else if event_state.mode == Mode::PopupInsert && key_event.code == KeyCode::Esc {
            app.state.mode = Mode::Popup;
        }

        match app.state.screen {
            Screen::Dashboard => {
                interface
                    .screens
                    .dashboard
                    .key_event_handler(app, key_event, event_state)
            }
            Screen::ProjectManagement => {
                interface
                    .screens
                    .project_management
                    .key_event_handler(app, key_event, event_state)
            }
            _ => {}
        }

        if event_state.mode == Mode::Popup {
            // Global popup keybinds
            match key_event.code {
                // Close the popup
                KeyCode::Char('q') => {
                    app.state.mode = Mode::Navigation;
                    app.state.popup = GlobalPopup::None;
                }
                KeyCode::Char('j') => app.state.scroll_view_state.scroll_down(),
                KeyCode::Char('k') => app.state.scroll_view_state.scroll_up(),
                KeyCode::Char('J') => app.state.scroll_view_state.scroll_page_down(),
                KeyCode::Char('K') => app.state.scroll_view_state.scroll_page_up(),
                KeyCode::Char('g') => app.state.scroll_view_state.scroll_to_top(),
                KeyCode::Char('G') => app.state.scroll_view_state.scroll_to_bottom(),
                _ => {}
            }
        }

        if event_state.mode == Mode::PopupInsert && key_event.code == KeyCode::Esc {
            app.state.mode = Mode::Popup
        }

        Ok(())
    }
}
