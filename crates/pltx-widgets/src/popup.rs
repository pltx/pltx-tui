use pltx_app::App;
use pltx_config::ColorsConfig;
use pltx_utils::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Clear},
    Frame,
};

#[derive(Clone, Copy)]
pub struct PopupSize {
    pub width: u16,
    pub height: u16,
    percentage_based_width: bool,
    percentage_based_height: bool,
}

impl Default for PopupSize {
    fn default() -> Self {
        Self {
            width: 70,
            height: 20,
            percentage_based_width: false,
            percentage_based_height: false,
        }
    }
}

impl PopupSize {
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

/// Popup widget
pub struct PopupWidget<'a> {
    title_top: Option<&'a str>,
    title_bottom: Option<&'a str>,
    pub size: PopupSize,
    pub popup_area: Rect,
    area: Rect,
    pub sub_area: Rect,
    colors: &'a ColorsConfig,
}

// TODO: implement the CustomWidget trait
impl<'a> PopupWidget<'a> {
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

        frame.render_widget(Clear, self.popup_area);
        frame.render_widget(block, self.popup_area);
        self
    }
}

impl<'a> PopupWidget<'a> {
    pub fn new(app: &'a App, area: Rect) -> PopupWidget<'a> {
        let colors = &app.config.colors;

        let size = PopupSize::default();

        let popup = centered_rect(
            (size.width, size.percentage_based_width),
            (size.height, size.percentage_based_height),
            area,
        );

        let [sub_area] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .constraints([Constraint::Fill(1)])
            .areas(popup);

        PopupWidget {
            title_top: None,
            title_bottom: None,
            area,
            size,
            popup_area: popup,
            sub_area,
            colors,
        }
    }

    pub fn size(mut self, size: PopupSize) -> Self {
        self.size = size;
        self.popup_area = centered_rect(
            (size.width, size.percentage_based_width),
            (size.height, size.percentage_based_height),
            self.area,
        );
        self.sub_area = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .constraints([Constraint::Fill(1)])
            .areas::<1>(self.popup_area)[0];
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
}
