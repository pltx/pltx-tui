// type ScreenRenderFn = &'static dyn Fn(&mut Frame, &App, Rect);

use ratatui::{layout::Rect, Frame};

use crate::App;

pub trait RenderScreen {
    fn render(frame: &mut Frame, app: &App, area: Rect);
}

pub trait RenderPopup {
    fn render(self, frame: &mut Frame, app: &App);
}
