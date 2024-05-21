use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyEvent;
use pltx_app::{
    state::{Mode, State},
    App,
};
use pltx_utils::{CompositeWidget, DefaultWidget, FormWidget, KeyEventHandler};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders},
    Frame,
};

/// Form widget
pub struct Form<I> {
    focused_input: usize,
    pub input_widgets: Vec<Rc<RefCell<dyn FormWidget>>>,
    pub inputs: I,
    fixed_width: Option<u16>,
}

impl<I> Form<I> {
    pub fn new(input_widgets: Vec<Rc<RefCell<dyn FormWidget>>>, inputs: I, mode: Mode) -> Self {
        let max_title_len = input_widgets
            .iter()
            .map(|i| (**i).borrow().title_len())
            .max()
            .unwrap_or(0);

        for input in input_widgets.iter() {
            let mut access_input = (**input).borrow_mut();
            access_input.form_compatible();
            access_input.mode(mode);
            access_input.max_title_len(max_title_len);
        }

        Self {
            focused_input: 0,
            input_widgets,
            inputs,
            fixed_width: None,
        }
    }
}

impl<I> DefaultWidget for Form<I> {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool) {
        let colors = &app.config.colors;

        let width_layout = if let Some(fixed_width) = self.fixed_width {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(fixed_width)])
                .split(area)[0]
        } else {
            area
        };

        let split_layout = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints(
                self.input_widgets
                    .iter()
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<Constraint>>(),
            )
            .split(width_layout);

        for (i, input) in self.input_widgets.iter().enumerate() {
            (**input).borrow_mut().render(
                frame,
                app,
                split_layout[i],
                focused && self.focused_input == i,
            );
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if focused {
                colors.primary
            } else {
                colors.border
            }));

        frame.render_widget(block, width_layout);
    }
}

impl<I> CompositeWidget for Form<I> {
    fn focus_first(&mut self) {
        self.focused_input = 0;
    }

    fn focus_last(&mut self) {
        self.focused_input = self.input_widgets.len() - 1;
    }

    fn focus_next_or<F>(&mut self, cb: F)
    where
        F: FnOnce(),
    {
        if self.is_focus_last() {
            cb()
        } else {
            self.focused_input += 1;
        }
    }

    fn focus_prev_or<F>(&mut self, cb: F)
    where
        F: FnOnce(),
    {
        if self.is_focus_first() {
            cb()
        } else {
            self.focused_input -= 1;
        }
    }

    fn is_focus_first(&self) -> bool {
        self.focused_input == 0
    }

    fn is_focus_last(&self) -> bool {
        self.focused_input == self.input_widgets.len() - 1
    }
}

impl<I> KeyEventHandler for Form<I> {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) {
        (*self.input_widgets[self.focused_input])
            .borrow_mut()
            .key_event_handler(app, key_event, event_state);
    }
}

impl<I> Form<I> {
    pub fn fixed_width(mut self, fixed_width: u16) -> Self {
        self.fixed_width = Some(fixed_width);
        self
    }
}
