use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::App;

pub fn render_settings(frame: &mut Frame, _: &App, window: Rect) {
    let text = Paragraph::new("Version: 0.0.1");
    frame.render_widget(text, window);
}
