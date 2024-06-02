use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, DefaultWidget, KeyEventHandler};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use crate::{PopupSize, PopupWidget, Scrollable};

pub struct FormInputState {
    pub title: String,
    pub height: u16,
    pub uses_insert_mode: bool,
    pub hidden: bool,
    /// Determines whether pressing enter or ] should take the user
    /// back to the input selection. If the input is composite and has multiple
    /// screens, then this should be conditionally disabled.
    pub enter_back: bool,
}

pub trait FormWidget: KeyEventHandler + DefaultWidget {
    fn form(self) -> Rc<RefCell<Self>>
    where
        Self: Sized;
    fn state(&self) -> FormInputState;
    fn reset(&mut self);
}

#[derive(PartialEq)]
pub enum EditorView {
    Selection,
    Input,
}

type FormInputWidget = Rc<RefCell<dyn FormWidget>>;

pub struct FormInput(pub Rc<RefCell<dyn FormWidget>>);

impl From<FormInputWidget> for FormInput {
    fn from(input: FormInputWidget) -> Self {
        Self(input)
    }
}

/// The form struct will handle key events and rendering inputs in a form.
/// Parent scopes must keep their own references to inputs to access their
/// values.
pub struct Form {
    inputs: Vec<FormInputWidget>,
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
            inputs: inputs
                .into_iter()
                .map(|i| i.0)
                .collect::<Vec<FormInputWidget>>(),
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
                .map(|i| i.0)
                .collect::<Vec<FormInputWidget>>(),
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
        I: Into<FormInputWidget>,
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
            (*input).borrow_mut().reset();
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
                            let border_height = 2;
                            let margin = 2;
                            let height = self.current_input_state().height + border_height + margin;
                            self.size = PopupSize::default()
                                .width(self.default_size.width)
                                .height(height);
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
                    let border_height = 2;
                    let margin = 2;
                    let height = self.current_input_state().height + border_height + margin;
                    self.size = PopupSize::default()
                        .width(self.default_size.width)
                        .height(height);
                    self.view = EditorView::Input;
                    if self.current_input_state().uses_insert_mode {
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
            if self.current_input_state().enter_back {
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

            self.current_input().key_event_handler(app, key_event);
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
                .filter(|i| !(*i).borrow().state().hidden)
                .enumerate()
                .map(|(i, input)| {
                    if self.selection.focused == i {
                        Paragraph::new(format!(" {} ", (*input).borrow().state().title))
                            .style(Style::new().bg(colors.input_focus_bg))
                    } else {
                        Paragraph::new(format!(" {} ", (*input).borrow().state().title))
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

            self.current_input().render(frame, app, input_layout, true);
        }
    }
}

impl Form {
    /// Gets the current inputs, excluding hidden inputs in the process
    fn current_input(&self) -> RefMut<dyn FormWidget> {
        let widget = self
            .inputs
            .iter()
            .filter(|i| !(*i).borrow().state().hidden)
            .collect::<Vec<&FormInputWidget>>()[self.selection.focused];
        (*widget).borrow_mut()
    }

    fn current_input_state(&self) -> FormInputState {
        let widget = self
            .inputs
            .iter()
            .filter(|i| !(*i).borrow().state().hidden)
            .collect::<Vec<&FormInputWidget>>()[self.selection.focused];
        (*widget).borrow().state()
    }
}
