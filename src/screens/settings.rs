use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{utils::RenderScreen, App};

pub struct Settings;

impl RenderScreen for Settings {
    fn init() -> Settings {
        Settings {}
    }

    fn render(&mut self, frame: &mut Frame, _: &App, area: Rect) {
        let text = Paragraph::new("Version: 0.0.1");
        frame.render_widget(text, area);
    }
}
