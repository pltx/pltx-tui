use crossterm::event::{Event, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect, Frame};

use crate::{state::State, App};

pub trait RenderScreen {
    fn init() -> Self
    where
        Self: Sized;
    fn render(&mut self, frame: &mut Frame, app: &App, area: Rect);
}

pub trait RenderPopup {
    fn render(&mut self, frame: &mut Frame, app: &mut App);
    fn render_widgets_into_scrollview(&self, buf: &mut Buffer, app: &App);
}

pub trait EventHandler {
    fn event_handler(event: &Event, app: &mut App);
}

pub trait KeyEventHandler {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State);
}
