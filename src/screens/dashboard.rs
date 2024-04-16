use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::App;

pub fn render_dashboard(frame: &mut Frame, _: &App, window: Rect) {
    let text = Paragraph::new("Dashboard");
    frame.render_widget(text, window);
}
