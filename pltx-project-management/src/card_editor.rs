use std::{cell::RefCell, collections::HashSet, convert::From, rc::Rc, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Display, App};
use pltx_config::ColorsConfig;
use pltx_utils::{
    current_timestamp, db_datetime_to_string, parse_user_datetime_option, CompositeWidget,
    DefaultWidget, FormWidget, KeyEventHandler, Popup,
};
use pltx_widgets::{self, Buttons, Form, PopupSize, PopupWidget, Selection, Switch, TextInput};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    Frame,
};

use super::open_project::ProjectLabel;

#[derive(PartialEq, Clone, Copy)]
enum Action {
    Save,
    Cancel,
}

struct Properties {
    important: Rc<RefCell<Switch>>,
    start_date: Rc<RefCell<TextInput>>,
    due_date: Rc<RefCell<TextInput>>,
    reminder: Rc<RefCell<TextInput>>,
}

struct Inputs {
    title: TextInput,
    description: TextInput,
    labels: Selection<i32>,
    properties: Form<Properties>,
    actions: Buttons<Action>,
}

// #[derive(Clone)]
// struct CardSubtask {
//     id: i32,
//     card_id: i32,
//     value: String,
//     completed: bool,
//     created_at: String,
//     updated_at: String,
// }

#[derive(Clone)]
struct CardData {
    id: i32,
    title: String,
    description: Option<String>,
    important: bool,
    start_date: Option<String>,
    due_date: Option<String>,
    reminder: Option<i32>,
    // position: i32,
    // created_at: String,
    // updated_at: String,
}

#[derive(PartialEq)]
enum FocusedPane {
    Title,
    Description,
    Labels,
    Properties,
    Actions,
}

pub struct CardEditor {
    is_new: bool,
    data: Option<CardData>,
    project_id: Option<i32>,
    list_id: Option<i32>,
    inputs: Inputs,
    size: PopupSize,
    original_labels: HashSet<usize>,
    focused_pane: FocusedPane,
}

