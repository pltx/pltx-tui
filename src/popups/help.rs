use ratatui::{
    layout::Alignment,
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders},
    Frame,
};

use crate::{
    utils::{centered_rect, RenderPopup},
    App,
};

pub struct Help {
    pub width: u16,
    pub height: u16,
}

impl Help {
    pub fn init() -> Help {
        Help {
            width: 40,
            height: 30,
        }
    }
}

impl RenderPopup for Help {
    fn render(self, frame: &mut Frame, app: &App) {
        let colors = &app.config.colors;

        let popup_block = Block::default()
            .title(Title::from(" Help Menu ").alignment(Alignment::Center))
            .title_style(Style::new().fg(colors.fg))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(colors.popup_border))
            .bg(colors.popup_bg);

        let popup_area = centered_rect(self.width, self.height, frame.size());
        frame.render_widget(popup_block, popup_area);
    }
}
