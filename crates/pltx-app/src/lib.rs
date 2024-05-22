use pltx_config::Config;
use pltx_database::Database;
use ratatui::style::Color;
use state::{Display, GlobalPopup, Mode, ModuleState, Pane};

pub mod state;

pub struct ModeColor<'a> {
    pub text: &'a str,
    pub fg: Color,
    pub bg: Color,
}

pub struct App {
    pub config: Config,
    pub db: Database,
    pub display: Display,
    pub module: ModuleState,
    pub pane: Pane,
    pub popup: GlobalPopup,
    pub exit: bool,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            config,
            db: Database::init(),
            display: Display::Default(Mode::Normal),
            module: ModuleState::Dashboard,
            pane: Pane::Navigation,
            popup: GlobalPopup::None,
            exit: false,
        }
    }

    pub fn exit(&mut self) {
        self.exit = true
    }

    /// Reset the display to
    /// [`Display::Default(Mode::Normal)`](Display).
    pub fn reset_display(&mut self) {
        self.display = Display::Default(Mode::Normal);
    }

    /// Sets the display to [`Display::Default()`](Display).
    pub fn default_display(&mut self) {
        self.display = Display::Default(self.mode());
    }

    /// Sets the display to [`Display::Popup()`](Display).
    pub fn popup_display(&mut self) {
        self.display = Display::Popup(self.mode());
    }

    /// Sets the display to [`Display::Command()`](Display).
    pub fn command_display(&mut self) {
        self.display = Display::Command(self.mode());
    }

    /// Returns the current application mode.
    pub fn mode(&self) -> Mode {
        self.display.mode()
    }

    fn set_mode(&mut self, mode: Mode) {
        self.display = match self.display {
            Display::Default(_) => Display::Default(mode),
            Display::Popup(_) => Display::Popup(mode),
            Display::Command(_) => Display::Command(mode),
        }
    }

    /// Sets the mode to normal.
    pub fn normal_mode(&mut self) {
        self.set_mode(Mode::Normal);
    }

    /// Sets the mode to insert.
    pub fn insert_mode(&mut self) {
        self.set_mode(Mode::Insert);
    }

    /// Sets the mode to interactive.
    pub fn interactive_mode(&mut self) {
        self.set_mode(Mode::Interactive);
    }

    /// Sets the mode to delete.
    pub fn delete_mode(&mut self) {
        self.set_mode(Mode::Delete);
    }

    /// Returns if the application is in normal mode.
    pub fn is_normal_mode(&self) -> bool {
        self.display.mode() == Mode::Normal
    }

    /// Returns if the application is in insert mode.
    pub fn is_insert_mode(&self) -> bool {
        self.display.mode() == Mode::Insert
    }

    /// Returns if the application is in interactive mode.
    pub fn is_interactive_mode(&self) -> bool {
        self.display.mode() == Mode::Interactive
    }

    /// Returns if the application is in delete mode.
    pub fn is_delete_mode(&self) -> bool {
        self.display.mode() == Mode::Delete
    }

    /// Returns the display in string form.
    pub fn display_string(&self, display: Display) -> &str {
        match display {
            Display::Default(_) => "Default",
            Display::Popup(_) => "Popup",
            Display::Command(_) => "Command",
        }
    }

    /// Returns a mode in string form with its colors.
    pub fn mode_data(&self, mode: Mode) -> ModeColor {
        let colors = &self.config.colors;

        ModeColor {
            text: match mode {
                Mode::Normal => "Normal",
                Mode::Insert => "Insert",
                Mode::Interactive => "Interactive",
                Mode::Delete => "Delete",
            },
            fg: match mode {
                Mode::Normal => colors.status_bar_normal_mode_fg,
                Mode::Insert => colors.status_bar_insert_mode_fg,
                Mode::Interactive => colors.status_bar_interactive_mode_fg,
                Mode::Delete => colors.status_bar_delete_mode_fg,
            },
            bg: match mode {
                Mode::Normal => colors.status_bar_normal_mode_bg,
                Mode::Insert => colors.status_bar_insert_mode_bg,
                Mode::Interactive => colors.status_bar_interactive_mode_bg,
                Mode::Delete => colors.status_bar_delete_mode_bg,
            },
        }
    }

    /// Returns the current mode in string form with its colors.
    pub fn current_mode_data(&self) -> ModeColor {
        self.mode_data(self.mode())
    }

    // TODO: Provide a more usable return type.
    pub fn module_list<'a>(&self) -> Vec<(ModuleState, &'a str)> {
        vec![
            (ModuleState::Dashboard, "Dashboard"),
            (ModuleState::ProjectManagement, "Project Management"),
        ]
    }
}