impl Popup<Option<i32>> for CardEditor {
    fn init(_: &App) -> Self {
        let size = PopupSize::default().percentage_based().width(80).height(80);

        let important_input = Rc::new(RefCell::new(Switch::from("Important")));
        let start_date_input = Rc::new(RefCell::new(
            TextInput::new("Start Date")
                .display(Display::popup())
                .datetime_input(),
        ));
        let due_date_input = Rc::new(RefCell::new(
            TextInput::new("Due Date")
                .display(Display::popup())
                .datetime_input(),
        ));
        let reminder_input = Rc::new(RefCell::new(
            TextInput::new("Reminder")
                .display(Display::popup())
                .datetime_input(),
        ));

        let properties = Properties {
            important: Rc::clone(&important_input),
            start_date: Rc::clone(&start_date_input),
            due_date: Rc::clone(&due_date_input),
            reminder: Rc::clone(&reminder_input),
        };

        Self {
            is_new: false,
            data: None,
            project_id: None,
            list_id: None,
            inputs: Inputs {
                title: TextInput::new("Title").display(Display::popup()).max(50),
                description: TextInput::new("Description")
                    .display(Display::popup())
                    .max(4000),
                labels: Selection::default(),
                properties: Form::new(
                    vec![important_input, start_date_input, due_date_input],
                    properties,
                    Display::popup(),
                )
                .fixed_width(34),
                actions: Buttons::from([(Action::Save, "Save Card"), (Action::Cancel, "Cancel")]),
            },
            size,
            original_labels: HashSet::new(),
            focused_pane: FocusedPane::Title,
        }
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Option<i32> {
        match self.focused_pane {
            FocusedPane::Title => {
                self.inputs.title.key_event_handler(app, key_event);
            }
            FocusedPane::Description => {
                self.inputs.description.key_event_handler(app, key_event);
            }
            FocusedPane::Labels => {
                self.inputs.labels.key_event_handler(app, key_event);
            }
            FocusedPane::Properties => self.inputs.properties.key_event_handler(app, key_event),
            FocusedPane::Actions => {}
        };

        if app.is_normal_mode() {
            match key_event.code {
                KeyCode::Char('q') => {
                    app.reset_display();
                    self.reset();
                }
                KeyCode::Char('j') => self.next_pane(),
                KeyCode::Char('k') => self.prev_pane(),
                KeyCode::Enter => {
                    if let Some(id) = self.submit(app) {
                        return Some(id);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let popup = PopupWidget::new(app, area)
            .title_top(if self.is_new { "New Card" } else { "Edit Card" })
            .size(self.size.clone())
            .render(frame);

        let label_len = self.inputs.labels.options.len() as u16;
        let label_height = if label_len > 0 { label_len + 2 } else { 0 };

        let [title_layout, description_layout, label_layout, properties_layout, actions_layout] =
            Layout::default()
                .vertical_margin(1)
                .horizontal_margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(10),
                    Constraint::Length(label_height),
                    Constraint::Length(self.inputs.properties.input_widgets.len() as u16 + 2),
                    Constraint::Length(4),
                ])
                .areas(popup.area);

        self.inputs.title.render(
            frame,
            app,
            title_layout,
            self.focused_pane == FocusedPane::Title,
        );

        self.inputs.description.render(
            frame,
            app,
            description_layout,
            self.focused_pane == FocusedPane::Description,
        );

        self.inputs.labels.render(
            frame,
            app,
            label_layout,
            self.focused_pane == FocusedPane::Labels,
        );

        self.inputs.properties.render(
            frame,
            app,
            properties_layout,
            self.focused_pane == FocusedPane::Properties,
        );

        self.inputs.actions.render(
            frame,
            app,
            actions_layout,
            self.focused_pane == FocusedPane::Actions,
        );
    }
}

impl CardEditor {
    fn db_new_card(&self, app: &mut App) -> Result<i32, &str> {
        let highest_position = app.db.get_highest_position("project_card").unwrap();

        let query = "INSERT INTO project_card (project_id, list_id, title, description, \
                     important, start_date, due_date, reminder, position) VALUES (?1, ?2, ?3, ?4, \
                     ?5, ?6, ?7, ?8, ?9)";
        let inputs = &self.inputs.properties.inputs;
        let params = (
            Some(self.project_id),
            Some(self.list_id),
            self.inputs.title.input_string(),
            self.inputs.description.input_string(),
            (*inputs.important).borrow().state,
            parse_user_datetime_option((*inputs.start_date).borrow().input_string()),
            parse_user_datetime_option((*inputs.due_date).borrow().input_string()),
            Option::<String>::None,
            highest_position + 1,
        );
        app.db.conn.execute(query, params).unwrap();

        let new_card_id = app.db.last_row_id("project_card").unwrap();

        self.db_new_card_labels(app, new_card_id).unwrap();

        Ok(new_card_id)
    }

    fn db_new_card_labels(&self, app: &App, card_id: i32) -> rusqlite::Result<()> {
        for index in self.inputs.labels.selected.iter() {
            let label = self.inputs.labels.options[*index].clone();
            let query =
                "INSERT INTO card_label (project_id, card_id, label_id) VALUES (?1, ?2, ?3)";
            app.db
                .conn
                .execute(query, (Some(self.project_id), card_id, label.0))?;
        }

        Ok(())
    }

    fn db_edit_card(&self, app: &mut App) -> Result<i32, &str> {
        if let Some(data) = &self.data {
            let query = "UPDATE project_card SET title = ?1, description = ?2, important = ?3, \
                         start_date = ?4, due_date = ?5, reminder = ?6, updated_at = ?7 WHERE id \
                         = ?8";
            let mut stmt = app.db.conn.prepare(query).unwrap();
            let inputs = &self.inputs.properties.inputs;
            stmt.execute((
                self.inputs.title.input_string(),
                self.inputs.description.input_string(),
                (*inputs.important).borrow().state,
                parse_user_datetime_option((*inputs.start_date).borrow().input_string()),
                parse_user_datetime_option((*inputs.due_date).borrow().input_string()),
                Option::<String>::None,
                current_timestamp(),
                data.id,
            ))
            .unwrap();

            self.db_edit_card_labels(app, data.id).unwrap();

            Ok(data.id)
        } else {
            Err("list data was not set")
        }
    }

    fn db_edit_card_labels(&self, app: &App, card_id: i32) -> rusqlite::Result<()> {
        for (i, label) in self.inputs.labels.options.iter().enumerate() {
            if self.inputs.labels.selected.contains(&i) {
                if !self.original_labels.contains(&i) {
                    let query = "INSERT INTO card_label (project_id, card_id, label_id) VALUES \
                                 (?1, ?2, ?3)";
                    app.db
                        .conn
                        .execute(query, (Some(self.project_id), card_id, label.0))?;
                }
            } else {
                let query = "DELETE FROM card_label WHERE card_id = ?1 and label_id = ?2";
                let mut stmt = app.db.conn.prepare(query)?;
                stmt.execute((card_id, &label.0))?;
            }
        }

        Ok(())
    }
}

impl CardEditor {
    fn next_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => self.focused_pane = FocusedPane::Description,
            FocusedPane::Description => {
                if self.inputs.labels.options.is_empty() {
                    self.focused_pane = FocusedPane::Properties;
                } else {
                    self.focused_pane = FocusedPane::Labels;
                }
            }
            FocusedPane::Labels => self
                .inputs
                .labels
                .focus_next_or(|| self.focused_pane = FocusedPane::Properties),
            FocusedPane::Properties => self
                .inputs
                .properties
                .focus_next_or(|| self.focused_pane = FocusedPane::Actions),
            FocusedPane::Actions => self
                .inputs
                .actions
                .focus_next_or(|| self.focused_pane = FocusedPane::Title),
        }
    }

    fn prev_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => {
                self.focused_pane = FocusedPane::Actions;
                self.inputs.actions.focus_last();
            }
            FocusedPane::Description => self.focused_pane = FocusedPane::Title,
            FocusedPane::Labels => self
                .inputs
                .labels
                .focus_prev_or(|| self.focused_pane = FocusedPane::Description),
            FocusedPane::Properties => self.inputs.properties.focus_prev_or(|| {
                if self.inputs.labels.options.is_empty() {
                    self.focused_pane = FocusedPane::Description;
                } else {
                    self.focused_pane = FocusedPane::Labels;
                }
            }),
            FocusedPane::Actions => self
                .inputs
                .actions
                .focus_prev_or(|| self.focused_pane = FocusedPane::Properties),
        }
    }

    fn submit(&mut self, app: &mut App) -> Option<i32> {
        if self.focused_pane == FocusedPane::Actions {
            if self.inputs.actions.is_focused(Action::Save) {
                let id = if self.is_new {
                    Some(self.db_new_card(app).unwrap_or_else(|e| panic!("{e}")))
                } else {
                    Some(self.db_edit_card(app).unwrap_or_else(|e| panic!("{e}")))
                };
                self.reset();
                app.reset_display();
                return id;
            } else if self.inputs.actions.is_focused(Action::Cancel) {
                self.reset();
                app.reset_display();
            }
        }
        None
    }
}

