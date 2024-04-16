/// The current mode
#[derive(Eq, PartialEq)]
pub enum Mode {
    Navigation,
    Popup,
}

/// The current screen (page)
#[derive(Eq, PartialEq, Clone)]
pub enum Screen {
    Dashboard,
    Sleep,
    Settings,
}

/// The current window
#[derive(Eq, PartialEq, Clone)]
pub enum Window {
    Navigation,
    Screen,
}

/// The current popup
#[derive(Eq, PartialEq, Clone)]
pub enum Popup {
    Help,
    None,
}

pub struct State {
    pub mode: Mode,
    pub screen: Screen,
    pub window: Window,
    pub popup: Popup,
}
