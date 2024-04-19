use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Clear},
    Frame,
};

use crate::{config::ColorsConfig, utils::centered_rect_absolute, App};

/// Popup component.
pub struct Popup<'a> {
    pub absolute: bool,
    pub title: Option<&'a str>,
    pub width: u16,
    pub height: u16,
    pub area: Rect,
    pub rect: Rect,
    pub sub_area: Rect,
    pub colors: &'a ColorsConfig,
}

impl<'a> Popup<'a> {
    /// Create a new popup component
    pub fn new(app: &'a App, rect: Rect) -> Popup<'a> {
        let colors = &app.config.colors;
        let default_width = 70;
        let default_height = 20;

        let area = centered_rect_absolute(default_width, default_height, rect);

        let [sub_area] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Fill(1)])
            .areas(area);

        Popup {
            absolute: false,
            title: None,
            width: default_width,
            height: default_height,
            rect,
            area,
            sub_area,
            colors,
        }
    }

    pub fn set_size(mut self, width: u16, height: u16) -> Self {
        self.area = centered_rect_absolute(width, height, self.rect);
        self.sub_area = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Fill(1)])
            .areas::<1>(self.area)[0];
        self
    }

    pub fn set_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn remove_title(mut self) -> Self {
        self.title = None;
        self
    }

    pub fn render(self, frame: &mut Frame) -> Self {
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_style(Style::new().fg(self.colors.fg))
            .border_style(Style::new().fg(self.colors.popup_border))
            .bg(self.colors.popup_bg);

        if let Some(title) = self.title {
            block = block.title(Title::from(format!(" {title} ")).alignment(Alignment::Center))
        }
        frame.render_widget(Clear, self.area);
        frame.render_widget(block, self.area);
        self
    }
}
