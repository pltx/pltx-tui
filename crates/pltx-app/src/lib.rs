//! Contains the application state. The [`App`] is passed to all functions
//! that require state throughout the application.

use pltx_config::{Config, ProfileConfig};
use pltx_database::Database;
use state::{AppModule, AppPopup, Mode, ModeColors, View};

mod module;
/// Application state that affects what is rendered on the screen.
pub mod state;
mod widget;

pub use module::*;
pub use widget::*;

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

impl DebugMode {
    /// Toggle whether the debug pane is showing.
    pub fn toggle(&mut self) {
        if self.enabled {
            self.show = !self.show
        }
    }

    /// Toggle whether to render the application in the minimum supported
    /// screeen size.
    pub fn toggle_min_preview(&mut self) {
        if self.enabled {
            self.min_preview = !self.min_preview;
        }
    }

    /// Move the debug pane to the next position if it's showing.
    pub fn next_position(&mut self) {
        if self.enabled && self.show {
            self.position = self.position.next();
        }
    }
}

/// The application state.
pub struct App {
    /// The user configuration after it has been merged with the base
    /// configuration.
    pub config: Config,
    /// The merged profile config values to determine which files should be used
    /// for handling data.
    pub profile: ProfileConfig,
    /// The current view to prioritize/render.
    pub view: View,
    /// The current application mode.
    pub mode: Mode,
    /// The current application module.
    pub module: AppModule,
    /// The selected popup. Will only show if the [`View`] is set to
    /// [`View::Popup`].
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
            view: View::Default,
            mode: Mode::Normal,
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

    /// Handle the tick event.
    pub fn tick(&self) {}

    /// Returns the current mode's colors.
    pub fn mode_colors(&self) -> ModeColors {
        self.mode.colors(&self.config.colors)
    }
}
