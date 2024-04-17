use ratatui::{
    layout::Alignment,
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders},
};

use crate::App;

/// Popup component.
pub struct Popup<'a> {
    pub absolute: bool,
    pub title: Option<&'a str>,
    pub block: Block<'a>,
}

impl<'a> Popup<'a> {
    fn default() -> Popup<'a> {
        Popup {
            absolute: false,
            title: Option::None,
            block: Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        }
    }

    /// Create a new popup component
    pub fn new(app: &App) -> Popup {
        let colors = &app.config.colors;

        let mut popup = Popup::default();
        popup.block = popup
            .block
            .title(Title::from(" Help Menu ").alignment(Alignment::Center))
            .title_style(Style::new().fg(colors.fg))
            .border_style(Style::new().fg(colors.popup_border))
            .bg(colors.popup_bg);

        popup
    }
}
