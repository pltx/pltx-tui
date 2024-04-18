use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{utils::RenderScreen, App};

pub struct Sleep;

impl RenderScreen for Sleep {
    fn init() -> Sleep {
        Sleep {}
    }

    fn render(&mut self, frame: &mut Frame, _: &App, area: Rect) {
        let text = Paragraph::new("Sleep Module");
        frame.render_widget(text, area);
    }
}
