use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{
    utils::{InitScreen, RenderScreen},
    App,
};

pub struct Settings;

impl InitScreen for Settings {
    fn init() -> Settings {
        Settings {}
    }
}

impl RenderScreen for Settings {
    fn render(&mut self, _: &App, frame: &mut Frame, area: Rect) {
        let text = Paragraph::new("Version: 0.0.1");
        frame.render_widget(text, area);
    }
}
