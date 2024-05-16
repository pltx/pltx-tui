#![deny(unused_extern_crates)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::missing_safety_doc)]

use color_eyre::eyre::WrapErr;
use ratatui::style::Color;
use tui_scrollview::ScrollViewState;

pub mod components;
pub mod popups;
pub mod screens;
pub mod utils;

pub mod command_handler;
pub mod config;
pub mod database;
pub mod errors;
pub mod keybinds;
pub mod state;
pub mod tracing;
pub mod tui;
pub mod ui;

use command_handler::CommandHandler;
use config::Config;
use database::Database;
use keybinds::EventHandler;
use state::{GlobalPopup, Mode, Pane, Screen, State};
use ui::Interface;

pub struct App {
    exit: bool,
    config: Config,
    db: Database,
    state: State,
    scroll_view_state: ScrollViewState,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            exit: false,
            config,
            db: Database::init(),
            state: State {
                mode: Mode::Navigation,
                screen: Screen::Dashboard,
                pane: Pane::Navigation,
                popup: GlobalPopup::None,
            },
            scroll_view_state: ScrollViewState::new(),
        }
    }

    /// Runs the application's main loop until the user quits.
    pub fn run(&mut self, terminal: &mut tui::Tui) -> color_eyre::eyre::Result<()> {
        self.db.ensure_tables().unwrap_or_else(|e| panic!("{e}"));
        self.db.insert_session().unwrap_or_else(|e| panic!("{e}"));
        let mut interface = Interface::init(self);
        interface.init_data(self).unwrap_or_else(|e| panic!("{e}"));
        let mut command_handler = CommandHandler::new();
        let mut event_handler = EventHandler::init();

        while !self.exit {
            terminal.draw(|frame| {
                interface.render(frame, self, &mut command_handler);
            })?;

            event_handler
                .handle_events(self, &mut interface, &mut command_handler)
                .wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true
    }

    fn get_mode_text(&self, mode: Mode) -> &str {
        match mode {
            Mode::Navigation => "Navigation",
            Mode::Insert => "Insert",
            Mode::Popup => "Popup",
            Mode::PopupInsert => "Popup Insert",
            Mode::Delete => "Delete",
            Mode::Command => "Command",
            Mode::CommandInsert => "Command Insert",
        }
    }

    /// Returns (text, fg, bg).
    fn get_mode_colors(&self) -> (&str, Color, Color) {
        let colors = &self.config.colors;
        let get_mode_colors = || match self.state.mode {
            Mode::Navigation => (
                colors.status_bar_navigation_mode_fg,
                colors.status_bar_navigation_mode_bg,
            ),
            Mode::Insert => (
                colors.status_bar_insert_mode_fg,
                colors.status_bar_insert_mode_bg,
            ),
            Mode::Popup => (
                colors.status_bar_popup_mode_fg,
                colors.status_bar_popup_mode_bg,
            ),
            Mode::PopupInsert => (
                colors.status_bar_popup_insert_mode_fg,
                colors.status_bar_popup_insert_mode_bg,
            ),
            Mode::Delete => (
                colors.status_bar_delete_mode_fg,
                colors.status_bar_delete_mode_bg,
            ),
            Mode::Command => (
                colors.status_bar_command_mode_fg,
                colors.status_bar_command_mode_bg,
            ),
            Mode::CommandInsert => (
                colors.status_bar_command_insert_mode_fg,
                colors.status_bar_command_insert_mode_bg,
            ),
        };
        (
            self.get_mode_text(self.state.mode),
            get_mode_colors().0,
            get_mode_colors().1,
        )
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
