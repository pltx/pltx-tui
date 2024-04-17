/// The current mode.
#[derive(Eq, PartialEq, Clone)]
pub enum Mode {
    Navigation,
    Popup,
}

/// The current screen (page).
#[derive(Eq, PartialEq, Clone)]
pub enum Screen {
    Dashboard,
    Sleep,
    Settings,
}

/// The currently focused window.
#[derive(Eq, PartialEq, Clone)]
pub enum Window {
    Navigation,
    Screen,
}

/// The current popup that is showing or none.
#[derive(Eq, PartialEq, Clone)]
pub enum Popup {
    Help,
    None,
}

/// Collection of states for the application.
#[derive(Clone)]
pub struct State {
    pub mode: Mode,
    pub screen: Screen,
    pub window: Window,
    pub popup: Popup,
}
