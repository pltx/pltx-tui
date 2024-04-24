use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Widget,
    Frame,
};

use crate::{
    components::{self, TextInput},
    state::{Mode, State},
    utils::{Init, KeyEventHandlerReturn, RenderPopup},
    App,
};

struct Inputs {
    title: TextInput,
}

struct ListData {
    project_id: i32,
    id: i32,
    title: String,
}
pub struct ListEditor {
    new: bool,
    data: Option<ListData>,
    pub width: u16,
    pub height: u16,
    project_id: Option<i32>,
    inputs: Inputs,
}

impl Init for ListEditor {
    fn init(_: &mut crate::App) -> ListEditor {
        ListEditor {
            new: false,
            data: None,
            width: 60,
            height: 5,
            project_id: None,
            inputs: Inputs {
                title: TextInput::new().set_title("Title").set_max(50),
            },
        }
    }
}

impl ListEditor {
    fn db_new_list(&self, app: &mut App) -> rusqlite::Result<()> {
        if self.project_id.is_none() {
            panic!("project_id was not set")
        }

        struct ProjectQuery {
            position: i32,
        }
        let mut stmt = app.db.conn.prepare("SELECT position from project_list")?;
        let project_iter = stmt.query_map([], |r| {
            Ok(ProjectQuery {
                position: r.get(0)?,
            })
        })?;
        let mut highest_position = 0;
        for project in project_iter {
            let project_pos = project.unwrap().position;
            if project_pos > highest_position {
                highest_position = project_pos;
            }
        }

        // TODO: Replace with error notification
        if highest_position >= 5 {
            return Ok(());
        }

        app.db.conn.execute(
            "INSERT INTO project_list (project_id, title, position) VALUES (?1, ?2, ?3)",
            (
                Some(&self.project_id),
                &self.inputs.title.input,
                highest_position,
            ),
        )?;
        Ok(())
    }

    fn db_edit_list(&self, app: &mut App) -> rusqlite::Result<()> {
        if let Some(data) = &self.data {
            let query = "UPDATE project_list SET title = ?1 WHERE id = ?2";
            let mut stmt = app.db.conn.prepare(query)?;
            stmt.execute(rusqlite::params![&self.inputs.title.input, data.id,])?;
        } else {
            panic!("list data was not set");
        }

        Ok(())
    }
}

impl KeyEventHandlerReturn<bool> for ListEditor {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) -> bool {
        self.inputs.title.handle_key_event(app, key_event);

        if key_event.code == KeyCode::Enter {
            if self.new {
                self.db_new_list(app).unwrap_or_else(|e| panic!("{e}"));
            } else {
                self.db_edit_list(app).unwrap_or_else(|e| panic!("{e}"));
            }
            app.state.mode = Mode::Navigation;
            self.inputs.title.reset();
            return true;
        }

        false
    }
}

impl RenderPopup for ListEditor {
    fn render(&mut self, frame: &mut Frame, app: &App) {
        let popup = components::Popup::new(app, frame.size())
            .set_title_top(if self.new { "New List" } else { "Edit List" })
            .set_size(self.width, self.height)
            .render(frame);

        let [title_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Length(3)])
            .areas(popup.area);

        frame.render_widget(self.title(app), title_layout);
    }
}

impl ListEditor {
    pub fn set_new(mut self) -> Self {
        self.new = true;
        self
    }

    pub fn set_project_id(&mut self, project_id: i32) {
        self.project_id = Some(project_id)
    }

    pub fn set_list(&mut self, app: &App, list_id: i32) -> rusqlite::Result<()> {
        let list_query = "SELECT id, project_id, title FROM project_list WHERE id = ?1";
        let mut list_stmt = app.db.conn.prepare(list_query)?;
        let list = list_stmt.query_row([list_id], |r| {
            Ok(ListData {
                id: r.get(0)?,
                project_id: r.get(1)?,
                title: r.get(2)?,
            })
        })?;

        self.data = Some(ListData {
            id: list.id,
            project_id: list.project_id,
            title: list.title.clone(),
        });

        self.inputs.title.set_input(list.title.clone());
        self.inputs.title.cursor_end_line();

        Ok(())
    }

    fn title(&self, app: &App) -> impl Widget {
        self.inputs.title.render(app, true)
    }
}
