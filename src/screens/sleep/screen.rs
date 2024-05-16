use pltx_app::App;
use pltx_utils::{Init, RenderScreen};
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

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
