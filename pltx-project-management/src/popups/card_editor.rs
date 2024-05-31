use std::{cell::RefCell, collections::HashSet, rc::Rc, str::FromStr};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{
    state::View, App, CompositeWidget, DefaultWidget, FormWidget, KeyEventHandler, Popup,
};
use pltx_config::ColorsConfig;
use pltx_database::Database;
use pltx_utils::DateTime;
use pltx_widgets::{Buttons, Form, PopupSize, PopupWidget, Selection, Switch, TextInput};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    Frame,
};

use crate::open_project::ProjectLabel;

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
    start_date: Option<DateTime>,
    due_date: Option<DateTime>,
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

impl Popup<Result<bool>> for CardEditor {
    fn init() -> Self {
        let size = PopupSize::default().percentage_based().width(80).height(80);

        let important_input = Rc::new(RefCell::new(Switch::from("Important")));
        let start_date_input = Rc::new(RefCell::new(
            TextInput::new("Start Date")
                .view(View::Popup)
                .datetime_input(),
        ));
        let due_date_input = Rc::new(RefCell::new(
            TextInput::new("Due Date")
                .view(View::Popup)
                .datetime_input(),
        ));
        let reminder_input = Rc::new(RefCell::new(
            TextInput::new("Reminder")
                .view(View::Popup)
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
                title: TextInput::new("Title").view(View::Popup).max(50),
                description: TextInput::new("Description").view(View::Popup).max(4000),
                labels: Selection::default(),
                properties: Form::new(
                    [important_input, start_date_input, due_date_input],
                    properties,
                    View::Popup,
                )
                .fixed_width(34),
                actions: Buttons::from([(Action::Save, "Save Card"), (Action::Cancel, "Cancel")]),
            },
            size,
            original_labels: HashSet::new(),
            focused_pane: FocusedPane::Title,
        }
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
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

        if app.mode.is_normal() {
            match key_event.code {
                KeyCode::Char('q') => {
                    app.view.default();
                    self.reset();
                }
                KeyCode::Char('j') => self.next_pane(),
                KeyCode::Char('k') => self.prev_pane(),
                KeyCode::Enter => return self.submit(app),
                _ => {}
            }
        }

        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let popup = PopupWidget::new(app, area)
            .title_top(if self.is_new { "New Card" } else { "Edit Card" })
            .size(self.size)
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
                .areas(popup.popup_area);

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
    fn db_new_card(&self, db: &Database) -> Result<i32> {
        let highest_position = db.get_highest_position("project_card")?;

        let query = "INSERT INTO project_card (project_id, list_id, title, description, \
                     important, start_date, due_date, reminder, position, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
        let inputs = &self.inputs.properties.inputs;
        let params = (
            Some(self.project_id),
            Some(self.list_id),
            self.inputs.title.input_string(),
            self.inputs.description.input_string(),
            (*inputs.important).borrow().state,
            DateTime::from_input((*inputs.start_date).borrow().input_string()),
            DateTime::from_input((*inputs.due_date).borrow().input_string()),
            Option::<String>::None,
            highest_position + 1,
            DateTime::now(),
            DateTime::now(),
        );
        db.conn().execute(query, params)?;

        let new_card_id = db.last_row_id("project_card")?;

        self.db_new_card_labels(db, new_card_id)?;

        Ok(new_card_id)
    }

    fn db_new_card_labels(&self, db: &Database, card_id: i32) -> Result<()> {
        let conn = db.conn();

        for index in self.inputs.labels.selected.iter() {
            let label = self.inputs.labels.options[*index].clone();
            let query = "INSERT INTO card_label (project_id, card_id, label_id, created_at, \
                         updated_at) VALUES (?1, ?2, ?3, ?4, ?5)";
            conn.execute(
                query,
                (
                    Some(self.project_id),
                    card_id,
                    label.0,
                    DateTime::now(),
                    DateTime::now(),
                ),
            )?;
        }

        Ok(())
    }

    fn db_edit_card(&self, db: &Database) -> Result<i32> {
        let data = self.data.as_ref().expect("list data was not set");
        let query = "UPDATE project_card SET title = ?1, description = ?2, important = ?3, \
                     start_date = ?4, due_date = ?5, reminder = ?6, updated_at = ?7 WHERE id = ?8";
        let conn = db.conn();
        let mut stmt = conn.prepare(query)?;
        let inputs = &self.inputs.properties.inputs;
        stmt.execute((
            self.inputs.title.input_string(),
            self.inputs.description.input_string(),
            (*inputs.important).borrow().state,
            DateTime::from_input((*inputs.start_date).borrow().input_string()),
            DateTime::from_input((*inputs.due_date).borrow().input_string()),
            Option::<String>::None,
            DateTime::now(),
            data.id,
        ))?;

        self.db_edit_card_labels(db, data.id)?;

        Ok(data.id)
    }

    fn db_edit_card_labels(&self, db: &Database, card_id: i32) -> Result<()> {
        let conn = db.conn();

        for (i, label) in self.inputs.labels.options.iter().enumerate() {
            if self.inputs.labels.selected.contains(&i) {
                if !self.original_labels.contains(&i) {
                    let query = "INSERT INTO card_label (project_id, card_id, label_id, \
                                 created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)";
                    conn.execute(
                        query,
                        (
                            Some(self.project_id),
                            card_id,
                            label.0,
                            DateTime::now(),
                            DateTime::now(),
                        ),
                    )?;
                }
            } else {
                let query = "DELETE FROM card_label WHERE card_id = ?1 and label_id = ?2";
                let mut stmt = conn.prepare(query)?;
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
            FocusedPane::Labels => self.inputs.labels.focus_next(),
            FocusedPane::Properties => self.inputs.properties.focus_next(),
            FocusedPane::Actions => self.inputs.actions.focus_next(),
        }
    }

    fn prev_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => {
                self.focused_pane = FocusedPane::Actions;
                self.inputs.actions.focus_last();
            }
            FocusedPane::Description => self.focused_pane = FocusedPane::Title,
            FocusedPane::Labels => self.inputs.labels.focus_prev(),
            FocusedPane::Properties => self.inputs.properties.focus_prev(),
            FocusedPane::Actions => self.inputs.actions.focus_prev(),
        }
    }

