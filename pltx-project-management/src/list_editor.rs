use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Display, App};
use pltx_tracing::trace_panic;
use pltx_utils::{DefaultWidget, KeyEventHandler, Popup};
use pltx_widgets::{self, PopupSize, PopupWidget, TextInput};
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

impl Popup<Option<i32>> for ListEditor {
    fn init(_: &App) -> ListEditor {
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

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Option<i32> {
        self.inputs.title.key_event_handler(app, key_event);

        if let Some(project_id) = self.project_id {
            if key_event.code == KeyCode::Enter {
                let list_id = if self.is_new {
                    Some(
                        self.db_new_list(app, project_id)
                            .unwrap_or_else(|e| panic!("{e}")),
                    )
                } else {
                    Some(self.db_edit_list(app).unwrap_or_else(|e| panic!("{e}")))
                };
                app.reset_display();
                self.inputs.title.reset();
                return list_id;
            }
        }

        None
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let popup = PopupWidget::new(app, area)
            .title_top(if self.is_new { "New List" } else { "Edit List" })
            .size(self.size.clone())
            .render(frame);

        let [title_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Length(3)])
            .areas(popup.area);

        self.inputs.title.render(frame, app, title_layout, true);
    }
}

impl ListEditor {
    fn db_new_list(&self, app: &mut App, project_id: i32) -> Result<i32, String> {
        let highest_position = app
            .db
            .get_highest_position_where("project_list", "project_id", project_id)
            .unwrap();

        let max_lists = app.config.modules.project_management.max_lists;
        if highest_position == max_lists - 1 {
            // TODO: Replace with error notification
            return Err(format!("cannot create more than {} lists", max_lists));
        }

        let query = "INSERT INTO project_list (project_id, title, position) VALUES (?1, ?2, ?3)";
        let params = (
            project_id,
            self.inputs.title.input_string(),
            highest_position + 1,
        );
        app.db.conn.execute(query, params).unwrap();

        let new_list_id = app.db.last_row_id("project_list").unwrap();

        Ok(new_list_id)
    }

    fn db_edit_list(&self, app: &mut App) -> Result<i32, &str> {
        if let Some(data) = &self.data {
            let query = "UPDATE project_list SET title = ?1 WHERE id = ?2";
            let mut stmt = app.db.conn.prepare(query).unwrap();
            stmt.execute((&self.inputs.title.input_string(), data.id))
                .unwrap();
            Ok(data.id)
        } else {
            Err("list data was not set")
        }
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

    pub fn set(&mut self, app: &App, list_id: i32) -> Result<(), &str> {
        let query = "SELECT id, title FROM project_list WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query).unwrap();
        let list = stmt
            .query_row([list_id], |r| {
                Ok(ListData {
                    id: r.get(0)?,
                    title: r.get(1)?,
                })
            })
            .unwrap_or_else(|e| trace_panic!("{e}"));

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
