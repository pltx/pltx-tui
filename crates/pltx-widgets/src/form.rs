use std::{cell::RefCell, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, DefaultWidget, KeyEventHandler};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use crate::{PopupSize, PopupWidget, Scrollable, Selection, TextInput};

pub trait FormWidget: KeyEventHandler + DefaultWidget {
    fn form(self) -> Rc<RefCell<Self>>
    where
        Self: Sized;
    fn hidden(&self) -> bool;
    fn get_title(&self) -> String;
    /// The result determines whether pressing enter or ] should take the user
    /// back to the input selection. If the input is composite and has multiple
    /// screens, then this should be conditionally disabled.
    fn enter_back(&self) -> bool;
    fn reset(&mut self);
}

#[derive(PartialEq)]
pub enum EditorView {
    Selection,
    Input,
}

pub struct FormInput {
    widget: Rc<RefCell<dyn FormWidget>>,
    height: Option<u16>,
    /// Whether the input uses insert mode.
    needs_insert: bool,
}

impl From<Rc<RefCell<TextInput>>> for FormInput {
    fn from(widget: Rc<RefCell<TextInput>>) -> Self {
        Self {
            widget,
            height: None,
            needs_insert: true,
        }
    }
}

impl From<Rc<RefCell<Selection<i32>>>> for FormInput {
    fn from(widget: Rc<RefCell<Selection<i32>>>) -> Self {
        Self {
            widget,
            height: None,
            needs_insert: false,
        }
    }
}

impl FormInput {
    pub fn new(widget: Rc<RefCell<dyn FormWidget>>) -> Self {
        Self {
            widget,
            height: None,
            needs_insert: false,
        }
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    pub fn needs_insert(mut self) -> Self {
        self.needs_insert = true;
        self
    }
}

/// The form struct will handle key events and rendering inputs in a form.
/// Parent scopes must keep their own references to inputs to access their
/// values.
pub struct Form {
    inputs: Vec<FormInput>,
    view: EditorView,
    selection: Scrollable,
    default_title: Option<String>,
    title: Option<String>,
    default_size: PopupSize,
    size: PopupSize,
    show_close_prompt: bool,
}

impl From<Vec<FormInput>> for Form {
    fn from(inputs: Vec<FormInput>) -> Self {
        let border_height = 2;
        let default_size = PopupSize::default().height(inputs.len() as u16 + border_height);

        Self {
            inputs,
            view: EditorView::Selection,
            selection: Scrollable::default(),
            default_title: None,
            title: None,
            default_size,
            size: default_size,
            show_close_prompt: false,
        }
    }
}

impl<I, const N: usize> From<[I; N]> for Form
where
    I: Into<FormInput>,
{
    fn from(inputs: [I; N]) -> Self {
        let border_height = 2;
        let default_size = PopupSize::default().height(inputs.len() as u16 + border_height);

        Self {
            inputs: inputs
                .into_iter()
                .map(Into::into)
                .collect::<Vec<FormInput>>(),
            view: EditorView::Selection,
            selection: Scrollable::default(),
            default_title: None,
            title: None,
            default_size,
            size: default_size,
            show_close_prompt: false,
        }
    }
}

impl Form {
    pub fn add_input<I>(&mut self, input: I)
    where
        I: Into<FormInput>,
    {
        self.inputs.push(input.into());
    }

    pub fn default_title(mut self, title: &str) -> Self {
        self.default_title = Some(title.to_owned());
        self.title = Some(title.to_owned());
        self
    }

    pub fn default_size(mut self, size: PopupSize) -> Self {
        self.default_size = size;
        self.size = size;
        self
    }

    pub fn title(&mut self, title: &str) {
        self.title = Some(title.to_owned());
    }

    pub fn reset(&mut self) {
        for input in self.inputs.iter_mut() {
            (*input.widget).borrow_mut().reset();
        }
        self.view = EditorView::Selection;
        self.selection.reset();
        self.title.clone_from(&self.default_title);
        self.size = self.default_size;
        self.show_close_prompt = false;
    }
}

#[derive(PartialEq)]
pub enum FormState {
    Closed,
    Submit,
    None,
}

impl FormState {
    pub fn is_closed(&self) -> bool {
        self == &FormState::Closed
    }

    pub fn is_submit(&self) -> bool {
        self == &FormState::Submit
    }