    fn submit(&mut self, app: &mut App) -> Result<bool> {
        if self.focused_pane == FocusedPane::Actions {
            if self.inputs.actions.is_focused(Action::Save) {
                if self.is_new {
                    self.db_new_card(&app.db)?;
                } else {
                    self.db_edit_card(&app.db)?;
                }
                self.reset();
                app.view.default();
                return Ok(true);
            } else if self.inputs.actions.is_focused(Action::Cancel) {
                self.reset();
                app.view.default();
            }
        }

        Ok(false)
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

    pub fn set_data(&mut self, db: &Database, card_id: i32) -> Result<()> {
        self.reset();

        let conn = db.conn();

        let query = "SELECT id, title, description, important, start_date, due_date, reminder, \
                     position, created_at, updated_at FROM project_card WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        let mut card = stmt.query_row([card_id], |r| {
            Ok(CardData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                important: r.get(3)?,
                start_date: DateTime::from_db_option(r.get(4)?),
                due_date: DateTime::from_db_option(r.get(5)?),
                reminder: r.get(6)?,
                // position: r.get(7)?,
                // created_at: r.get(8)?,
                // updated_at: r.get(9)?,
            })
        })?;

        self.db_get_card_labels(db, &mut card)?;
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
                    .input(start_date.display());
            }

            if let Some(due_date) = &data.due_date {
                (*self.inputs.properties.inputs.due_date)
                    .borrow_mut()
                    .input(due_date.display());
            }

            if let Some(reminder) = &data.reminder {
                (*self.inputs.properties.inputs.reminder)
                    .borrow_mut()
                    .input(reminder.to_string());
            }
        }

        Ok(())
    }

    fn db_get_card_labels(&mut self, db: &Database, data: &mut CardData) -> Result<()> {
        let conn = db.conn();
        let query = "SELECT label_id from card_label WHERE card_id = ?1";
        let mut stmt = conn.prepare(query)?;
        let label_id_iter = stmt.query_map([data.id], |r| r.get::<usize, i32>(0))?;

        for label in label_id_iter {
            let label_id = label?;
            let index_in_project_labels = self
                .inputs
                .labels
                .options
                .iter()
                .position(|l| l.0 == label_id)
                .expect("failed to get project label index");
            self.inputs.labels.selected.insert(index_in_project_labels);
            self.original_labels.insert(index_in_project_labels);
        }
        if let Some(start_date) = &data.start_date {
            (*self.inputs.properties.inputs.start_date)
                .borrow_mut()
                .input(start_date.display());
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
