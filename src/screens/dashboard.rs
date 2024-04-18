use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{utils::RenderScreen, App};

pub struct Dashboard;

impl RenderScreen for Dashboard {
    fn init() -> Dashboard {
        Dashboard {}
    }

    fn render(&mut self, frame: &mut Frame, _: &App, area: Rect) {
        let text = Paragraph::new(format!("{}", area.height));
        frame.render_widget(text, area);
    }
}
