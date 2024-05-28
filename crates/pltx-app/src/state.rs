/// The mode the application is in. The mode status is shown at the bottom left
/// of the status bar.
#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    /// Default mode mainly for navigation.
    Normal,
    /// For editing text inputs.
    Insert,
    /// For editing the layout and moving things around.
    Interactive,
    /// For prompting the user to delete the selected item.
    Delete,
}

/// Represents what's being being rendered at the highest level.
#[derive(PartialEq, Clone, Copy)]
pub enum Display {
    /// Render only the main content.
    Default(Mode),

    /// Renders a popup over the main content. Keypresses are directed only to
    /// the popup.
    Popup(Mode),

    /// Renders a popup with the command prompt over the main content.
    /// Keypressed are directed only to the command prompt.
    Command(Mode),
}

impl Display {
    /// Creates a new default display in normal mode.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Display::Default(Mode::Normal)
    }

    /// Creates a new popup display in normal mode.
    pub fn popup() -> Self {
        Display::Popup(Mode::Normal)
    }

    /// Creates a new command display in normal mode.
    pub fn command() -> Self {
        Display::Command(Mode::Normal)
    }

    /// Returns true if the display is [`Default`](Display::Default).
    pub fn is_default(&self) -> bool {
        matches!(self, Display::Default(_))
    }

    /// Returns true if the display is [`Popup`](Display::Popup).
    pub fn is_popup(&self) -> bool {
        matches!(self, Display::Popup(_))
    }

    /// Returns true if the display is [`Command`](Display::Command).
    pub fn is_command(&self) -> bool {
        matches!(self, Display::Command(_))
    }

    /// Extract the mode from the display.
    pub fn mode(&self) -> Mode {
        match self {
            Display::Default(mode) => *mode,
            Display::Popup(mode) => *mode,
            Display::Command(mode) => *mode,
        }
    }

    /// Returns the insert mode equivalent of the current display is.
    pub fn insert_equivalent(&self) -> Display {
        match self {
            Display::Default(_) => Display::Default(Mode::Insert),
            Display::Popup(_) => Display::Popup(Mode::Insert),
            Display::Command(_) => Display::Command(Mode::Insert),
        }
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
