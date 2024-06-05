use core::fmt;

use pltx_config::ColorsConfig;
use ratatui::style::Color;

/// The mode the application is in. The mode status is shown at the bottom left
/// of the status bar.
#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    /// Default mode mainly for navigation.
    Normal,
    /// For editing text inputs.
    Insert,
    /// For prompting the user to delete the selected item.
    Delete,
}

/// Used to get the mode properties (colors).
pub struct ModeColors {
    /// The foreground color of the mode shown in the status bar.
    pub fg: Color,
    /// The background color of the mode shown in the status bar.
    pub bg: Color,
}

impl Mode {
    /// Sets the mode to [`Normal`](Mode::Normal).
    pub fn normal(&mut self) {
        *self = Mode::Normal;
    }

    /// Sets the mode to [`Insert`](Mode::Insert).
    pub fn insert(&mut self) {
        *self = Mode::Insert;
    }

    /// Sets the mode to [`Delete`](Mode::Delete).
    pub fn delete(&mut self) {
        *self = Mode::Delete;
    }

    /// Returns true if the mode is [`Normal`](Mode::Normal).
    pub fn is_normal(&self) -> bool {
        self == &Mode::Normal
    }

    /// Returns true if the mode is [`Insert`](Mode::Insert).
    pub fn is_insert(&self) -> bool {
        self == &Mode::Insert
    }

    /// Returns true if the mode is [`Delete`](Mode::Delete).
    pub fn is_delete(&self) -> bool {
        self == &Mode::Delete
    }

    /// Returns a modes colors.
    pub fn colors(&self, colors: &ColorsConfig) -> ModeColors {
        ModeColors {
            fg: match *self {
                Mode::Normal => colors.status_bar_normal_mode_fg,
                Mode::Insert => colors.status_bar_insert_mode_fg,
                Mode::Delete => colors.status_bar_delete_mode_fg,
            },
            bg: match *self {
                Mode::Normal => colors.status_bar_normal_mode_bg,
                Mode::Insert => colors.status_bar_insert_mode_bg,
                Mode::Delete => colors.status_bar_delete_mode_bg,
            },
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Mode::Normal => "Normal",
                Mode::Insert => "Insert",
                Mode::Delete => "Delete",
            }
        )
    }
}

/// Represents what's being being rendered at the highest level.
#[derive(PartialEq, Clone, Copy)]
pub enum View {
    /// Render only the main content.
    Default,
    /// Renders a popup over the main content. Keypresses are directed only to
    /// the popup.
    Popup,
    /// Renders a popup with the command prompt over the main content.
    /// Keypressed are directed only to the command prompt.
    Command,
}

impl View {
    /// Sets the view to [`Default`](View::Default).
    pub fn default(&mut self) {
        *self = View::Default;
    }

    /// Sets the view to [`Popup`](View::Popup).
    pub fn popup(&mut self) {
        *self = View::Popup
    }

    /// Sets the view to [`Command`](View::Command).
    pub fn command(&mut self) {
        *self = View::Command;
    }

    /// Returns true if the view is [`Default`](View::Default).
    pub fn is_default(&self) -> bool {
        self == &View::Default
    }

    /// Returns true if the view is [`Popup`](View::Popup).
    pub fn is_popup(&self) -> bool {
        self == &View::Popup
    }

    /// Returns true if the view is [`Command`](View::Command).
    pub fn is_command(&self) -> bool {
        self == &View::Command
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                View::Default => "Default",
                View::Popup => "Popup",
                View::Command => "Command",
            }
        )
    }
}

/// Represents the current module.
#[allow(missing_docs)]
#[derive(PartialEq, Clone)]
pub enum AppModule {
    Home,
    ProjectManagement,
    None,
}

/// Used to get the string representation of a app module.
pub struct ModuleText<'a> {
    /// Reference to the modules enum field.
    pub module: AppModule,
    /// The string representation of the module.
    pub text: &'a str,
}

/// The current popup that is showing or none.
#[allow(missing_docs)]
#[derive(PartialEq, Clone, Default)]
pub enum AppPopup {
    #[default]
    None,
}
