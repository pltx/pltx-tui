use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Clear},
    Frame,
};

use crate::{
    config::ColorsConfig,
    utils::{centered_rect, centered_rect_absolute},
    App,
};

/// Popup component.
pub struct Popup<'a> {
    pub absolute: bool,
    title_top: Option<&'a str>,
    title_bottom: Option<&'a str>,
    pub width: u16,
    pub height: u16,
    pub percentage_sizing: bool,
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
            title_top: None,
            title_bottom: None,
            width: default_width,
            height: default_height,
            percentage_sizing: false,
            rect,
            area,
            sub_area,
            colors,
        }
    }

    pub fn percentage_sizing(mut self) -> Self {
        self.percentage_sizing = true;
        self
    }

    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.area = if self.percentage_sizing {
            centered_rect(width, height, self.rect)
        } else {
            centered_rect_absolute(width, height, self.rect)
        };
        self.sub_area = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Fill(1)])
            .areas::<1>(self.area)[0];
        self
    }

    pub fn title_top(mut self, title: &'a str) -> Self {
        self.title_top = Some(title);
        self
    }

    pub fn title_bottom(mut self, title: &'a str) -> Self {
        self.title_bottom = Some(title);
        self
    }

    pub fn render(self, frame: &mut Frame) -> Self {
        let mut block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_style(Style::new().fg(self.colors.fg))
            .border_style(Style::new().fg(self.colors.popup_border))
            .bg(self.colors.popup_bg);

        if let Some(title) = self.title_top {
            block = block.title(Title::from(format!(" {title} ")).alignment(Alignment::Center))
        }
        frame.render_widget(Clear, self.area);
        frame.render_widget(block, self.area);
        self
    }
}
