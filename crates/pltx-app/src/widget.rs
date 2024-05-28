use ratatui::{layout::Rect, Frame};

use crate::{state::Display, App, KeyEventHandler};

pub trait DefaultWidget {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool);
}

pub trait MutableWidget {
    fn render(&mut self, frame: &mut Frame, app: &App, area: Rect, focused: bool);
}

pub trait FormWidget: DefaultWidget + KeyEventHandler {
    fn form_compatible(&mut self);
    fn display(&mut self, display: Display);
    fn title_len(&self) -> u16;
    fn max_title_len(&mut self, max_title_len: u16);
    fn reset(&mut self);
}

pub trait CompositeWidget {
    fn focus_first(&mut self);
    fn focus_last(&mut self);
    fn focus_next(&mut self);
    fn focus_prev(&mut self);
    fn is_focus_first(&self) -> bool;
    fn is_focus_last(&self) -> bool;
}
