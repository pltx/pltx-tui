use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, CompositeWidget, DefaultWidget, KeyEventHandler};
use pltx_utils::symbols;
use ratatui::{
    layout::Rect,
    style::{Modifier, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{FormInputState, FormWidget};

const DEFAULT_HEIGHT: u16 = 12;

type SelectionOptions<T = String> = Vec<(T, Span<'static>)>;

/// Selection Widget
pub struct Selection<T> {
    pub options: SelectionOptions<T>,
    pub focused_option: usize,
    pub selected: HashSet<usize>,
    title: String,
    height: u16,
    checklist: bool,
}

impl<T> Selection<T> {
    pub fn new(title: &str, options: SelectionOptions<T>) -> Self {
        Self {
            options,
            focused_option: 0,
            selected: HashSet::new(),
            title: title.into(),
            height: DEFAULT_HEIGHT,
            checklist: false,
        }
    }

    pub fn default_height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn height(&mut self, height: u16) {
        self.height = height;
    }

    pub fn checklist(mut self) -> Self {
        self.checklist = true;
        self
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

    pub fn reset(&mut self) {
        self.focused_option = 0;
        self.selected.clear();
    }
}

impl<T> KeyEventHandler for Selection<T> {
    fn key_event_handler(&mut self, _: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => self.focus_next(),
            KeyCode::Char('k') => self.focus_prev(),
            KeyCode::Char('g') => self.focus_first(),
            KeyCode::Char('G') => self.focus_last(),
            KeyCode::Char(' ') => self.select(),
            KeyCode::Char('a') => self.toggle_all(),
            KeyCode::Char('i') => self.invert_selection(),
            _ => {}
        }
    }
}

impl<T> FormWidget for Selection<T> {
    fn form(self) -> Rc<RefCell<Self>>
    where
        Self: Sized,
    {
        Rc::new(RefCell::new(self))
    }

    fn state(&self) -> FormInputState {
        FormInputState {
            title: self.title.clone(),
            height: self.height,
            uses_insert_mode: false,
            hidden: self.options.is_empty(),
            enter_back: true,
        }
    }

    fn reset(&mut self) {
        self.reset();
    }
}

impl<T> CompositeWidget for Selection<T> {
    fn focus_next(&mut self) {
        if !self.is_focus_last() {
            self.focused_option += 1;
        }
    }

    fn focus_prev(&mut self) {
        if !self.is_focus_first() {
            self.focused_option -= 1;
        }
    }

    fn is_focus_first(&self) -> bool {
        self.focused_option == 0
    }

    fn is_focus_last(&self) -> bool {
        self.focused_option == self.options.len() - 1
    }

    fn focus_first(&mut self) {
        self.focused_option = 0;
    }

    fn focus_last(&mut self) {
        self.focused_option = self.options.len() - 1;
    }
}

impl<T> DefaultWidget for Selection<T> {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused_widget: bool) {
        let colors = &app.config.colors;

        let mut text = vec![];
        let fill_char = if self.checklist { symbols::CHECK } else { "x" };

        for (i, option) in self.options.iter().enumerate() {
            let focused = focused_widget && self.focused_option == i;

            text.push(Line::from(vec![
                Span::from(if focused { "❯" } else { " " })
                    .bold()
                    .fg(colors.primary),
                Span::from("[").fg(if self.checklist && self.selected.contains(&i) {
                    colors.tertiary_fg
                } else {
                    colors.secondary_fg
                }),
                Span::from(if self.selected.contains(&i) {
                    fill_char
                } else {
                    " "
                })
                .fg(if self.checklist {
                    colors.success
                } else {
                    colors.fg
                }),
                Span::from("] ").fg(if self.checklist && self.selected.contains(&i) {
                    colors.tertiary_fg
                } else {
                    colors.secondary_fg
                }),
                if self.checklist {
                    if self.selected.contains(&i) {
                        option
                            .1
                            .clone()
                            .fg(colors.secondary_fg)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        option.1.clone()
                    }
                } else if focused {
                    option.1.clone().bold()
                } else {
                    option.1.clone()
                },
            ]))
        }
        let paragraph = Paragraph::new(text);

        frame.render_widget(paragraph, area);
    }
}
