use pltx_config::Config;
use pltx_database::Database;
use ratatui::style::Color;
use state::{GlobalPopup, Mode, Pane, Screen, State};
use tui_scrollview::ScrollViewState;

pub mod state;

pub struct App {
    pub config: Config,
    pub db: Database,
    pub state: State,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            config,
            db: Database::init(),
            state: State {
                mode: Mode::Navigation,
                screen: Screen::Dashboard,
                pane: Pane::Navigation,
                popup: GlobalPopup::None,
                exit: false,
                scroll_view_state: ScrollViewState::new(),
            },
        }
    }

    pub fn exit(&mut self) {
        self.state.exit = true
    }

    pub fn get_mode_text(&self, mode: Mode) -> &str {
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
    pub fn get_mode_colors(&self) -> (&str, Color, Color) {
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
