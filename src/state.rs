#[derive(Eq, PartialEq)]
pub enum Mode {
    Navigation,
}

#[derive(Eq, PartialEq, Clone)]
pub enum Screen {
    Dashboard,
    Sleep,
    Settings,
}

pub struct State {
    pub mode: Mode,
    pub screen: Screen,
}
