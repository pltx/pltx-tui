use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use super::{
    card_editor::CardEditor, list_editor::ListEditor, projects::ProjectsState, screen::ScreenPane,
};
use crate::{
    state::{GlobalPopup, Mode, State},
    trace_panic,
    utils::{
        pane_title_bottom, Init, InitData, KeyEventHandlerReturn, RenderPage, RenderPopup,
        RenderPopupContained, ScreenKeybinds,
    },
    App,
};

#[derive(Clone)]
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
    label_id: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Clone)]
struct CardSubtask {
    id: i32,
    card_id: i32,
    value: String,
    completed: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Clone)]
struct ProjectCard {
    id: i32,
    list_id: i32,
    title: String,
    description: Option<String>,
    important: i32,
    due_date: Option<String>,
    reminder: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
    labels: Vec<CardLabel>,
    subtasks: Vec<CardSubtask>,
}

#[derive(Clone)]
struct ProjectList {
    id: i32,
    title: String,
    color: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
    cards: Vec<ProjectCard>,
}

#[derive(Default, Clone)]
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
    NewCard,
    EditCard,
    None,
}

struct Popups {
    new_list: ListEditor,
    edit_list: ListEditor,
    new_card: CardEditor,
    edit_card: CardEditor,
}

#[derive(PartialEq)]
enum FocusedPane {
    List,
    Card,
    None,
}

#[derive(PartialEq)]
enum DeleteSelection {
    List,
    Card,
    None,
}

pub struct OpenProject {
    project_id: Option<i32>,
    selected_list_id: Option<i32>,
    selected_card_ids: HashMap<i32, Option<i32>>,
    data: ProjectData,
    popup: Popup,
    popups: Popups,
    delete_selection: DeleteSelection,
}

impl Init for OpenProject {
    fn init(app: &mut App) -> OpenProject {
        OpenProject {
            project_id: None,
            selected_list_id: None,
            selected_card_ids: HashMap::new(),
            data: ProjectData::default(),
            popup: Popup::None,
            popups: Popups {
                new_list: ListEditor::init(app).empty(),
                edit_list: ListEditor::init(app),
                new_card: CardEditor::init(app).empty(),
                edit_card: CardEditor::init(app),
            },
            delete_selection: DeleteSelection::None,
        }
    }
}

impl InitData for OpenProject {
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()> {
        self.db_get_project(app);
        Ok(())
    }
}

impl OpenProject {
    pub fn db_get_project(&mut self, app: &mut App) -> Result<(), &str> {
        let query = "SELECT id, title, description, position, created_at, updated_at FROM project \
                     WHERE id = ?1 ORDER BY position";
        let mut stmt = app.db.conn.prepare(query).unwrap();
        let mut project = stmt
            .query_row([&self.project_id], |r| {
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
            })
            .unwrap();
        project.labels = self.db_get_labels(app).unwrap();
        project.lists = self.db_get_lists(app).unwrap();
        project = self.db_get_cards(app, &mut project).unwrap();
        project = self.db_get_card_labels(app, &mut project).unwrap();
        project = self.db_get_card_subtasks(app, &mut project).unwrap();

        if !project.lists.is_empty() && self.selected_list_id.is_none() {
            self.selected_list_id = Some(project.lists[0].id);

            for list in project.lists.clone() {
                if list.cards.is_empty() {
                    self.selected_card_ids.insert(list.id, None);
                } else {
                    self.selected_card_ids
                        .insert(list.id, Some(list.cards[0].id));
                }
            }
        }

        if let Some(list_id) = self.selected_list_id {
            self.popups.edit_list.set(app, list_id).unwrap();

            if let Some(project_id) = self.project_id {
                self.popups.new_card.ids(project_id, list_id);
                self.popups.edit_card.ids(project_id, list_id);
            }

            if let Some(card_id) = self.selected_card_id() {
                self.popups
                    .edit_card
                    .set(app, card_id)
                    .unwrap_or_else(|e| trace_panic!("{e}"));
            }
        }

        self.data = project;

        Ok(())
    }

