use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Display, App, DefaultWidget, KeyEventHandler, Popup};
use pltx_database::Database;
use pltx_utils::DateTime;
use pltx_widgets::{PopupSize, PopupWidget, TextInput};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

struct Inputs {
    title: TextInput,
}

#[derive(Clone)]
struct ListData {
    id: i32,
    title: String,
}
pub struct ListEditor {
    is_new: bool,
    data: Option<ListData>,
    project_id: Option<i32>,
    inputs: Inputs,
    size: PopupSize,
}

impl Popup<Result<bool>> for ListEditor {
    fn init() -> ListEditor {
        let size = PopupSize::default().width(60).height(5);

        ListEditor {
            is_new: false,
            data: None,
            project_id: None,
            inputs: Inputs {
                title: TextInput::new("Title")
                    .display(Display::popup())
                    .max(50)
                    .size((size.width - 2, size.height - 2)),
            },
            size,
        }
    }

    /// Returns whether the data is the database was modified.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        self.inputs.title.key_event_handler(app, key_event);

        if let Some(project_id) = self.project_id {
            if key_event.code == KeyCode::Enter {
                if self.is_new {
                    self.db_new_list(app, project_id)?;
                } else {
                    self.db_edit_list(&app.db)?;
                }
                app.reset_display();
                self.inputs.title.reset();
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let popup = PopupWidget::new(app, area)
            .title_top(if self.is_new { "New List" } else { "Edit List" })
            .size(self.size)
            .render(frame);

        let [title_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Length(3)])
            .areas(popup.popup_area);

        self.inputs.title.render(frame, app, title_layout, true);
    }
}

impl ListEditor {
    fn db_new_list(&self, app: &mut App, project_id: i32) -> Result<i32> {
        let highest_position =
            app.db
                .get_highest_position_where("project_list", "project_id", project_id)?;

        let max_lists = app.config.modules.project_management.max_lists;
        if highest_position == max_lists - 1 {
            // TODO: Replace with error notification
            return Err(eyre!("cannot create more than {} lists", max_lists));
        }

        let query = "INSERT INTO project_list (project_id, title, position, created_at, \
                     updated_at) VALUES (?1, ?2, ?3, ?4, ?5)";
        let params = (
            project_id,
            self.inputs.title.input_string(),
            highest_position + 1,
            DateTime::now(),
            DateTime::now(),
        );
        app.db.conn().execute(query, params)?;

        let new_list_id = app.db.last_row_id("project_list")?;

        Ok(new_list_id)
    }

    fn db_edit_list(&self, db: &Database) -> Result<i32> {
        let data = self.data.as_ref().expect("list data was not set");

        let conn = db.conn();
        let query = "UPDATE project_list SET title = ?1, updated_at = ?2 WHERE id = ?3";
        let mut stmt = conn.prepare(query)?;
        stmt.execute((&self.inputs.title.input_string(), DateTime::now(), data.id))?;
        Ok(data.id)
    }
}

impl ListEditor {
    pub fn set_new(mut self) -> Self {
        self.is_new = true;
        self
    }

    pub fn project_id(&mut self, project_id: i32) {
        self.project_id = Some(project_id)
    }

    pub fn set(&mut self, db: &Database, list_id: i32) -> Result<()> {
        let conn = db.conn();
        let query = "SELECT id, title FROM project_list WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        let list = stmt.query_row([list_id], |r| {
            Ok(ListData {
                id: r.get(0)?,
                title: r.get(1)?,
            })
        })?;

        self.data = Some(list.clone());
        self.inputs.title.input(list.title);

        Ok(())
    }

    pub fn reset(&mut self, app: &mut App) {
        app.reset_display();
        self.data = None;
        self.project_id = None;
        self.inputs.title.reset();
    }
}
