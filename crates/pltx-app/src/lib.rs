//! Contains the application state. The [`App`] is passed to all functions
//! that require state throughout the application.

use pltx_config::{Config, ProfileConfig};
use pltx_database::Database;
use ratatui::style::Color;
use state::{AppModule, AppPopup, Display, Mode};

mod module;
/// Application state that affects what is rendered on the screen.
pub mod state;
mod widget;

pub use module::*;
pub use widget::*;

/// Used to get the mode properties based on the mode.
pub struct ModeData<'a> {
    /// Text string representation of the mode.
    pub text: &'a str,
    /// The foreground color of the mode displayed in the status bar.
    pub fg: Color,
    /// The background color of the mode display in the status bar.
    pub bg: Color,
}

/// The position of the debug pane on the screen.
#[allow(missing_docs)]
pub enum DebugPosition {
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

impl DebugPosition {
    /// Get the next position of the debug pane.
    pub fn next(&self) -> Self {
        match self {
            DebugPosition::Top => DebugPosition::TopRight,
            DebugPosition::TopRight => DebugPosition::Right,
            DebugPosition::Right => DebugPosition::BottomRight,
            DebugPosition::BottomRight => DebugPosition::Bottom,
            DebugPosition::Bottom => DebugPosition::BottomLeft,
            DebugPosition::BottomLeft => DebugPosition::Left,
            DebugPosition::Left => DebugPosition::TopLeft,
            DebugPosition::TopLeft => DebugPosition::Top,
        }
    }
}

/// Debug mode state.
pub struct DebugMode {
    /// Whether debug mode is enabled. This is based on the log_level
    /// configuration. If it is set to "debug", then debug mode will be
    /// enabled.
    pub enabled: bool,
    /// Whether to show the debug pane.
    pub show: bool,
    /// Whether to render the application in the minimum supported screen size.
    pub min_preview: bool,
    /// The position of the debug pane when it's showing.
    pub position: DebugPosition,
}

/// The application state.
pub struct App {
    /// The user configuration after it has been merged with the base
    /// configuration.
    pub config: Config,
    /// The merged profile config values to determine which files should be used
    /// for handling data.
    pub profile: ProfileConfig,
    /// The current display mode.
    pub display: Display,
    /// Which module is currently being displayed.
    pub module: AppModule,
    /// The selected popup. Will only show if the display is set to
    /// [`Display::Popup`].
    pub popup: AppPopup,
    /// The breadcrumbs shown in the titlebar.
    pub breadcrumbs: Vec<String>,
    /// The database state and utility methods.
    pub db: Database,
    /// The debug state.
    pub debug: DebugMode,
    /// When set to true, the application will quit on the next frame render.
    pub exit: bool,
}

impl App {
    /// New a new instance of the application.
    pub fn new(config: Config, profile: ProfileConfig) -> App {
        let debug_enabled = &config.log_level == "debug";
        let db_file = profile.db_file.to_owned();

        App {
            config,
            profile,
            display: Display::Default(Mode::Normal),
            module: AppModule::Home,
            popup: AppPopup::None,
            breadcrumbs: vec![],
            db: Database::init(db_file),
            debug: DebugMode {
                enabled: debug_enabled,
                show: false,
                min_preview: true,
                position: DebugPosition::TopRight,
            },
            exit: false,
        }
    }

    /// Exit the application on next frame render.
    pub fn exit(&mut self) {
        self.exit = true
    }

    /// Toggle whether the debug pane is showing.
    pub fn toggle_debug(&mut self) {
        if self.debug.enabled {
            self.debug.show = !self.debug.show;
        }
    }

    /// Toggle whether to show the application in the minimum supported screeen
    /// size.
    pub fn toggle_min_preview(&mut self) {
        if self.debug.enabled {
            self.debug.min_preview = !self.debug.min_preview;
        }
    }

    /// Move the debug pane to the next position if it's showing.
    pub fn next_debug_position(&mut self) {
        if self.debug.enabled && self.debug.show {
            self.debug.position = self.debug.position.next();
        }
    }

    /// Handle the tick event.
    pub fn tick(&self) {}

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
    pub fn mode_data(&self, mode: Mode) -> ModeData {
        let colors = &self.config.colors;

        ModeData {
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
    pub fn current_mode_data(&self) -> ModeData {
        self.mode_data(self.mode())
    }
}
