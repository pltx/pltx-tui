use color_eyre::eyre::WrapErr;
use ratatui::style::Color;
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
pub mod tracing;
pub mod tui;
pub mod ui;

use config::Config;
use database::Database;
use keybinds::EventHandler;
use state::{Mode, Pane, Popup, Screen, State};
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
                popup: Popup::None,
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

    fn get_mode_text(&self, mode: Mode) -> &str {
        match mode {
            Mode::Navigation => "Navigation",
            Mode::Insert => "Insert",
            Mode::Popup => "Popup",
            Mode::PopupInsert => "Popup Insert",
            Mode::Delete => "Delete",
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
