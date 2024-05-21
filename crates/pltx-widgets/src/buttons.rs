use pltx_app::App;
use pltx_utils::{CompositeWidget, DefaultWidget};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

struct SideSpacing {
    left: u8,
    right: u8,
}

/// Buttons widget
pub struct Buttons<T> {
    width: u16,
    side_spacing: SideSpacing,
    auto_width: bool,
    pub buttons: Vec<(T, String)>,
    focused_button: usize,
}

impl<T> DefaultWidget for Buttons<T> {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool) {
        let colors = &app.config.colors;

        let layout_border_width: u16 = 2;

        let width = if self.auto_width {
            let mut longest_title = 0;
            for btn in self.buttons.iter() {
                let title_len = btn.1.chars().count();
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

        let button_line = |(i, (_, title)): (usize, &(T, String))| -> Line {
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
            .style(if focused && i == self.focused_button {
                Style::new()
                    .bold()
                    .fg(colors.active_fg)
                    .bg(colors.active_bg)
            } else {
                Style::new().fg(colors.secondary)
            })
        };

        let paragraph = Paragraph::new(Text::from(
            self.buttons
                .iter()
                .enumerate()
                .map(button_line)
                .collect::<Vec<Line>>(),
        ))
        .block(block);

        frame.render_widget(paragraph, layout);
    }
}

impl<T> CompositeWidget for Buttons<T> {
    fn focus_first(&mut self) {
        self.focused_button = 0;
    }

    fn focus_last(&mut self) {
        self.focused_button = self.buttons.len() - 1;
    }

    fn focus_next_or<F>(&mut self, cb: F)
    where
        F: FnOnce(),
    {
        if self.is_focus_last() {
            cb()
        } else {
            self.focused_button += 1;
        }
    }

    fn focus_prev_or<F>(&mut self, cb: F)
    where
        F: FnOnce(),
    {
        if self.is_focus_first() {
            cb()
        } else {
            self.focused_button -= 1;
        }
    }

    fn is_focus_first(&self) -> bool {
        self.focused_button == 0
    }

    fn is_focus_last(&self) -> bool {
        self.focused_button == self.buttons.len() - 1
    }
}

impl<T, const N: usize> From<[(T, &str); N]> for Buttons<T>
where
    T: Copy,
{
    fn from(buttons: [(T, &str); N]) -> Self {
        Self {
            width: 0,
            side_spacing: SideSpacing { left: 1, right: 2 },
            auto_width: true,
            buttons: buttons
                .iter()
                .map(|b| (b.0, b.1.to_string()))
                .collect::<Vec<(T, String)>>(),
            focused_button: 0,
        }
    }
}

impl<T> Buttons<T>
where
    T: PartialEq,
{
    pub fn fixed_width(mut self, width: u16) -> Self {
        self.width = width;
        self.auto_width = false;
        self
    }

    pub fn side_spacing(mut self, left: u8, right: u8) -> Self {
        self.side_spacing = SideSpacing { left, right };
        self
    }

    pub fn is_focused(&self, compare_to: T) -> bool {
        self.buttons[self.focused_button].0 == compare_to
    }

    pub fn reset(&mut self) {
        self.focus_first();
    }
}