impl CardEditor {
    pub fn set_new(mut self) -> Self {
        self.is_new = true;
        self.inputs.actions.buttons[0].1 = String::from("Create New Card");
        self
    }

    pub fn ids(&mut self, project_id: i32, list_id: i32) {
        self.project_id = Some(project_id);
        self.list_id = Some(list_id);
    }

    pub fn labels(&mut self, colors: &ColorsConfig, labels: Vec<ProjectLabel>) {
        self.inputs.labels.options(
            labels
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    (
                        l.id,
                        Span::from(l.title.clone())
                            .style(
                                if self.focused_pane == FocusedPane::Labels
                                    && self.inputs.labels.focused_option == i
                                {
                                    Style::new().bold()
                                } else {
                                    Style::new()
                                },
                            )
                            .fg(Color::from_str(&l.color.clone()).unwrap_or(colors.fg)),
                    )
                })
                .collect::<Vec<(i32, Span)>>(),
        );
    }

    pub fn set_data(&mut self, app: &App, card_id: i32) -> rusqlite::Result<()> {
        self.reset();

        let query = "SELECT id, title, description, important, start_date, due_date, reminder, \
                     position, created_at, updated_at FROM project_card WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        let mut card = stmt.query_row([card_id], |r| {
            Ok(CardData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                important: r.get(3)?,
                start_date: db_datetime_to_string(r.get(4)?),
                due_date: db_datetime_to_string(r.get(5)?),
                reminder: r.get(6)?,
                // position: r.get(7)?,
                // created_at: r.get(8)?,
                // updated_at: r.get(9)?,
            })
        })?;

        self.db_get_card_labels(app, &mut card)?;
        self.data = Some(card);

        if let Some(data) = &self.data {
            self.inputs.title.input(data.title.clone());

            if let Some(description) = &data.description {
                self.inputs.description.input(description.clone());
            }

            (*self.inputs.properties.inputs.important)
                .borrow_mut()
                .set_state(data.important);

            if let Some(start_date) = &data.start_date {
                (*self.inputs.properties.inputs.start_date)
                    .borrow_mut()
                    .input(start_date.clone());
            }

            if let Some(due_date) = &data.due_date {
                (*self.inputs.properties.inputs.due_date)
                    .borrow_mut()
                    .input(due_date.clone());
            }

            if let Some(reminder) = &data.reminder {
                (*self.inputs.properties.inputs.reminder)
                    .borrow_mut()
                    .input(reminder.to_string());
            }
        }

        Ok(())
    }

    fn db_get_card_labels(&mut self, app: &App, data: &mut CardData) -> rusqlite::Result<()> {
        let query = "SELECT label_id from card_label WHERE card_id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        let label_id_iter = stmt.query_map([data.id], |r| r.get(0))?;

        for label in label_id_iter {
            let label_id = label.unwrap();
            let index_in_project_labels = self
                .inputs
                .labels
                .options
                .iter()
                .position(|l| l.0 == label_id)
                .unwrap();
            self.inputs.labels.selected.insert(index_in_project_labels);
            self.original_labels.insert(index_in_project_labels);
        }
        if let Some(start_date) = &data.start_date {
            (*self.inputs.properties.inputs.start_date)
                .borrow_mut()
                .input(start_date.clone());
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.inputs.title.reset();
        self.inputs.description.reset();
        self.original_labels.clear();
        self.inputs.labels.reset();
        (*self.inputs.properties.inputs.important)
            .borrow_mut()
            .reset();
        (*self.inputs.properties.inputs.start_date)
            .borrow_mut()
            .reset();
        (*self.inputs.properties.inputs.due_date)
            .borrow_mut()
            .reset();
        self.inputs.actions.focus_first();
        self.focused_pane = FocusedPane::Title;
    }
}
