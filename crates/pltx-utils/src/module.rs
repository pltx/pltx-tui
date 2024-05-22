use crossterm::event::KeyEvent;
use pltx_app::App;
use ratatui::{layout::Rect, Frame};

/// Create a new module. Used to represent a new
/// module or section of the application.
pub trait Module {
    fn init(app: &App) -> Self;
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent);
    fn render(&mut self, app: &App, frame: &mut Frame, area: Rect);
}

/// Create new screen. Used to represent a new screen in a module.
pub trait Screen<T = ()> {
    fn init(app: &App) -> Self;
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
    fn render(&self, app: &App, frame: &mut Frame, area: Rect);
}

/// Create a new popup.
pub trait Popup<T = ()> {
    fn init(app: &App) -> Self;
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
    fn render(&self, app: &App, frame: &mut Frame, area: Rect);
}

pub trait KeyEventHandler<T = ()> {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> T;
}
