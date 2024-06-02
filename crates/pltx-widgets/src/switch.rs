use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::View, App, DefaultWidget, FormWidgetOld, KeyEventHandler};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{FormInputState, FormWidget};

pub struct Switch {
    pub state: bool,
    title: String,
    original_state: bool,
    max_title_len: u16,
}

impl DefaultWidget for Switch {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool) {
        let colors = &app.config.colors;

        let [title_layout, input_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(self.max_title_len + 2),
                Constraint::Min(0),
            ])
            .areas(area);

        frame.render_widget(Paragraph::new(format!("{}: ", self.title)), title_layout);

        let bracket_style = if focused {
            Style::new()
                .bold()
                .fg(colors.active_fg)
                .bg(colors.active_bg)
        } else {
            Style::new().fg(colors.secondary_fg)
        };

        let paragraph = Paragraph::new(Line::from(vec![
            Span::from("[").style(bracket_style),
            Span::from(if self.state { "x" } else { " " }).style(if focused {
                Style::new()
                    .bold()
                    .fg(colors.active_fg)
                    .bg(colors.active_bg)
            } else {
                Style::new()
            }),
            Span::from("]").style(bracket_style),
        ]));

        frame.render_widget(paragraph, input_layout);
    }
}

impl FormWidgetOld for Switch {
    fn form_compatible(&mut self) {}
    fn view(&mut self, _: View) {}

    fn reset(&mut self) {
        self.state = self.original_state;
    }
}

impl FormWidget for Switch {
    fn form(self) -> Rc<RefCell<Self>>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(self))
    }

    fn state(&self) -> FormInputState {
        FormInputState {
            title: self.title.clone(),
            height: 1,
            uses_insert_mode: false,
            hidden: false,
            enter_back: true,
        }
    }

    fn reset(&mut self) {
        self.state = self.original_state;
    }
}

impl KeyEventHandler for Switch {
    fn key_event_handler(&mut self, _: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') | KeyCode::Enter => self.toggle_state(),
            _ => {}
        }
    }
}

impl From<&str> for Switch {
    fn from(title: &str) -> Self {
        Self {
            title: title.to_string(),
            original_state: false,
            state: false,
            max_title_len: 0,
        }
    }
}

impl Switch {
    pub fn toggle_state(&mut self) {
        self.state = !self.state;
    }

    pub fn set_state(&mut self, state: bool) {
        self.original_state = state;
        self.state = state;
    }
}
