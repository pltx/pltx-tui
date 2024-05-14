use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
    Frame,
};

use crate::{
    components::{self, Buttons, PopupSize, TextInput, TextInputEvent},
    config::ColorsConfig,
    state::{Mode, State},
    utils::{current_timestamp, Init, KeyEventHandler, RenderPopupContained},
    App,
};

struct Inputs {
    title: TextInput,
    description: TextInput,
}

struct ProjectLabel {
    id: i32,
    title: String,
    color: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Clone)]
struct CardLabel {
    id: i32,
    card_id: i32,
    label_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Clone)]
struct CardSubtask {
    id: i32,
    card_id: i32,
    value: String,
    completed: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Clone)]
struct CardData {
    id: i32,
    list_id: i32,
    title: String,
    description: Option<String>,
    important: bool,
    due_date: Option<String>,
    reminder: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
    labels: Vec<CardLabel>,
    subtasks: Vec<CardSubtask>,
}

#[derive(PartialEq)]
enum FocusedPane {
    Title,
    Description,
    Actions,
}

#[derive(PartialEq)]
enum Action {
    Save,
    Cancel,
}

pub struct CardEditor {
    is_new: bool,
    data: Option<CardData>,
    size: PopupSize,
    project_id: Option<i32>,
    list_id: Option<i32>,
    inputs: Inputs,
    focused_pane: FocusedPane,
    action: Action,
}

impl Init for CardEditor {
    fn init(_: &mut crate::App) -> CardEditor {
        CardEditor {
            is_new: false,
            data: None,
            size: PopupSize::new().percentage_based().width(80).height(80),
            project_id: None,
            list_id: None,
            inputs: Inputs {
                title: TextInput::new(Mode::Popup).title("Title").max(100),
                description: TextInput::new(Mode::Popup).title("Description").max(4000),
            },
            focused_pane: FocusedPane::Title,
            action: Action::Save,
        }
    }
}

impl CardEditor {
    fn db_new_card(&self, app: &mut App) -> Result<i32, &str> {
        struct CardQuery {
            position: i32,
        }
        let mut stmt = app
            .db
            .conn
            .prepare("SELECT position from project_card where list_id = ?1")
            .unwrap();
        let project_iter = stmt
            .query_map([self.list_id], |r| {
                Ok(CardQuery {
                    position: r.get(0)?,
                })
            })
            .unwrap();
        let mut highest_position = 0;
        for project in project_iter {
            let project_pos = project.unwrap().position;
            if project_pos > highest_position {
                highest_position = project_pos;
            }
        }

        app.db
            .conn
            .execute(
                "INSERT INTO project_card (project_id, list_id, title, description, important, \
                 due_date, reminder, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    Some(&self.project_id),
                    Some(&self.list_id),
                    self.inputs.title.input_string(),
                    self.inputs.description.input_string(),
                    true,
                    Option::<String>::None,
                    Option::<String>::None,
                    highest_position,
                ),
            )
            .unwrap();

        let new_card_id = app.db.last_row_id("project_card").unwrap();

        Ok(new_card_id)
    }

    fn db_edit_card(&self, app: &mut App) -> Result<i32, &str> {
        if let Some(data) = &self.data {
            let query = "UPDATE project_card SET title = ?1, description = ?2, important = ?3, \
                         due_date = ?4, reminder = ?5, updated_at = ?6 WHERE id = ?7";
            let mut stmt = app.db.conn.prepare(query).unwrap();
            stmt.execute((
                self.inputs.title.input_string(),
                self.inputs.description.input_string(),
                true,
                Option::<String>::None,
                Option::<String>::None,
                current_timestamp(),
                data.id,
            ))
            .unwrap();
            Ok(data.id)
        } else {
            Err("list data was not set")
        }
    }
}

impl KeyEventHandler<Option<i32>> for CardEditor {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) -> Option<i32> {
        match self.focused_pane {
            FocusedPane::Title => self.inputs.title.handle_key_event(app, key_event),
            FocusedPane::Description => self.inputs.description.handle_key_event(app, key_event),
            _ => TextInputEvent::None,
        };

