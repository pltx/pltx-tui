use pltx_app::App;
use ratatui::{layout::Rect, Frame};

pub trait CustomWidget {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool);
}
