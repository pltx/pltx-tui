use pltx_app::{state::Mode, App};
use ratatui::{layout::Rect, Frame};

use crate::KeyEventHandler;

pub trait DefaultWidget {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool);
}

pub trait FormWidget: DefaultWidget + KeyEventHandler {
    fn form_compatible(&mut self);
    fn mode(&mut self, mode: Mode);
    fn title_len(&self) -> u16;
    fn max_title_len(&mut self, max_title_len: u16);
    fn reset(&mut self);
}

pub trait CompositeWidget {
    fn focus_first(&mut self);
    fn focus_last(&mut self);
    fn focus_next_or<F>(&mut self, cb: F)
    where
        F: FnOnce();
    fn focus_prev_or<F>(&mut self, cb: F)
    where
        F: FnOnce();
    fn is_focus_first(&self) -> bool;
    fn is_focus_last(&self) -> bool;
}
