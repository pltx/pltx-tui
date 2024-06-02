use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::View, App, DefaultWidget, KeyEventHandler, Popup};
use pltx_database::Database;
use pltx_utils::DateTime;
use pltx_widgets::{PopupSize, PopupWidget, TextInput};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

#[derive(Clone)]
struct ListData {
    id: i32,
    title: String,
}

pub struct ListEditor {
    project_id: Option<i32>,
    original_data: Option<ListData>,
    title_input: TextInput,
    size: PopupSize,
}

impl Popup<Result<bool>> for ListEditor {
    fn init() -> ListEditor {
        let size = PopupSize::default().width(60).height(6);

        ListEditor {
            project_id: None,
            original_data: None,
            title_input: TextInput::new("Title")
                .view(View::Popup)
                .max(50)
                .size((size.width - 2, size.height - 2))
                .prompt(),
            size,
        }
    }

    /// Returns whether the data is the database was modified.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        self.title_input.key_event_handler(app, key_event);

        if app.mode.is_normal() && key_event.code == KeyCode::Char('q') {
            self.reset(app);
            return Ok(false);
        }

        if let Some(project_id) = self.project_id {
            if key_event.code == KeyCode::Enter {
                if self.original_data.is_some() {
                    self.db_edit_list(&app.db)?;
                } else {
                    self.db_new_list(app, project_id)?;
                }
                self.reset(app);
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let popup = PopupWidget::new(app, area)
            .title_top(if self.original_data.is_some() {
                "Edit List"
            } else {
                "New List"
            })
            .size(self.size)
            .render(frame);

        let [title_layout] = Layout::default()
            .margin(2)
            .constraints([Constraint::Length(3)])
            .areas(popup.popup_area);

        self.title_input.render(frame, app, title_layout, true);
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
            self.title_input.input_string(),
            highest_position + 1,
            DateTime::now(),
            DateTime::now(),
        );
        app.db.conn().execute(query, params)?;

        let new_list_id = app.db.last_row_id("project_list")?;

        Ok(new_list_id)
    }

    fn db_edit_list(&self, db: &Database) -> Result<i32> {
        let data = self.original_data.as_ref().expect("list data was not set");

        let conn = db.conn();
        let query = "UPDATE project_list SET title = ?1, updated_at = ?2 WHERE id = ?3";
        let mut stmt = conn.prepare(query)?;
        stmt.execute((&self.title_input.input_string(), DateTime::now(), data.id))?;
        Ok(data.id)
    }
}

impl ListEditor {
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

        self.original_data = Some(list.clone());
        self.title_input.input(list.title);

        Ok(())
    }

    pub fn reset(&mut self, app: &mut App) {
        app.view.default();
        app.mode.normal();
        self.original_data = None;
        self.project_id = None;
        self.title_input.reset();
    }
}
