use tui_scrollview::ScrollViewState;

/// The current mode.
#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Navigation,
    Insert,
    Popup,
    PopupInsert,
    Delete,
    Command,
    CommandInsert,
}

/// The current screen (page).
#[derive(PartialEq, Clone)]
pub enum Screen {
    Dashboard,
    ProjectManagement,
    Sleep,
    Settings,
    None,
}

/// The currently focused pane.
#[derive(PartialEq, Clone)]
pub enum Pane {
    Navigation,
    Screen,
}

/// The current popup that is showing or none.
#[derive(PartialEq, Clone)]
pub enum GlobalPopup {
    Help,
    None,
}

/// Collection of states for the application.
#[derive(Clone)]
pub struct State {
    pub mode: Mode,
    pub screen: Screen,
    pub pane: Pane,
    pub popup: GlobalPopup,
    pub exit: bool,
    pub scroll_view_state: ScrollViewState,
}
