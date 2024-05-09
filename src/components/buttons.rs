use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::config::ColorsConfig;

pub struct Buttons {
    width: u16,
    buttons: Vec<(String, bool)>,
}

impl Buttons {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Buttons {
        Buttons {
            width: 100,
            buttons: vec![],
        }
    }

    pub fn set_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn set_buttons(mut self, buttons: Vec<(&str, bool)>) -> Self {
        self.buttons = buttons
            .iter()
            .map(|b| (format!(" {} ", b.0), b.1))
            .collect::<Vec<(String, bool)>>();
        self
    }

    pub fn render(
        &self,
        colors: &ColorsConfig,
        area: Rect,
        focused: bool,
    ) -> (impl Widget, (Rect, Rect, Rect)) {
        let [space_1, layout, space_2] = Layout::default()
            .vertical_margin(1)
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width - self.width) / 2),
                Constraint::Length(self.width),
                Constraint::Length((area.width - self.width) / 2),
            ])
            .areas(area);

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if focused {
                colors.primary
            } else {
                colors.border
            }));

        let button_line = |(title, focused): &(String, bool)| -> Line {
            Line::styled(
                title.clone(),
                if *focused {
                    Style::new()
                        .bold()
                        .fg(colors.active_fg)
                        .bg(colors.active_bg)
                } else {
                    Style::new().fg(colors.secondary)
                },
            )
        };

        let paragraph = Paragraph::new(Text::from(
            self.buttons.iter().map(button_line).collect::<Vec<Line>>(),
        ))
        .centered()
        .block(block);

        (paragraph, (space_1, layout, space_2))
    }
}
