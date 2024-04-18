use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{
    utils::{InitScreen, RenderScreen},
    App,
};

pub struct Sleep;

impl InitScreen for Sleep {
    fn init() -> Sleep {
        Sleep {}
    }
}

impl RenderScreen for Sleep {
    fn render(&mut self, _: &mut App, frame: &mut Frame, area: Rect) {
        let text = Paragraph::new("Sleep Module");
        frame.render_widget(text, area);
    }
}
