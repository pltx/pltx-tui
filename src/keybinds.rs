use color_eyre::eyre::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::{
    state::{Mode, Pane, Popup, Screen, State},
    ui::Interface,
    utils::KeyEventHandler,
    App,
};

pub struct Popups {}

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
    ) -> color_eyre::eyre::Result<()> {
        let e = event::read()?;
        // A copy of the application state at the start of the event. Since there are
        // seperate global and component-specific functions that handle events,
        // they can conflict if not using the original state of the application
        // at the start of the event. For example, if the global handler runs
        // first and opens the help menu when pressing `?`, but the help popup
        // event handler sees that the current mode state is set to `Mode::Popup`,
        // then it will immediately close the popup when the user presses `?`, thus the
        // popup will never open.
        let event_state = &app.state.clone();
        // Pass the event for the popup to handle
        // if self.state.mode == Mode::Popup {
        //     match self.state.popup {
        //         _ => {}
        //     }
        // }
        match e {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(app, interface, key_event, event_state)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    // Makes keybinds by handing key events
    fn handle_key_event(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        key_event: KeyEvent,
        event_state: &State,
    ) -> color_eyre::eyre::Result<()> {
        let screen_list = &app.get_screen_list();
        let screen_index = screen_list
            .iter()
            .position(|s| s.0 == event_state.hover_screen)
            .unwrap();

        // TODO: Convert to match statements when adding more poups
        if app.state.mode == Mode::Popup && app.state.popup == Popup::Help {
            interface
                .popups
                .help
                .key_event_handler(app, key_event, event_state);
        }

        if event_state.mode == Mode::Navigation {
            match key_event.code {
                // Show the help menu
                KeyCode::Char('?') => {
                    app.state.mode = Mode::Popup;
                    app.state.popup = Popup::Help;
                }
                // Quit the application
                KeyCode::Char('q') | KeyCode::Char('Q') => app.exit(),
                // Select and focus on the screen. Each screen must handle it's own keybinds to go
                // back to the navigation pane.
                KeyCode::Enter => {
                    if event_state.pane == Pane::Navigation {
                        app.state.screen = app.state.hover_screen.clone();
                        app.state.pane = Pane::Screen;
                    }
                }
                _ => {}
            }
            if event_state.pane == Pane::Navigation {
                match key_event.code {
                    // Go down an option
                    KeyCode::Char('j') => {
                        if screen_index == screen_list.len() - 1 {
                            app.state.hover_screen = screen_list[0].0.clone();
                        } else {
                            app.state.hover_screen = screen_list[screen_index + 1].0.clone();
                        }
                    }
                    // Go up an option
                    KeyCode::Char('k') => {
                        if screen_index == 0 {
                            app.state.hover_screen = screen_list[screen_list.len() - 1].0.clone();
                        } else {
                            app.state.hover_screen = screen_list[screen_index - 1].0.clone();
                        }
                    }
                    _ => {}
                }
            }
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
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    app.state.mode = Mode::Navigation;
                    app.state.popup = Popup::None;
                }
                KeyCode::Char('j') => app.scroll_view_state.scroll_down(),
                KeyCode::Char('k') => app.scroll_view_state.scroll_up(),
                KeyCode::Char('J') => app.scroll_view_state.scroll_page_down(),
                KeyCode::Char('K') => app.scroll_view_state.scroll_page_up(),
                KeyCode::Char('g') => app.scroll_view_state.scroll_to_top(),
                KeyCode::Char('G') => app.scroll_view_state.scroll_to_bottom(),
                _ => {}
            }
        }
        Ok(())
    }
}
