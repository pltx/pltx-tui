use crossterm::event::{Event, KeyEvent};
use ratatui::{buffer::Buffer, layout::Rect, Frame};

use crate::{state::State, App};

pub trait RenderScreen {
    fn render(frame: &mut Frame, app: &App, area: Rect);
}

pub trait RenderPopup {
    fn render(&mut self, frame: &mut Frame, app: &mut App);
    fn render_widgets_into_scrollview(&self, buf: &mut Buffer, app: &App);
}

pub trait PopupEventHandler {
    fn event_handler(event: &Event, app: &mut App);
}

pub trait PopupKeyEventHandler {
    fn key_event_handler(app: &mut App, key_event: KeyEvent, event_state: &State);
}
