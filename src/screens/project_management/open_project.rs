use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use super::{list_editor::ListEditor, projects::ProjectsState, screen::ScreenPane};
use crate::{
    state::{Mode, State},
    utils::{
        pane_title_bottom, Init, InitData, KeyEventHandlerReturn, RenderPage, RenderPopup,
        ScreenKeybinds,
    },
    App,
};

struct ProjectLabel {
    id: i32,
    title: String,
    color: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
}

struct CardLabel {
    id: i32,
    card_id: i32,
    label_id: i32,
    created_at: String,
    updated_at: String,
}

struct CardSubtask {
    id: i32,
    card_id: i32,
    value: String,
    completed: i32,
    created_at: String,
    updated_at: String,
}

struct ProjectCard {
    id: i32,
    list_id: i32,
    title: String,
    description: Option<String>,
    important: i32,
    due_date: Option<String>,
    reminder: i32,
    position: i32,
    created_at: String,
    updated_at: String,
    labels: Vec<CardLabel>,
    subtasks: Vec<CardSubtask>,
}

struct ProjectList {
    id: i32,
    title: String,
    color: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
    cards: Vec<ProjectCard>,
}

#[derive(Default)]
struct ProjectData {
    id: i32,
    title: String,
    description: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
    labels: Vec<ProjectLabel>,
    lists: Vec<ProjectList>,
}

#[derive(PartialEq)]
enum Popup {
    NewList,
    EditList,
    None,
}

struct Popups {
    new_list: ListEditor,
    edit_list: ListEditor,
}

#[derive(PartialEq)]
enum FocusedPane {
    List,
    Card,
    None,
}

pub struct OpenProject {
    project_id: Option<i32>,
    selected_list_id: Option<i32>,
    selected_card_id: Option<i32>,
    data: ProjectData,
    popup: Popup,
    popups: Popups,
    focused_pane: FocusedPane,
}

impl Init for OpenProject {
    fn init(app: &mut App) -> OpenProject {
        OpenProject {
            project_id: None,
            selected_list_id: None,
            selected_card_id: None,
            data: ProjectData::default(),
            popup: Popup::None,
            popups: Popups {
                new_list: ListEditor::init(app).set_new(),
                edit_list: ListEditor::init(app),
            },
            focused_pane: FocusedPane::None,
        }
    }
}

impl InitData for OpenProject {
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()> {
        self.db_get_project(app)
    }
}

impl OpenProject {
    pub fn db_get_project(&mut self, app: &mut App) -> rusqlite::Result<()> {
        if self.project_id.is_none() {
            panic!("project_id was not set")
        }

        let project_query = "SELECT id, title, description, position, created_at, updated_at FROM \
                             project WHERE id = ?1 ORDER BY position";
        let mut project_stmt = app.db.conn.prepare(project_query)?;
        let mut project = project_stmt.query_row([&self.project_id], |r| {
            Ok(ProjectData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                position: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
                labels: vec![],
                lists: vec![],
            })
        })?;

