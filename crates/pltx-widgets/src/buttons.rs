use pltx_config::ColorsConfig;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

struct SideSpacing {
    left: u8,
    right: u8,
}

pub struct Buttons {
    width: u16,
    side_spacing: SideSpacing,
    auto_width: bool,
    buttons: Vec<(String, bool)>,
}

impl Buttons {
    pub fn from(buttons: Vec<(&str, bool)>) -> Self {
        Self {
            width: 0,
            side_spacing: SideSpacing { left: 1, right: 2 },
            auto_width: true,
            buttons: buttons
                .iter()
                .map(|b| (b.0.to_string(), b.1))
                .collect::<Vec<(String, bool)>>(),
        }
    }

    pub fn fixed_width(mut self, width: u16) -> Self {
        self.width = width;
        self.auto_width = false;
        self
    }

    pub fn side_spacing(mut self, left: u8, right: u8) -> Self {
        self.side_spacing = SideSpacing { left, right };
        self
    }

    pub fn render(&self, colors: &ColorsConfig, area: Rect, focused: bool) -> (impl Widget, Rect) {
        let layout_border_width: u16 = 2;

        let width = if self.auto_width {
            let mut longest_title = 0;
            for btn in self.buttons.iter() {
                let title_len = btn.0.chars().count();
                if title_len > longest_title {
                    longest_title = title_len;
                }
            }
            (longest_title as u16).saturating_add(
                layout_border_width
                    + self.side_spacing.left as u16
                    + self.side_spacing.right as u16,
            )
        } else {
            self.width
        };

        let [height_layout] = Layout::default()
            .constraints([Constraint::Length(self.buttons.len() as u16 + 2)])
            .areas(area);

        let [layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(width)])
            .areas(height_layout);

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if focused {
                colors.primary
            } else {
                colors.border
            }));

        let button_line = |(title, btn_focused): &(String, bool)| -> Line {
            Line::from(vec![
                Span::from(format!(
                    "{}{}{}",
                    " ".repeat(self.side_spacing.left as usize),
                    title,
                    " ".repeat(self.side_spacing.right as usize),
                )),
                Span::from(" ".repeat((layout.width as usize).saturating_sub(
                    title.chars().count()
                        + self.side_spacing.left as usize
                        + self.side_spacing.right as usize,
                ))),
            ])
            .style(if focused && *btn_focused {
                Style::new()
                    .bold()
                    .fg(colors.active_fg)
                    .bg(colors.active_bg)
            } else {
                Style::new().fg(colors.secondary)
            })
        };

        let paragraph = Paragraph::new(Text::from(
            self.buttons.iter().map(button_line).collect::<Vec<Line>>(),
        ))
        .block(block);

        (paragraph, layout)
    }
}
