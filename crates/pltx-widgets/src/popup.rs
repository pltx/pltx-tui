use pltx_app::App;
use pltx_config::ColorsConfig;
use pltx_utils::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Clear},
    Frame,
};

#[derive(Clone)]
pub struct PopupSize {
    pub width: u16,
    pub height: u16,
    percentage_based_width: bool,
    percentage_based_height: bool,
}

impl PopupSize {
    #[allow(clippy::new_without_default)]
    pub fn new() -> PopupSize {
        PopupSize {
            width: 70,
            height: 20,
            percentage_based_width: false,
            percentage_based_height: false,
        }
    }

    pub fn percentage_based(mut self) -> Self {
        self.percentage_based_width = true;
        self.percentage_based_height = true;
        self
    }

    pub fn percentage_based_width(mut self) -> Self {
        self.percentage_based_width = true;
        self
    }

    pub fn percentage_based_height(mut self) -> Self {
        self.percentage_based_height = true;
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }
}

/// Popup component
pub struct Popup<'a> {
    title_top: Option<&'a str>,
    title_bottom: Option<&'a str>,
    pub size: PopupSize,
    pub area: Rect,
    rect: Rect,
    pub sub_area: Rect,
    colors: &'a ColorsConfig,
}

impl<'a> Popup<'a> {
    pub fn new(app: &'a App, rect: Rect) -> Popup<'a> {
        let colors = &app.config.colors;

        let size = PopupSize::new();

        let area = centered_rect(
            (size.width, size.percentage_based_width),
            (size.height, size.percentage_based_height),
            rect,
        );

        let [sub_area] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Fill(1)])
            .areas(area);

        Popup {
            title_top: None,
            title_bottom: None,
            rect,
            size,
            area,
            sub_area,
            colors,
        }
    }

    pub fn size(mut self, size: PopupSize) -> Self {
        self.size = size.clone();
        self.area = centered_rect(
            (size.width, size.percentage_based_width),
            (size.height, size.percentage_based_height),
            self.rect,
        );
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