        let project_label_query = "SELECT id, title, color, position, created_at, updated_at FROM \
                                   project_label WHERE project_id = ?1 ORDER BY position";
        let mut project_label_stmt = app.db.conn.prepare(project_label_query)?;
        let project_label_iter = project_label_stmt.query_map([&self.project_id], |r| {
            Ok(ProjectLabel {
                id: r.get(0)?,
                title: r.get(1)?,
                color: r.get(2)?,
                position: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })?;
        for label in project_label_iter {
            project.labels.push(label.unwrap());
        }

        let project_list_query = "SELECT id, title, color, position, created_at, updated_at FROM \
                                  project_list WHERE project_id = ?1 ORDER BY position";
        let mut project_list_stmt = app.db.conn.prepare(project_list_query)?;
        let project_list_iter = project_list_stmt.query_map([&self.project_id], |r| {
            Ok(ProjectList {
                id: r.get(0)?,
                title: r.get(1)?,
                color: r.get(2)?,
                position: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
                cards: vec![],
            })
        })?;
        for list in project_list_iter {
            project.lists.push(list.unwrap())
        }

        let project_card_query = "SELECT id, list_id, title, description, important, due_date, \
                                  reminder, position, created_at, updated_at FROM project_card \
                                  WHERE project_id = ?1 ORDER BY position";
        let mut project_card_stmt = app.db.conn.prepare(project_card_query)?;
        let project_card_iter = project_card_stmt.query_map([&self.project_id], |r| {
            Ok(ProjectCard {
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
        for card in project_card_iter {
            let c = card.unwrap();
            let index = project
                .lists
                .iter()
                .position(|l| l.id == c.list_id)
                .unwrap();
            project.lists[index].cards.push(c);
        }

        let card_label_query = "SELECT id, card_id, label_id, created_at, updated_at FROM \
                                card_label WHERE project_id = ?1";
        let mut card_label_stmt = app.db.conn.prepare(card_label_query)?;
        let card_label_iter = card_label_stmt.query_map([&self.project_id], |r| {
            Ok(CardLabel {
                id: r.get(0)?,
                card_id: r.get(1)?,
                label_id: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
            })
        })?;
        for card_label in card_label_iter {
            let label = card_label.unwrap();
            let list_index = project
                .lists
                .iter()
                .position(|l| l.cards.iter().any(|c| c.id == label.card_id))
                .unwrap();
            let card_index = project.lists[list_index]
                .cards
                .iter()
                .position(|c| c.id == label.card_id)
                .unwrap();
            project.lists[list_index].cards[card_index]
                .labels
                .push(label);
        }

        let card_subtask_query = "SELECT id, card_id, value, completed, created_at, updated_at \
                                  FROM card_subtask WHERE project_id = ?1";
        let mut card_subtask_stmt = app.db.conn.prepare(card_subtask_query)?;
        let card_subtask_iter = card_subtask_stmt.query_map([&self.project_id], |r| {
            Ok(CardSubtask {
                id: r.get(0)?,
                card_id: r.get(0)?,
                value: r.get(0)?,
                completed: r.get(0)?,
                created_at: r.get(0)?,
                updated_at: r.get(0)?,
            })
        })?;
        for card_subtask in card_subtask_iter {
            let subtask = card_subtask.unwrap();
            let list_index = project
                .lists
                .iter()
                .position(|l| l.cards.iter().any(|c| c.id == subtask.card_id))
                .unwrap();
            let card_index = project.lists[list_index]
                .cards
                .iter()
                .position(|c| c.id == subtask.card_id)
                .unwrap();
            project.lists[list_index].cards[card_index]
                .subtasks
                .push(subtask);
        }

        if !project.lists.is_empty() && self.selected_list_id.is_none() {
            self.selected_list_id = Some(project.lists[0].id);
            self.focused_pane = FocusedPane::List;
            // TODO: Check if the first list has cards. If so, focus on the
            // first card instead.
        }

        if let Some(list_id) = self.selected_list_id {
            self.popups
                .edit_list
                .set_list(app, list_id)
                .unwrap_or_else(|e| panic!("{e}"));
        }

        self.data = project;

        Ok(())
    }

    fn db_delete_list(&mut self, app: &App, selected_list_id: i32) -> rusqlite::Result<()> {
        struct Select {
            position: i32,
        }
        let select_query = "SELECT position FROM project_list WHERE id = ?1";
        let mut select_stmt = app.db.conn.prepare(select_query)?;
        let select = select_stmt.query_row([self.selected_list_id], |r| {
            Ok(Select {
                position: r.get(0)?,
            })
        })?;

        let query = "DELETE FROM project_list WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        stmt.execute([self.selected_list_id])?;

        let update_position_query =
            "UPDATE project_list SET position = position - 1 WHERE position > ?1";
        let mut update_position_stmt = app.db.conn.prepare(update_position_query)?;
        update_position_stmt.execute([select.position])?;

        let selected_list_index = self
            .data
            .lists
            .iter()
            .position(|l| l.id == selected_list_id)
            .unwrap_or(0);

        if self.data.lists.len() == 1 {
            self.selected_list_id = None;
        } else if selected_list_index != self.data.lists.len() - 1 {
            self.selected_list_id = Some(self.data.lists[selected_list_index + 1].id);
        } else if selected_list_index != 0 {
            self.selected_list_id = Some(self.data.lists[selected_list_index - 1].id);
        } else {
            self.selected_list_id = Some(self.data.lists[0].id);
        }

        Ok(())
    }
}
impl KeyEventHandlerReturn<bool> for OpenProject {
    fn key_event_handler(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
        event_state: &State,
    ) -> bool {
        if app.state.mode == Mode::Popup
            || app.state.mode == Mode::PopupInsert
                && match self.popup {
                    Popup::NewList => {
                        self.popups
                            .new_list
                            .key_event_handler(app, key_event, event_state)
                    }
                    Popup::EditList => {
                        self.popups
                            .edit_list
                            .key_event_handler(app, key_event, event_state)
                    }
                    Popup::None => false,
                }
        {
            self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"))
        }

        if app.state.mode == Mode::Popup && key_event.code == KeyCode::Char('q') {
            self.popup = Popup::None;
        }

        if app.state.mode == Mode::Navigation {
            match key_event.code {
                KeyCode::Char('[') => return true,
                KeyCode::Char('n') => {
                    self.popup = Popup::NewList;
                    app.state.mode = Mode::PopupInsert;
                    // TODO: Create a new card
                }
                KeyCode::Char('e') => {
                    self.popup = Popup::EditList;
                    app.state.mode = Mode::PopupInsert;
                    if let Some(list_id) = self.selected_list_id {
                        self.popups
                            .edit_list
                            .set_list(app, list_id)
                            .unwrap_or_else(|e| panic! {"{e}"});
                    }
                    // TODO: Edit a card
                }
                KeyCode::Char('d') => {
                    app.state.mode = Mode::Delete;
                    // TODO: Delete a card
                }
                KeyCode::Char('h') => {
                    if let Some(list_id) = self.selected_list_id {
                        let list_index = self
                            .data
                            .lists
                            .iter()
                            .position(|l| l.id == list_id)
                            .unwrap();
                        if list_index != 0 {
                            self.selected_list_id = Some(self.data.lists[list_index - 1].id);
                        }
                    }
                }
                KeyCode::Char('l') => {
                    if let Some(list_id) = self.selected_list_id {
                        let list_index = self
                            .data
                            .lists
                            .iter()
                            .position(|l| l.id == list_id)
                            .unwrap();
                        if list_index != self.data.lists.len() - 1 {
                            self.selected_list_id = Some(self.data.lists[list_index + 1].id);
                        }
                    }
                }
                _ => {}
            }
        }

        if app.state.mode == Mode::Delete {
            match key_event.code {
                KeyCode::Char('y') => {
                    if self.focused_pane == FocusedPane::List {
                        if let Some(selected_list_id) = self.selected_list_id {
                            self.db_delete_list(app, selected_list_id)
                                .unwrap_or_else(|e| panic!("{e}"));
                            self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"));
                            app.state.mode = Mode::Navigation;
                        }
                    }
                }
                KeyCode::Char('n') => {
                    app.state.mode = Mode::Navigation;
                }
                _ => {}
            }
        }
        false
    }
}

impl OpenProject {
    fn list_keybinds<'a>(&self) -> Vec<(&'a str, &'a str)> {
        vec![("n", "New List"), ("e", "Edit List"), ("d", "Delete List")]
    }

    fn card_keybinds<'a>(&self) -> Vec<(&'a str, &'a str)> {
        vec![("n", "New Card"), ("e", "Edit Card"), ("d", "Delete Card")]
    }
}

impl ScreenKeybinds for OpenProject {
    fn screen_keybinds<'a>(&self) -> Vec<(&'a str, &'a str)> {
        [self.list_keybinds(), self.card_keybinds()].concat()
    }
}

impl RenderPage<ProjectsState> for OpenProject {
    fn render(
        &mut self,
        app: &mut App,
        frame: &mut Frame,
        area: ratatui::prelude::Rect,
        state: ProjectsState,
    ) {
        let colors = &app.config.colors.clone();
        let block = Block::new()
            .title_top(
                Line::from(format!(" {} ", self.data.title)).style(Style::new().bold().fg(
                    if state.screen_pane != ScreenPane::Tabs && self.data.lists.is_empty() {
                        colors.primary
                    } else {
                        colors.secondary
                    },
                )),
            )
            .title_bottom(pane_title_bottom(
                colors,
                self.list_keybinds(),
                state.screen_pane != ScreenPane::Tabs && self.data.lists.is_empty(),
            ))
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        if self.data.lists.is_empty() {
            let content = Paragraph::new(Text::from(vec![Line::from(vec![
                Span::from("You have no lists in your project. Press "),
                Span::styled("n", Style::new().bold().fg(colors.keybind_key)),
                Span::from(" to create a new list."),
            ])]))
            .block(block.border_style(Style::new().fg(
                if state.screen_pane == ScreenPane::Main {
                    colors.primary
                } else {
                    colors.border
                },
            )));
            frame.render_widget(content, area)
        } else {
            frame.render_widget(block.border_style(Style::new().fg(colors.border)), area);
            let [block_layout] = Layout::default()
                .constraints([Constraint::Min(1)])
                .vertical_margin(1)
                .horizontal_margin(2)
                .areas(area);

            let list_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    self.data
                        .lists
                        .iter()
                        .map(|_| Constraint::Fill(1))
                        .collect::<Vec<Constraint>>(),
                )
                .split(block_layout);

            for (list_index, list_layout) in list_layout.iter().enumerate() {
                let list = &self.data.lists[list_index];
                let list_block = Block::new()
                    .title(Title::from(format!(" {} ", list.title)).alignment(Alignment::Center))
                    .padding(Padding::proportional(1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(
                        if state.screen_pane == ScreenPane::Main
                            && self.focused_pane == FocusedPane::List
                            && self.selected_list_id.is_some_and(|id| id == list.id)
                        {
                            colors.primary
                        } else {
                            colors.border
                        },
                    ));

                let content = Paragraph::new(list.title.to_string()).block(list_block.clone());
                frame.render_widget(content, *list_layout);
            }
        }

        if app.state.mode == Mode::Popup || app.state.mode == Mode::PopupInsert {
            match self.popup {
                Popup::NewList => self.popups.new_list.render(frame, app),
                Popup::EditList => self.popups.edit_list.render(frame, app),
                Popup::None => {}
            }
        }
    }
}

impl OpenProject {
    pub fn set_project_id(&mut self, project_id: i32) {
        self.project_id = Some(project_id);
        self.popups.new_list.set_project_id(project_id);
    }
}
