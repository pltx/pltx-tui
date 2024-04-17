use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{utils::RenderScreen, App};

pub struct ProjectManagement;

impl RenderScreen for ProjectManagement {
    fn render(frame: &mut Frame, _: &App, area: Rect) {
        let text = Paragraph::new("Project Management");
        frame.render_widget(text, area);
    }
}
