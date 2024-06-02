use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::App;

/// Create a new module. Used to represent a new
/// module or section of the application.
pub trait Module<T = ()> {
    /// Initialize the module. Used to fetch data that only needs to be fetched
    /// once.
    fn init(app: &App) -> Result<Self>
    where
        Self: Sized;
    /// Used to change state and fetch data based on user interaction. Can
    /// return a custom type for the parent to handle.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
    /// Render the module.
    fn render(&self, app: &App, frame: &mut Frame, area: Rect);
}

/// Create new screen. Used to represent a new screen in a module.
pub trait Screen<T = ()> {
    /// Initialize the module. Used to fetch data that only needs to be fetched
    /// once.
    fn init(app: &App) -> Result<Self>
    where
        Self: Sized;
    /// Used to change state and fetch data based on user interaction. Can
    /// return a custom type for the parent to handle.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
    /// Render the screen.
    fn render(&self, app: &App, frame: &mut Frame, area: Rect);
}

/// Create a new popup.
pub trait Popup<T = ()> {
    /// Initialize the module. Do not use to fetch data, create separate methods
    /// that can be called by the parent based on it's own state.
    fn init() -> Self;
    /// Used to change state and fetch data based on user interaction. Can
    /// return a custom type for the parent to handle.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
    /// Render the popup.
    fn render(&self, app: &App, frame: &mut Frame, area: Rect);
}

/// For special widgets that cannot implement other component traits.
pub trait KeyEventHandler<T = ()> {
    /// Used to change state and fetch data based on user interaction. Can
    /// return a custom type for the parent to handle.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
}
