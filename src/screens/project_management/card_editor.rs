use std::{collections::HashSet, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, Paragraph, Widget},
    Frame,
};

use super::open_project::ProjectLabel;
use crate::{
    components::{self, Buttons, PopupSize, Selection, TextInput, TextInputEvent},
    config::ColorsConfig,
    state::{Mode, State},
    utils::{current_timestamp, Init, KeyEventHandler, RenderPopupContained},
    App,
};

struct Inputs {
    title: TextInput,
    description: TextInput,
    labels: Selection<i32>,
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
    // important: bool,
    // due_date: Option<String>,
    // reminder: Option<String>,
    // position: i32,
    // created_at: String,
    // updated_at: String,
    // subtasks: Vec<CardSubtask>,
}

#[derive(PartialEq)]
enum FocusedPane {
    Title,
    Description,
    Labels,
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
    original_labels: HashSet<usize>,
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
                labels: Selection::new(Mode::Popup),
            },
            original_labels: HashSet::new(),
            focused_pane: FocusedPane::Title,
            action: Action::Save,
        }
    }
}

impl CardEditor {
    fn db_new_card(&self, app: &mut App) -> Result<i32, &str> {
        let highest_position = app.db.get_highest_position("project_card").unwrap_or(-1);

        let query = "INSERT INTO project_card (project_id, list_id, title, description, \
                     important, due_date, reminder, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, \
                     ?8)";
        let params = (
            Some(self.project_id),
            Some(self.list_id),
            self.inputs.title.input_string(),
            self.inputs.description.input_string(),
            true,
            Option::<String>::None,
            Option::<String>::None,
            highest_position.saturating_add(1),
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

impl KeyEventHandler<Option<i32>> for CardEditor {
    fn key_event_handler(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
        event_state: &State,
    ) -> Option<i32> {
        match self.focused_pane {
            FocusedPane::Title => self.inputs.title.key_event_handler(app, key_event),
            FocusedPane::Description => self.inputs.description.key_event_handler(app, key_event),
            FocusedPane::Labels => {
                self.inputs
                    .labels
                    .key_event_handler(app, key_event, event_state);
                TextInputEvent::None
            }
            FocusedPane::Actions => TextInputEvent::None,
        };

        if app.state.mode == Mode::Popup {
            match key_event.code {
                KeyCode::Char('q') => {
                    app.state.mode = Mode::Navigation;
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
}

impl CardEditor {
    fn next_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => self.focused_pane = FocusedPane::Description,
            FocusedPane::Description => self.focused_pane = FocusedPane::Labels,
            FocusedPane::Labels => self
                .inputs
                .labels
                .focus_next_or(|| self.focused_pane = FocusedPane::Actions),
            FocusedPane::Actions => match self.action {
                Action::Save => self.action = Action::Cancel,
                Action::Cancel => {
                    self.focused_pane = FocusedPane::Title;
                    self.action = Action::Save
                }
            },
        }
    }

    fn prev_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => {
                self.focused_pane = FocusedPane::Actions;
                self.action = Action::Cancel;
            }
            FocusedPane::Description => self.focused_pane = FocusedPane::Title,
            FocusedPane::Labels => self
                .inputs
                .labels
                .focus_prev_or(|| self.focused_pane = FocusedPane::Description),
            FocusedPane::Actions => match self.action {
                Action::Save => self.focused_pane = FocusedPane::Labels,
                Action::Cancel => self.action = Action::Save,
            },
        }
    }

    fn submit(&mut self, app: &mut App) -> Option<i32> {
        if self.focused_pane == FocusedPane::Actions {
            if self.action == Action::Save {
                let id = if self.is_new {
                    Some(self.db_new_card(app).unwrap_or_else(|e| panic!("{e}")))
                } else {
                    Some(self.db_edit_card(app).unwrap_or_else(|e| panic!("{e}")))
                };
                self.reset();
                app.state.mode = Mode::Navigation;
                return id;
            } else if self.action == Action::Cancel {
                self.reset();
                app.state.mode = Mode::Navigation;
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

        let [title_layout, description_layout, label_layout, actions_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Length(self.inputs.labels.options.len() as u16 + 2),
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

        frame.render_widget(self.render_labels(colors), label_layout);

        let actions = self.render_actions(colors, actions_layout);
        frame.render_widget(Block::new(), actions.1 .0);
        frame.render_widget(actions.0, actions.1 .1);
        frame.render_widget(Block::new(), actions.1 .0);
    }
}

impl CardEditor {
    fn render_labels<'b>(&self, colors: &ColorsConfig) -> Paragraph<'b> {
        self.inputs
            .labels
            .render(colors, self.focused_pane == FocusedPane::Labels)
    }

    fn render_actions(
        &self,
        colors: &ColorsConfig,
        area: Rect,
    ) -> (impl Widget, (Rect, Rect, Rect)) {
        Buttons::new(vec![
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
        .set_width(30)
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

        let query = "SELECT id, list_id, title, description, important, due_date, reminder, \
                     position, created_at, updated_at FROM project_card WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        let mut card = stmt.query_row([card_id], |r| {
            Ok(CardData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                // important: r.get(3)?,
                // due_date: r.get(4)?,
                // reminder: r.get(5)?,
                // position: r.get(6)?,
                // created_at: r.get(7)?,
                // updated_at: r.get(8)?,
                // subtasks: vec![],
            })
        })?;

        self.db_get_card_labels(app, &mut card)?;
        self.data = Some(card);

        if let Some(data) = &self.data {
            self.inputs.title.input(data.title.clone());
            if let Some(description) = &data.description {
                self.inputs.description.input(description.clone());
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

        Ok(())
    }

    pub fn reset(&mut self) {
        self.inputs.title.reset();
        self.inputs.description.reset();
        self.inputs.labels.reset();
        self.original_labels.clear();
        self.focused_pane = FocusedPane::Title;
        self.action = Action::Save;
    }
}
