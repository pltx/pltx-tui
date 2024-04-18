/// The current mode.
#[derive(PartialEq, Clone)]
pub enum Mode {
    Navigation,
    Popup,
}

/// The current screen (page).
#[derive(PartialEq, Clone)]
pub enum Screen {
    Dashboard,
    ProjectManagement,
    Sleep,
    Settings,
}

/// The currently focused pane.
#[derive(PartialEq, Clone)]
pub enum Pane {
    Navigation,
    Screen,
}

/// The current popup that is showing or none.
#[derive(PartialEq, Clone)]
pub enum Popup {
    Help,
    None,
}

/// Collection of states for the application.
#[derive(Clone)]
pub struct State {
    pub mode: Mode,
    pub screen: Screen,
    pub pane: Pane,
    pub popup: Popup,
}
