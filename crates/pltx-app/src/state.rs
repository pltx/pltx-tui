/// Application mode.
#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    /// Default mode for navigation.
    Normal,
    /// For editing inputs.
    Insert,
    /// For editing the layout and moving things around.
    Interactive,
    /// To prompt the user to delete something.
    Delete,
}

/// Application display
#[derive(PartialEq, Clone, Copy)]
pub enum Display {
    Default(Mode),
    Popup(Mode),
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

    pub fn mode(&self) -> Mode {
        match self {
            Display::Default(mode) => *mode,
            Display::Popup(mode) => *mode,
            Display::Command(mode) => *mode,
        }
    }

    // Returns the insert mode equivalent of whatever the current display is.
    pub fn insert_equivalent(&self) -> Display {
        match self {
            Display::Default(_) => Display::Default(Mode::Insert),
            Display::Popup(_) => Display::Popup(Mode::Insert),
            Display::Command(_) => Display::Command(Mode::Insert),
        }
    }
}

/// Represents the current section.
#[derive(PartialEq, Clone)]
pub enum ModuleState {
    Dashboard,
    ProjectManagement,
    None,
}

/// The currently focused pane.
#[derive(PartialEq, Clone)]
pub enum Pane {
    Navigation,
    Module,
}

/// The current popup that is showing or none.
#[derive(PartialEq, Clone)]
pub enum GlobalPopup {
    Help,
    None,
}
