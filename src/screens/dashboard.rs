use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{utils::RenderScreen, App};

pub struct Dashboard;

impl RenderScreen for Dashboard {
    fn render(frame: &mut Frame, _: &App, area: Rect) {
        let text = Paragraph::new("Dashboard");
        frame.render_widget(text, area);
    }
}
