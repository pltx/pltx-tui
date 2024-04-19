use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{
    utils::{Init, RenderScreen},
    App,
};

pub struct Sleep;

impl Init for Sleep {
    fn init(_: &mut App) -> Sleep {
        Sleep {}
    }
}

impl RenderScreen for Sleep {
    fn render(&mut self, _: &mut App, frame: &mut Frame, area: Rect) {
        let text = Paragraph::new("Sleep Module");
        frame.render_widget(text, area);
    }
}
