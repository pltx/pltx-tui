use std::{cell::RefCell, collections::HashSet, rc::Rc, str::FromStr};

use color_eyre::Result;
use crossterm::event::KeyEvent;
use pltx_app::{state::View, App, DefaultWidget, KeyEventHandler, Popup};
use pltx_config::ColorsConfig;
use pltx_database::Database;
use pltx_utils::DateTime;
use pltx_widgets::{Form, FormInput, FormWidget, Selection, TextInput};
use ratatui::{
    layout::Rect,
    style::{Color, Stylize},
    text::Span,
    Frame,
};

use crate::open_project::ProjectLabel;

struct Inputs {
    title: Rc<RefCell<TextInput>>,
    description: Rc<RefCell<TextInput>>,
    labels: Rc<RefCell<Selection<i32>>>,
    start_date: Rc<RefCell<TextInput>>,
    due_date: Rc<RefCell<TextInput>>,
    reminder: Rc<RefCell<TextInput>>,
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
    start_date: Option<DateTime>,
    due_date: Option<DateTime>,
    reminder: Option<i32>,
    // position: i32,
    // created_at: String,
    // updated_at: String,
}

pub struct CardEditor {
    project_id: Option<i32>,
    list_id: Option<i32>,
    original_data: Option<CardData>,
    original_labels: HashSet<usize>,
    inputs: Inputs,
    form: Form,
}

impl Popup<Result<bool>> for CardEditor {
    fn init() -> Self {
        let title = TextInput::new("Title").view(View::Popup).max(50).form();
        let description = TextInput::new("Description")
            .max(4000)
            .prompt_lines(10)
            .form();
        let labels = Selection::new("Labels", vec![]).form();
        let start_date = TextInput::new("Start Date").datetime_input().form();
        let due_date = TextInput::new("Due Date").datetime_input().form();
        let reminder = TextInput::new("Reminder").datetime_input().form();

        let inputs = Inputs {
            title: Rc::clone(&title),
            description: Rc::clone(&description),
            labels: Rc::clone(&labels),
            start_date: Rc::clone(&start_date),
            due_date: Rc::clone(&due_date),
            reminder: Rc::clone(&reminder),
        };

        Self {
            project_id: None,
            list_id: None,
            original_data: None,
            original_labels: HashSet::new(),
            inputs,
            form: Form::from([
                FormInput::from(title).height(6),
                FormInput::from(description).height(15),
                labels.into(),
                FormInput::from(start_date).height(6),
                FormInput::from(due_date).height(6),
                FormInput::from(reminder).height(6),
            ])
            .default_title("New Card"),
        }
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        let result = self.form.key_event_handler(app, key_event);

        if result.is_submit() {
            return self.submit(app);
        } else if result.is_closed() {
            self.reset();
        }

        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        self.form.render(frame, app, area, true);
    }
}

impl CardEditor {
    fn db_new_card(&self, db: &Database) -> Result<i32> {
        let highest_position = db.get_highest_position("project_card")?;

        let query = "INSERT INTO project_card (project_id, list_id, title, description, \
                     important, start_date, due_date, reminder, position, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
        let params = (
            Some(self.project_id),
            Some(self.list_id),
            (*self.inputs.title).borrow().input_string(),
            (*self.inputs.description).borrow().input_string(),
            false,
            DateTime::from_input((*self.inputs.start_date).borrow().input_string()),
            DateTime::from_input((*self.inputs.due_date).borrow().input_string()),
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

        for index in (*self.inputs.labels).borrow().selected.iter() {
            let label = (*self.inputs.labels).borrow().options[*index].clone();
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
        let data = self.original_data.as_ref().expect("list data was not set");
        let query = "UPDATE project_card SET title = ?1, description = ?2, important = ?3, \
                     start_date = ?4, due_date = ?5, reminder = ?6, updated_at = ?7 WHERE id = ?8";
        let conn = db.conn();
        let mut stmt = conn.prepare(query)?;
        stmt.execute((
            (*self.inputs.title).borrow().input_string(),
            (*self.inputs.description).borrow().input_string(),
            false,
            DateTime::from_input((*self.inputs.start_date).borrow().input_string()),
            DateTime::from_input((*self.inputs.due_date).borrow().input_string()),
            Option::<String>::None,
            DateTime::now(),
            data.id,
        ))?;

        self.db_edit_card_labels(db, data.id)?;

        Ok(data.id)
    }

    fn db_edit_card_labels(&self, db: &Database, card_id: i32) -> Result<()> {
        let conn = db.conn();

        for (i, label) in (*self.inputs.labels).borrow().options.iter().enumerate() {
            if (*self.inputs.labels).borrow().selected.contains(&i) {
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
    fn submit(&mut self, app: &mut App) -> Result<bool> {
        if self.original_data.is_some() {
            self.db_edit_card(&app.db)?;
        } else {
            self.db_new_card(&app.db)?;
        }
        self.reset();
        app.view.default();
        Ok(true)
    }
}

impl CardEditor {
    pub fn labels(&self, colors: &ColorsConfig, labels: Vec<ProjectLabel>) {
        (*self.inputs.labels).borrow_mut().options(
            labels
                .into_iter()
                .map(|l| {
                    (
                        l.id,
                        Span::from(l.title)
                            .fg(Color::from_str(&l.color.clone()).unwrap_or(colors.fg)),
                    )
                })
                .collect::<Vec<(i32, Span)>>(),
        )
    }
    pub fn ids(&mut self, project_id: i32, list_id: i32) {
        self.project_id = Some(project_id);
        self.list_id = Some(list_id);
    }

    pub fn set_data(&mut self, db: &Database, card_id: i32) -> Result<()> {
        self.reset();

        let conn = db.conn();

        let query = "SELECT id, title, description, start_date, due_date, reminder, position, \
                     created_at, updated_at FROM project_card WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        let mut card = stmt.query_row([card_id], |r| {
            Ok(CardData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                start_date: DateTime::from_db_option(r.get(3)?),
                due_date: DateTime::from_db_option(r.get(4)?),
                reminder: r.get(5)?,
                // position: r.get(7)?,
                // created_at: r.get(8)?,
                // updated_at: r.get(9)?,
            })
        })?;

        self.db_get_card_labels(db, &mut card)?;
        self.original_data = Some(card);

        if let Some(data) = &self.original_data {
            (*self.inputs.title).borrow_mut().input(data.title.clone());

            if let Some(description) = &data.description {
                (*self.inputs.description)
                    .borrow_mut()
                    .input(description.clone());
            }

            if let Some(start_date) = &data.start_date {
                (*self.inputs.start_date)
                    .borrow_mut()
                    .input(start_date.display());
            }

            if let Some(due_date) = &data.due_date {
                (*self.inputs.due_date)
                    .borrow_mut()
                    .input(due_date.display());
            }

            if let Some(reminder) = &data.reminder {
                (*self.inputs.reminder)
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
            let index_in_project_labels = (*self.inputs.labels)
                .borrow()
                .options
                .iter()
                .position(|l| l.0 == label_id)
                .expect("failed to get project label index");
            (*self.inputs.labels)
                .borrow_mut()
                .selected
                .insert(index_in_project_labels);
            self.original_labels.insert(index_in_project_labels);
        }
        if let Some(start_date) = &data.start_date {
            (*self.inputs.start_date)
                .borrow_mut()
                .input(start_date.display());
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.form.reset();
    }
}