    pub fn is_none(&self) -> bool {
        self == &FormState::None
    }
}

impl KeyEventHandler<FormState> for Form {
    /// Returns whether the user has submit the form.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> FormState {
        if app.mode.is_normal() {
            if self.show_close_prompt {
                match key_event.code {
                    KeyCode::Char('y') => {
                        self.reset();
                        app.view.default();
                        return FormState::Closed;
                    }
                    KeyCode::Char('n') => {
                        self.show_close_prompt = false;
                        self.title.clone_from(&self.default_title);
                        if self.view == EditorView::Input {
                            if let Some(height) = self.current_input().height {
                                self.size = PopupSize::default()
                                    .width(self.default_size.width)
                                    .height(height);
                            } else {
                                self.size = self.default_size;
                            }
                        } else {
                            self.size = self.default_size;
                        }
                        return FormState::None;
                    }
                    _ => {}
                }
            } else if key_event.code == KeyCode::Char('q') {
                self.show_close_prompt = true;
                self.title = None;
                self.size = PopupSize::default().width(35).height(5);
                return FormState::None;
            }
        }

        if app.mode.is_normal() && self.view == EditorView::Selection {
            self.selection.key_event_handler(app, key_event);

            match key_event.code {
                KeyCode::Enter | KeyCode::Char('l') => {
                    if let Some(height) = self.current_input().height {
                        self.size = PopupSize::default()
                            .width(self.default_size.width)
                            .height(height);
                    } else {
                        self.size = self.default_size;
                    }
                    self.view = EditorView::Input;
                    if self.current_input().needs_insert {
                        app.mode.insert();
                    }
                }
                KeyCode::Char('s') => {
                    if self.view == EditorView::Selection {
                        return FormState::Submit;
                    }
                }
                _ => {}
            }
        } else if self.view == EditorView::Input {
            if (*self.current_input().widget).borrow().enter_back() {
                match key_event.code {
                    KeyCode::Char('[') => {
                        if app.mode.is_normal() {
                            self.view = EditorView::Selection;
                            self.size = self.default_size;
                        }
                    }
                    KeyCode::Enter => {
                        self.view = EditorView::Selection;
                        app.mode.normal();
                        self.size = self.default_size;
                    }
                    _ => {}
                }
            }

            (*self.current_input().widget)
                .borrow_mut()
                .key_event_handler(app, key_event);
        }

        FormState::None
    }
}

impl DefaultWidget for Form {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, _: bool) {
        if !app.view.is_popup() {
            return;
        }

        let colors = &app.config.colors;

        let mut popup = PopupWidget::new(app, area).size(self.size);
        if let Some(title) = &self.title {
            popup = popup.title_top(title);
        }
        popup = popup.render(frame);
        let area = popup.sub_area;

        if self.show_close_prompt {
            let paragraph = Paragraph::new("Close without saving? (y/n)")
                .block(Block::new().padding(Padding::proportional(1)))
                .alignment(Alignment::Center)
                .bold()
                .fg(colors.danger);
            frame.render_widget(paragraph, area);
        } else if self.view == EditorView::Selection {
            let table = self
                .inputs
                .iter()
                .filter(|i| !(*i.widget).borrow().hidden())
                .enumerate()
                .map(|(i, input)| {
                    if self.selection.focused == i {
                        Paragraph::new(format!(" {} ", (*input.widget).borrow().get_title()))
                            .style(Style::new().bg(colors.input_focus_bg))
                    } else {
                        Paragraph::new(format!(" {} ", (*input.widget).borrow().get_title()))
                            .style(Style::new().fg(colors.secondary_fg))
                    }
                })
                .collect::<Vec<Paragraph>>();

            self.selection.render(frame, area, table);
        } else {
            let [input_layout] = Layout::default()
                .margin(1)
                .constraints([Constraint::Fill(1)])
                .areas(area);

            (*self.current_input().widget)
                .borrow()
                .render(frame, app, input_layout, true);
        }
    }
}

impl Form {
    /// Gets the current inputs, excluding hidden inputs in the process
    fn current_input(&self) -> &FormInput {
        self.inputs
            .iter()
            .filter(|i| !(*i.widget).borrow().hidden())
            .collect::<Vec<&FormInput>>()[self.selection.focused]
    }
}
