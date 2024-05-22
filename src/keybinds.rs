use color_eyre::eyre::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use pltx_app::{
    state::{Display, GlobalPopup, ModuleState, Pane},
    App,
};
use pltx_utils::{Module, Popup};

use crate::{command_handler::CommandHandler, ui::Interface};

pub struct EventHandler;

impl EventHandler {
    pub fn init() -> EventHandler {
        EventHandler {}
    }

    pub fn handle_events(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        command_handler: &mut CommandHandler,
    ) -> color_eyre::eyre::Result<()> {
        let e = event::read()?;
        match e {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
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
    ) -> color_eyre::eyre::Result<()> {
        let module_list = &app.module_list();
        let module_index = module_list.iter().position(|s| s.0 == app.module).unwrap();

        match app.display {
            Display::Default(_) => {
                if app.is_normal_mode() {
                    match key_event.code {
                        KeyCode::Char(':') => {
                            app.insert_mode();
                            app.command_display();
                        }
                        KeyCode::Char('?') => {
                            app.popup_display();
                            app.popup = GlobalPopup::Help;
                        }
                        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Char('L') => {
                            if app.pane == Pane::Navigation {
                                app.pane = Pane::Module;
                            }
                        }
                        _ => {}
                    }

                    if app.pane == Pane::Navigation {
                        match key_event.code {
                            // Go down an option
                            KeyCode::Char('j') => {
                                if module_index == module_list.len().saturating_sub(1) {
                                    app.module = module_list[0].0.clone();
                                } else {
                                    app.module = module_list[module_index + 1].0.clone();
                                }
                            }
                            // Go up an option
                            KeyCode::Char('k') => {
                                if module_index == 0 {
                                    app.module =
                                        module_list[module_list.len().saturating_sub(1)].0.clone();
                                } else {
                                    app.module =
                                        module_list[module_index.saturating_sub(1)].0.clone();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Display::Popup(_) => {
                if app.popup == GlobalPopup::Help {
                    interface.popups.help.key_event_handler(app, key_event);
                }
            }
            Display::Command(_) => command_handler.key_event_handler(app, key_event),
        }

        if app.is_insert_mode() && key_event.code == KeyCode::Esc {
            app.normal_mode();
        }

        match app.module {
            ModuleState::Dashboard => interface
                .modules
                .dashboard
                .key_event_handler(app, key_event),
            ModuleState::ProjectManagement => interface
                .modules
                .project_management
                .key_event_handler(app, key_event),
            _ => {}
        }

        Ok(())
    }
}
