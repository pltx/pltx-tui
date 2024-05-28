use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, DefaultWidget, KeyEventHandler};
use pltx_utils::symbols;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub struct Tabs<T: Clone + PartialEq> {
    tabs: Vec<(T, String)>,
    pub active: T,
}

impl<T: Clone + PartialEq> Tabs<T> {
    pub fn from(tabs: Vec<(T, &str)>) -> Self {
        Self {
            active: tabs[0].0.clone(),
            tabs: tabs
                .iter()
                .map(|t| (t.0.clone(), t.1.to_string()))
                .collect(),
        }
    }
}

impl<T: Clone + PartialEq> KeyEventHandler for Tabs<T> {
    fn key_event_handler(&mut self, _: &mut App, key_event: KeyEvent) {
        let tab_position = self
            .tabs
            .iter()
            .position(|t| t.0 == self.active)
            .expect("invalid tab position");

        match key_event.code {
            KeyCode::Char('}') => {
                if tab_position != self.tabs.len() - 1 {
                    self.active = self.tabs[tab_position + 1].0.clone();
                }
            }
            KeyCode::Char('{') => {
                if tab_position != 0 {
                    self.active = self.tabs[tab_position - 1].0.clone();
                }
            }
            _ => {}
        }
    }
}

impl<T: Clone + PartialEq> DefaultWidget for Tabs<T> {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, _: bool) {
        let colors = &app.config.colors;

        let layouts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    self.tabs
                        .iter()
                        .map(|t| Constraint::Length(t.1.chars().count() as u16 + 4))
                        .collect::<Vec<Constraint>>(),
                    vec![Constraint::Fill(1)],
                ]
                .concat(),
            )
            .split(area);

        let mut total_tabs_width = 0;

        for (i, t) in self.tabs.iter().enumerate() {
            let inside_width = t.1.chars().count() + 2;
            total_tabs_width += inside_width + 2;

            let tab = Paragraph::new(vec![
                Line::from(format!(
                    "{}{}{}",
                    symbols::border::TOP_LEFT_ROUNDED,
                    symbols::border::HORIZONTAL.repeat(inside_width),
                    symbols::border::TOP_RIGHT_ROUNDED
                )),
                Line::from(vec![
                    Span::from(format!("{} ", symbols::border::VERTICAL)),
                    Span::from(t.1.clone()).fg(if self.active == t.0 {
                        colors.tab_active_fg
                    } else {
                        colors.tab_fg
                    }),
                    Span::from(format!(" {}", symbols::border::VERTICAL)),
                ]),
                Line::from(if self.active == t.0 {
                    format!(
                        "{}{}{}",
                        symbols::border::BOTTOM_RIGHT,
                        " ".repeat(inside_width),
                        symbols::border::BOTTOM_LEFT,
                    )
                } else {
                    format!(
                        "{}{}{}",
                        symbols::border::BOTTOM_T,
                        symbols::border::HORIZONTAL.repeat(inside_width),
                        symbols::border::BOTTOM_T,
                    )
                }),
            ])
            .fg(colors.tab_border);
            frame.render_widget(tab, layouts[i]);
        }

        let border_line = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(
                symbols::border::HORIZONTAL
                    .repeat((area.width as usize).saturating_sub(total_tabs_width)),
            ),
        ])
        .fg(colors.tab_border);
        frame.render_widget(border_line, layouts[self.tabs.len()])
    }
}
