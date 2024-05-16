use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{
    state::{Mode, State},
    App,
};
use pltx_config::ColorsConfig;
use pltx_utils::KeyEventHandler;
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

type SelectionOptions<T = String> = Vec<(T, Span<'static>)>;

/// Selection Widget
pub struct Selection<T> {
    pub options: SelectionOptions<T>,
    pub focused_option: usize,
    pub selected: HashSet<usize>,
    mode: Mode,
}

impl<T> Selection<T> {
    pub fn new(mode: Mode) -> Self {
        Self {
            options: vec![],
            focused_option: 0,
            selected: HashSet::new(),
            mode,
        }
    }

    pub fn from(options: SelectionOptions<T>, mode: Mode) -> Self {
        Self {
            options,
            focused_option: 0,
            selected: HashSet::new(),
            mode,
        }
    }

    pub fn options(&mut self, options: SelectionOptions<T>) {
        self.options = options;
    }

    pub fn select(&mut self) {
        if self.selected.contains(&self.focused_option) {
            self.selected.remove(&self.focused_option);
        } else {
            self.selected.insert(self.focused_option);
        }
    }

    pub fn toggle_all(&mut self) {
        if self.selected.len() == self.options.len() {
            self.selected.clear();
        } else {
            for (i, _) in self.options.iter().enumerate() {
                self.selected.insert(i);
            }
        }
    }

    pub fn invert_selection(&mut self) {
        for (i, _) in self.options.iter().enumerate() {
            if self.selected.contains(&i) {
                self.selected.remove(&i);
            } else {
                self.selected.insert(i);
            }
        }
    }

    pub fn focus_next_or<F>(&mut self, cb: F)
    where
        F: FnOnce(),
    {
        if self.is_focus_last() {
            cb()
        } else {
            self.focused_option = self.focused_option.saturating_add(1);
        }
    }

    pub fn focus_prev_or<F>(&mut self, cb: F)
    where
        F: FnOnce(),
    {
        if self.is_focus_first() {
            cb()
        } else {
            self.focused_option = self.focused_option.saturating_sub(1);
        }
    }

    pub fn is_focus_first(&self) -> bool {
        self.focused_option == 0
    }

    pub fn is_focus_last(&self) -> bool {
        self.focused_option == self.options.len().saturating_sub(1)
    }

    pub fn render<'b>(&self, colors: &ColorsConfig, focused_widget: bool) -> Paragraph<'b> {
        let mut text = vec![];

        for (i, option) in self.options.iter().enumerate() {
            let focused = focused_widget && self.focused_option == i;

            text.push(Line::from(vec![
                Span::from(if focused { "❯" } else { " " })
                    .bold()
                    .fg(colors.primary),
                Span::from("[").fg(colors.secondary),
                Span::from(if self.selected.contains(&i) { "x" } else { " " }),
                Span::from("] ").fg(colors.secondary),
                if focused {
                    option.1.clone().bold()
                } else {
                    option.1.clone()
                },
            ]))
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if focused_widget {
                colors.primary
            } else {
                colors.border
            }));

        Paragraph::new(text).block(block)
    }

    pub fn reset(&mut self) {
        self.focused_option = 0;
        self.selected.clear();
    }
}

impl<T> KeyEventHandler for Selection<T> {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) {
        if app.state.mode == self.mode {
            match key_event.code {
                KeyCode::Char(' ') => self.select(),
                KeyCode::Char('a') => self.toggle_all(),
                KeyCode::Char('i') => self.invert_selection(),
                _ => {}
            }
        }
    }
}