        if app.state.mode == Mode::Popup {
            match key_event.code {
                KeyCode::Char('q') => self.reset(app),
                KeyCode::Char('j') => match self.focused_pane {
                    FocusedPane::Title => self.focused_pane = FocusedPane::Description,
                    FocusedPane::Description => self.focused_pane = FocusedPane::Actions,
                    FocusedPane::Actions => match self.action {
                        Action::Save => self.action = Action::Cancel,
                        Action::Cancel => {
                            self.focused_pane = FocusedPane::Title;
                            self.action = Action::Save
                        }
                    },
                },
                KeyCode::Char('k') => match self.focused_pane {
                    FocusedPane::Title => {
                        self.focused_pane = FocusedPane::Actions;
                        self.action = Action::Cancel;
                    }
                    FocusedPane::Description => self.focused_pane = FocusedPane::Title,
                    FocusedPane::Actions => match self.action {
                        Action::Save => self.focused_pane = FocusedPane::Description,
                        Action::Cancel => self.action = Action::Save,
                    },
                },
                KeyCode::Enter => {
                    if self.focused_pane == FocusedPane::Actions {
                        if self.action == Action::Save {
                            let id = if self.is_new {
                                Some(self.db_new_card(app).unwrap_or_else(|e| panic!("{e}")))
                            } else {
                                Some(self.db_edit_card(app).unwrap_or_else(|e| panic!("{e}")))
                            };
                            self.reset(app);
                            return id;
                        } else if self.action == Action::Cancel {
                            self.reset(app);
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }
}

impl RenderPopupContained for CardEditor {
    fn render(&mut self, frame: &mut Frame, app: &App, area: Rect) {
        let colors = &app.config.colors;

        let popup = components::Popup::new(app, area)
            .title_top(if self.is_new { "New Card" } else { "Edit Card" })
            .size(self.size.clone())
            .render(frame);

        let [title_layout, description_layout, actions_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Length(6),
            ])
            .areas(popup.area);

        frame.render_widget(
            self.inputs.title.render_block(
                app,
                self.size.width - 2,
                self.size.height - 2,
                self.focused_pane == FocusedPane::Title,
            ),
            title_layout,
        );
        frame.render_widget(
            self.inputs.description.render_block(
                app,
                self.size.width - 2,
                self.size.height - 2,
                self.focused_pane == FocusedPane::Description,
            ),
            description_layout,
        );

        let actions = self.actions(colors, actions_layout);
        frame.render_widget(Block::new(), actions.1 .0);
        frame.render_widget(actions.0, actions.1 .1);
        frame.render_widget(Block::new(), actions.1 .0);
    }
}

impl CardEditor {
    fn actions(&self, colors: &ColorsConfig, area: Rect) -> (impl Widget, (Rect, Rect, Rect)) {
        Buttons::new()
            .set_width(30)
            .set_buttons(vec![
                (
                    if self.is_new {
                        "Create New Card"
                    } else {
                        "Save Card"
                    },
                    self.focused_pane == FocusedPane::Actions && self.action == Action::Save,
                ),
                (
                    "Cancel",
                    self.focused_pane == FocusedPane::Actions && self.action == Action::Cancel,
                ),
            ])
            .render(colors, area, self.focused_pane == FocusedPane::Actions)
    }
}

impl CardEditor {
    pub fn empty(mut self) -> Self {
        self.is_new = true;
        self
    }

    pub fn ids(&mut self, project_id: i32, list_id: i32) {
        self.project_id = Some(project_id);
        self.list_id = Some(list_id);
    }

    pub fn set(&mut self, app: &App, card_id: i32) -> rusqlite::Result<()> {
        let query = "SELECT id, list_id, title, description, important, due_date, reminder, \
                     position, created_at, updated_at FROM project_card WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        let card = stmt.query_row([card_id], |r| {
            Ok(CardData {
                id: r.get(0)?,
                list_id: r.get(1)?,
                title: r.get(2)?,
                description: r.get(3)?,
                important: r.get(4)?,
                due_date: r.get(5)?,
                reminder: r.get(6)?,
                position: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
                labels: vec![],
                subtasks: vec![],
            })
        })?;

        self.data = Some(card.clone());
        self.inputs.title.input(card.title);
        if let Some(description) = card.description {
            self.inputs.description.input(description);
        }

        Ok(())
    }

    pub fn reset(&mut self, app: &mut App) {
        app.state.mode = Mode::Navigation;
        self.inputs.title.reset();
        self.inputs.description.reset();
        self.focused_pane = FocusedPane::Title;
        self.action = Action::Save;
    }
}
