use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::App;

pub fn render_sleep(frame: &mut Frame, _: &App, window: Rect) {
    let text = Paragraph::new("Sleep Module");
    frame.render_widget(text, window);
}