    fn db_get_labels(&self, app: &App) -> rusqlite::Result<Vec<ProjectLabel>> {
        let mut labels = vec![];
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
            labels.push(label.unwrap());
        }
        Ok(labels)
    }

    fn db_get_lists(&self, app: &App) -> rusqlite::Result<Vec<ProjectList>> {
        let mut lists = vec![];
        let query = "SELECT id, title, color, position, created_at, updated_at FROM project_list \
                     WHERE project_id = ?1 ORDER BY position";
        let mut stmt = app.db.conn.prepare(query)?;
        let project_list_iter = stmt.query_map([&self.project_id], |r| {
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
            lists.push(list.unwrap())
        }
        Ok(lists)
    }

    fn db_get_cards(&self, app: &App, project: &mut ProjectData) -> rusqlite::Result<ProjectData> {
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
        Ok(project.clone())
    }

    fn db_get_card_labels(
        &self,
        app: &App,
        project: &mut ProjectData,
    ) -> rusqlite::Result<ProjectData> {
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
        Ok(project.clone())
    }

    fn db_get_card_subtasks(
        &self,
        app: &App,
        project: &mut ProjectData,
    ) -> rusqlite::Result<ProjectData> {
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
        Ok(project.clone())
    }

    fn db_delete_list(&mut self, app: &App, selected_list_id: i32) -> rusqlite::Result<()> {
        let original_position = app.db.get_position("project_list", selected_list_id)?;

        let query = "DELETE FROM project_list WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        stmt.execute([self.selected_list_id])?;

        app.db.update_positions("project_list", original_position)?;

        // Update the position of `selected_list_id` before `data` is updated.
        let selected_list_index = self.selected_list_index().unwrap_or(0);
        if self.data.lists.len() == 1 {
            self.selected_list_id = None;
        } else if selected_list_index != self.data.lists.len().saturating_sub(1) {
            self.selected_list_id = Some(self.data.lists[selected_list_index + 1].id);
        } else if selected_list_index != 0 {
            self.selected_list_id = Some(self.data.lists[selected_list_index.saturating_sub(1)].id);
        } else {
            self.selected_list_id = Some(self.data.lists[0].id);
        }

        Ok(())
    }

    fn db_delete_card(&mut self, app: &mut App, selected_card_id: i32) -> rusqlite::Result<()> {
        let original_position = app.db.get_position("project_card", selected_card_id)?;

        let query = "DELETE FROM project_card WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        stmt.execute([selected_card_id])?;

        app.db.update_positions("project_card", original_position)?;

        // Update the position of the selected card before `data` is updated.
        let list_index = self.selected_list_index().unwrap_or(0);
        let list = &self.data.lists[list_index];
        let selected_card_index = self.selected_card_index().unwrap_or(0);

        if list.cards.len() == 1 {
            self.selected_card_ids.insert(list.id, None);
        } else if selected_card_index != list.cards.len().saturating_sub(1) {
            self.selected_card_ids
                .insert(list.id, Some(list.cards[selected_card_index + 1].id));
        } else if selected_card_index != 0 {
            self.selected_card_ids.insert(
                list.id,
                Some(list.cards[selected_card_index.saturating_sub(1)].id),
            );
        } else {
            self.selected_card_ids
                .insert(list.id, Some(list.cards[0].id));
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
        if (app.state.mode == Mode::Popup || app.state.mode == Mode::PopupInsert)
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
                Popup::NewCard => {
                    self.popups
                        .new_card
                        .key_event_handler(app, key_event, event_state)
                }
                Popup::EditCard => {
                    self.popups
                        .edit_card
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

        let selected_list_index = self.selected_list_index();
        let selected_card_index = self.selected_card_index();

        if app.state.mode == Mode::Navigation {
            let list: Option<&ProjectList> = if let Some(index) = selected_list_index {
                Some(&self.data.lists[index])
            } else {
                None
            };

            match key_event.code {
                KeyCode::Char('[') => return true,
                KeyCode::Char('N') => {
                    self.popup = Popup::NewList;
                    app.state.mode = Mode::PopupInsert;
                }
                KeyCode::Char('n') => {
                    if let Some(project_id) = self.project_id {
                        if let Some(list_id) = self.selected_list_id {
                            self.popups.new_card.ids(project_id, list_id);
                            self.popup = Popup::NewCard;
                            app.state.mode = Mode::PopupInsert;
                        }
                    }
                }
                KeyCode::Char('E') => {
                    if let Some(list_id) = self.selected_list_id {
                        self.popup = Popup::EditList;
                        app.state.mode = Mode::PopupInsert;
                        self.popups
                            .edit_list
                            .set(app, list_id)
                            .unwrap_or_else(|e| panic! {"{e}"});
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(project_id) = self.project_id {
                        if let Some(list_id) = self.selected_list_id {
                            if let Some(card_id) = self.selected_card_id() {
                                self.popups.edit_card.ids(project_id, list_id);
                                self.popup = Popup::EditCard;
                                app.state.mode = Mode::PopupInsert;
                                self.popups
                                    .edit_card
                                    .set(app, card_id)
                                    .unwrap_or_else(|e| panic!("{e}"))
                            }
                        }
                    }
                }
                KeyCode::Char('D') => {
                    self.delete_selection = DeleteSelection::List;
                    app.state.mode = Mode::Delete;
                }
                KeyCode::Char('d') => {
                    self.delete_selection = DeleteSelection::Card;
                    app.state.mode = Mode::Delete;
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
                            self.selected_list_id =
                                Some(self.data.lists[list_index.saturating_sub(1)].id);
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
                        if list_index != self.data.lists.len().saturating_sub(1) {
                            self.selected_list_id = Some(self.data.lists[list_index + 1].id);
                        }
                    }
                }
                KeyCode::Char('j') => {
                    if let Some(list) = list {
                        if let Some(selected_list_id) = self.selected_list_id {
                            if let Some(selected_card_index) = selected_card_index {
                                if selected_card_index != list.cards.len().saturating_sub(1) {
                                    self.selected_card_ids.insert(
                                        selected_list_id,
                                        Some(list.cards[selected_card_index.saturating_add(1)].id),
                                    );
                                } else {
                                    self.selected_card_ids
                                        .insert(selected_list_id, Some(list.cards[0].id));
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(list) = list {
                        if let Some(selected_list_id) = self.selected_list_id {
                            if let Some(selected_card_index) = selected_card_index {
                                if selected_card_index != 0 {
                                    self.selected_card_ids.insert(
                                        selected_list_id,
                                        Some(list.cards[selected_card_index.saturating_sub(1)].id),
                                    );
                                } else {
                                    self.selected_card_ids.insert(
                                        selected_list_id,
                                        Some(list.cards[list.cards.len().saturating_sub(1)].id),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if app.state.mode == Mode::Delete {
            match key_event.code {
                KeyCode::Char('y') => {
                    if self.delete_selection == DeleteSelection::List {
                        if let Some(selected_list_id) = self.selected_list_id {
                            self.db_delete_list(app, selected_list_id)
                                .unwrap_or_else(|e| panic!("{e}"));
                            self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"));
                            app.state.mode = Mode::Navigation;
                        }
                    } else if self.delete_selection == DeleteSelection::Card {
                        if let Some(selected_card_id) = self.selected_card_id() {
                            self.db_delete_card(app, selected_card_id)
                                .unwrap_or_else(|e| trace_panic!("{e}"));
                            self.db_get_project(app)
                                .unwrap_or_else(|e| trace_panic!("{e}"));
                            app.state.mode = Mode::Navigation;
                        }
                    }
                    self.delete_selection = DeleteSelection::None;
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
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect, state: ProjectsState) {
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
                Span::styled("N", Style::new().bold().fg(colors.keybind_key)),
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

            let project_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    self.data
                        .lists
                        .iter()
                        .map(|_| Constraint::Fill(1))
                        .collect::<Vec<Constraint>>(),
                )
                .split(block_layout);

            for (list_index, list_layout) in project_layout.iter().enumerate() {
                let list = &self.data.lists[list_index];
                let selected_list = state.screen_pane == ScreenPane::Main
                    && self.selected_list_id.is_some_and(|id| id == list.id);

                let list_block = Block::new()
                    .title(Title::from(format!(" {} ", list.title)).alignment(Alignment::Center))
                    .padding(Padding::proportional(1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(if selected_list {
                        colors.primary
                    } else {
                        colors.border
                    }));

                let mut text = vec![];

                if list.cards.is_empty() {
                    text.push(Line::from(vec![
                        Span::from("There are no tasks in this list. Press "),
                        Span::from("n").bold().fg(colors.keybind_key),
                        Span::from(" to create a new task."),
                    ]));
                } else {
                    for card in list.cards.iter() {
                        let selected = self.selected_card_id() == Some(card.id);
                        let unfocused_selected = self.selected_list_id.is_some_and(|list_id| {
                            list_id != self.data.lists[list_index].id
                                && self
                                    .selected_card_ids
                                    .get(&self.data.lists[list_index].id)
                                    .is_some_and(|id| id == &Some(card.id))
                        });
                        text.push(
                            Line::from(vec![Span::from(format!(" {} ", card.title))]).style(
                                if selected_list && selected {
                                    Style::new()
                                        .bold()
                                        .fg(colors.active_fg)
                                        .bg(colors.active_bg)
                                } else if unfocused_selected {
                                    Style::new().bold().fg(colors.bg).bg(colors.secondary)
                                } else {
                                    Style::new().fg(colors.fg)
                                },
                            ),
                        );
                        text.push(Line::from(format!(" {} ", card.id)));
                        text.push(Line::from(""))
                    }
                }

                let content = Paragraph::new(Text::from(text)).block(list_block.clone());
                frame.render_widget(content, *list_layout);
            }
        }

        if (app.state.mode == Mode::Popup || app.state.mode == Mode::PopupInsert)
            && app.state.popup == GlobalPopup::None
        {
            match self.popup {
                Popup::NewList => self.popups.new_list.render(frame, app, area),
                Popup::EditList => self.popups.edit_list.render(frame, app, area),
                Popup::NewCard => self.popups.new_card.render(frame, app, area),
                Popup::EditCard => self.popups.edit_card.render(frame, app, area),
                Popup::None => {}
            }
        }
    }
}

impl OpenProject {
    pub fn set_project_id(&mut self, project_id: i32) {
        self.project_id = Some(project_id);
        self.popups.new_list.project_id(project_id);
    }

    fn selected_card_id(&self) -> Option<i32> {
        if let Some(selected_list_id) = self.selected_list_id {
            if let Some(selected_card_id) = self.selected_card_ids.get(&selected_list_id) {
                *selected_card_id
            } else {
                None
            }
        } else {
            None
        }
    }

    fn selected_list_index(&self) -> Option<usize> {
        self.data
            .lists
            .iter()
            .position(|p| self.selected_list_id.is_some_and(|id| id == p.id))
    }

    fn selected_card_index(&self) -> Option<usize> {
        if let Some(index) = self.selected_list_index() {
            self.data.lists[index].cards.iter().position(|p| {
                if let Some(scid) = self.selected_card_id() {}
                self.selected_card_id() == Some(p.id)
            })
        } else {
            None
        }
    }
}
