use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::GlobalPopup, App, Popup, Screen};
use pltx_database::Database;
use pltx_tracing::trace_panic;
use pltx_utils::DateTime;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, BorderType, Borders, Padding, Paragraph, Widget},
    Frame,
};

use super::{card_editor::CardEditor, list_editor::ListEditor, projects::ProjectsState};
use crate::ProjectManagementPane;

#[derive(Clone)]
pub struct ProjectLabel {
    pub id: i32,
    pub title: String,
    pub color: String,
}

#[derive(Clone)]
struct ProjectCardLabel {
    card_id: i32,
    label_id: i32,
}

#[derive(Clone)]
struct ProjectCardSubtask {
    card_id: i32,
    // completed: i32,
}

#[derive(Clone)]
struct OpenProjectCard {
    id: i32,
    list_id: i32,
    title: String,
    important: bool,
    start_date: Option<DateTime>,
    due_date: Option<DateTime>,
    completed: bool,
    // position: i32,
    labels: HashSet<i32>,
    subtasks: Vec<ProjectCardSubtask>,
}

impl OpenProjectCard {
    fn in_progress(&self) -> bool {
        self.start_date.as_ref().is_some_and(|d| d.is_past()) && !self.overdue()
    }

    fn due_soon(&self) -> bool {
        self.due_date.as_ref().is_some_and(|d| d.is_past_days(3)) && !self.overdue()
    }

    fn overdue(&self) -> bool {
        self.due_date.as_ref().is_some_and(|d| d.is_past())
    }
}

#[derive(Clone)]
struct ProjectList {
    id: i32,
    title: String,
    // position: i32,
    cards: Vec<OpenProjectCard>,
}

#[derive(Default, Clone)]
struct ProjectData {
    title: String,
    labels: Vec<ProjectLabel>,
    lists: Vec<ProjectList>,
}

#[derive(PartialEq)]
enum OpenProjectPopup {
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
    popup: OpenProjectPopup,
    popups: Popups,
    delete_selection: DeleteSelection,
    projects_state: Option<ProjectsState>,
}

impl Screen<bool> for OpenProject {
    fn init(app: &App) -> OpenProject {
        OpenProject {
            project_id: None,
            selected_list_id: None,
            selected_card_ids: HashMap::new(),
            data: ProjectData::default(),
            popup: OpenProjectPopup::None,
            popups: Popups {
                new_list: ListEditor::init(app).set_new(),
                edit_list: ListEditor::init(app),
                new_card: CardEditor::init(app).set_new(),
                edit_card: CardEditor::init(app),
            },
            delete_selection: DeleteSelection::None,
            projects_state: None,
        }
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> bool {
        if app.display.is_popup() {
            if self.popup == OpenProjectPopup::NewList {
                if let Some(new_list_id) = self.popups.new_list.key_event_handler(app, key_event) {
                    self.selected_list_id = Some(new_list_id);
                    self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"));
                }
            } else if self.popup == OpenProjectPopup::NewCard {
                if let Some(new_card_id) = self.popups.new_card.key_event_handler(app, key_event) {
                    if let Some(selected_list_id) = self.selected_list_id {
                        self.selected_card_ids
                            .insert(selected_list_id, Some(new_card_id));
                        self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"));
                    }
                }
            }

            if match self.popup {
                OpenProjectPopup::EditList => {
                    self.popups.edit_list.key_event_handler(app, key_event)
                }
                OpenProjectPopup::EditCard => {
                    self.popups.edit_card.key_event_handler(app, key_event)
                }
                _ => None,
            }
            .is_some()
            {
                self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"))
            }
        }

        if app.display.is_popup() && key_event.code == KeyCode::Char('q') {
            app.reset_display();
            self.popup = OpenProjectPopup::None;
        }

        let selected_list_index = self.selected_list_index();
        let selected_card_index = self.selected_card_index();

        if app.display.is_default() {
            let list: Option<&ProjectList> = if let Some(index) = selected_list_index {
                Some(&self.data.lists[index])
            } else {
                None
            };

            match key_event.code {
                KeyCode::Char('[') => return true,
                KeyCode::Char('N') => {
                    self.popup = OpenProjectPopup::NewList;
                    app.popup_display();
                    app.insert_mode();
                }
                KeyCode::Char('n') => {
                    if let Some(project_id) = self.project_id {
                        if let Some(list_id) = self.selected_list_id {
                            self.popups.new_card.ids(project_id, list_id);
                            self.popup = OpenProjectPopup::NewCard;
                            app.popup_display();
                            app.insert_mode();
                        }
                    }
                }
                KeyCode::Char('E') => {
                    if let Some(list_id) = self.selected_list_id {
                        self.popup = OpenProjectPopup::EditList;
                        self.popups
                            .edit_list
                            .set(&app.db, list_id)
                            .unwrap_or_else(|e| panic! {"{e}"});
                        app.popup_display();
                        app.insert_mode();
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(project_id) = self.project_id {
                        if let Some(list_id) = self.selected_list_id {
                            if let Some(card_id) = self.selected_card_id() {
                                self.popups.edit_card.ids(project_id, list_id);
                                self.popup = OpenProjectPopup::EditCard;
                                self.popups
                                    .edit_card
                                    .set_data(&app.db, card_id)
                                    .unwrap_or_else(|e| panic!("{e}"));
                                app.popup_display();
                            }
                        }
                    }
                }
                KeyCode::Char('c') => {
                    if let Some(card_id) = self.selected_card_id() {
                        let list_index = self.selected_list_index().unwrap_or(0);
                        let card_index = self.selected_card_index().unwrap_or(0);
                        let completed = &self.data.lists[list_index].cards[card_index].completed;
                        self.db_toggle_card_completed(&app.db, card_id, *completed)
                            .unwrap();
                        self.db_get_project(app).unwrap();
                    }
                }
                KeyCode::Char('i') => {
                    if let Some(card_id) = self.selected_card_id() {
                        let list_index = self.selected_list_index().unwrap_or(0);
                        let card_index = self.selected_card_index().unwrap_or(0);
                        let important = &self.data.lists[list_index].cards[card_index].important;
                        self.db_toggle_card_important(&app.db, card_id, *important)
                            .unwrap();
                        self.db_get_project(app).unwrap();
                    }
                }
                KeyCode::Char('D') => {
                    if self.project_id.is_some() && self.selected_list_id.is_some() {
                        self.delete_selection = DeleteSelection::List;
                        app.delete_mode();
                    }
                }
                KeyCode::Char('d') => {
                    if self.selected_card_id().is_some() {
                        self.delete_selection = DeleteSelection::Card;
                        app.delete_mode();
                    }
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

        if app.is_delete_mode() {
            match key_event.code {
                KeyCode::Char('y') => {
                    if self.delete_selection == DeleteSelection::List {
                        if let Some(selected_list_id) = self.selected_list_id {
                            self.db_delete_list(&app.db, selected_list_id)
                                .unwrap_or_else(|e| panic!("{e}"));
                            self.db_get_project(app).unwrap_or_else(|e| panic!("{e}"));
                            app.normal_mode();
                        }
                    } else if self.delete_selection == DeleteSelection::Card {
                        if let Some(selected_card_id) = self.selected_card_id() {
                            self.db_delete_card(&app.db, selected_card_id)
                                .unwrap_or_else(|e| trace_panic!("{e}"));
                            self.db_get_project(app)
                                .unwrap_or_else(|e| trace_panic!("{e}"));
                            app.normal_mode();
                        }
                    }
                    self.delete_selection = DeleteSelection::None;
                }
                KeyCode::Char('n') => app.normal_mode(),

                _ => {}
            }
        }
        false
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors.clone();
        let block = Block::new()
            .title_top(
                Line::from(format!(" {} ", self.data.title))
                    .style(Style::new().bold().fg(colors.fg)),
            )
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        if self.data.lists.is_empty() {
            let content = Paragraph::new(Text::from(vec![Line::from(vec![
                Span::from("You have no lists in your project. Press "),
                Span::styled("N", Style::new().bold().fg(colors.keybind_key)),
                Span::from(" to create a new list."),
            ])]))
            .block(
                block.border_style(
                    Style::new().fg(
                        if self
                            .projects_state
                            .as_ref()
                            .is_some_and(|s| s.module_pane == ProjectManagementPane::Main)
                        {
                            colors.primary
                        } else {
                            colors.border
                        },
                    ),
                ),
            );
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
                let list = self.render_list(app, *list_layout, list_index);
                frame.render_widget(list.0, list.1);
            }
        }

        if app.display.is_popup() && app.popup == GlobalPopup::None {
            match self.popup {
                OpenProjectPopup::NewList => self.popups.new_list.render(app, frame, area),
                OpenProjectPopup::EditList => self.popups.edit_list.render(app, frame, area),
                OpenProjectPopup::NewCard => self.popups.new_card.render(app, frame, area),
                OpenProjectPopup::EditCard => self.popups.edit_card.render(app, frame, area),
                OpenProjectPopup::None => {}
            }
        }
    }
}

impl OpenProject {
    pub fn projects_state(&mut self, projects_state: ProjectsState) {
        self.projects_state = Some(projects_state);
    }

    pub fn db_get_project(&mut self, app: &mut App) -> Result<(), &str> {
        let conn = app.db.conn();
        let query = "SELECT title, description, position, created_at, updated_at FROM project \
                     WHERE id = ?1 ORDER BY position";
        let mut stmt = conn.prepare(query).unwrap();

        if let Some(project_id) = self.project_id {
            let mut project = stmt
                .query_row([project_id], |r| {
                    Ok(ProjectData {
                        title: r.get(0)?,
                        labels: vec![],
                        lists: vec![],
                    })
                })
                .unwrap();

            project.labels = self.db_get_labels(app).unwrap();
            project.lists = self.db_get_lists(&app.db, project_id).unwrap();
            project = self
                .db_get_cards(&app.db, &mut project, project_id)
                .unwrap();
            project = self
                .db_get_card_labels(&app.db, &mut project, project_id)
                .unwrap();
            project = self
                .db_get_card_subtasks(&app.db, &mut project, project_id)
                .unwrap();

            if !project.lists.is_empty() {
                if self.selected_list_id.is_none() {
                    self.selected_list_id = Some(project.lists[0].id);
                }

                for list in project.lists.clone() {
                    if list.cards.is_empty() {
                        self.selected_card_ids.entry(list.id).or_insert(None);
                    } else {
                        self.selected_card_ids
                            .entry(list.id)
                            .or_insert(Some(list.cards[0].id));
                    }
                }
            }

            if let Some(list_id) = self.selected_list_id {
                self.popups.edit_list.set(&app.db, list_id).unwrap();

                if let Some(project_id) = self.project_id {
                    self.popups.new_card.ids(project_id, list_id);
                    self.popups.edit_card.ids(project_id, list_id);
                }
            }

            self.data = project;
        }

        Ok(())
    }

    fn db_get_labels(&mut self, app: &App) -> rusqlite::Result<Vec<ProjectLabel>> {
        let mut labels = vec![];

        let conn = app.db.conn();
        let project_label_query = "SELECT id, title, color, position, created_at, updated_at FROM \
                                   project_label WHERE project_id = ?1 ORDER BY position";
        let mut project_label_stmt = conn.prepare(project_label_query)?;

        let project_label_iter = project_label_stmt.query_map([&self.project_id], |r| {
            Ok(ProjectLabel {
                id: r.get(0)?,
                title: r.get(1)?,
                color: r.get(2)?,
            })
        })?;

        for l in project_label_iter {
            let label = l.unwrap();
            labels.push(label);
        }

        self.popups
            .new_card
            .labels(&app.config.colors, labels.clone());
        self.popups
            .edit_card
            .labels(&app.config.colors, labels.clone());

        Ok(labels)
    }

    fn db_get_lists(&self, db: &Database, project_id: i32) -> rusqlite::Result<Vec<ProjectList>> {
        let mut lists = vec![];

        let conn = db.conn();
        let query =
            "SELECT id, title, position FROM project_list WHERE project_id = ?1 ORDER BY position";
        let mut stmt = conn.prepare(query)?;
        let project_list_iter = stmt.query_map([project_id], |r| {
            Ok(ProjectList {
                id: r.get(0)?,
                title: r.get(1)?,
                // position: r.get(2)?,
                cards: vec![],
            })
        })?;
        for list in project_list_iter {
            lists.push(list.unwrap())
        }
        Ok(lists)
    }

    fn db_get_cards(
        &self,
        db: &Database,
        project: &mut ProjectData,
        project_id: i32,
    ) -> rusqlite::Result<ProjectData> {
        let conn = db.conn();
        let project_card_query = "SELECT id, list_id, title, important, start_date, due_date, \
                                  completed, position FROM project_card WHERE project_id = ?1 \
                                  ORDER BY position";
        let mut project_card_stmt = conn.prepare(project_card_query)?;
        let project_card_iter = project_card_stmt.query_map([project_id], |r| {
            Ok(OpenProjectCard {
                id: r.get(0)?,
                list_id: r.get(1)?,
                title: r.get(2)?,
                important: r.get(3)?,
                start_date: DateTime::from_db_option(r.get(4)?),
                due_date: DateTime::from_db_option(r.get(5)?),
                completed: r.get(6)?,
                // position: r.get(7)?,
                labels: HashSet::new(),
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
        db: &Database,
        project: &mut ProjectData,
        project_id: i32,
    ) -> rusqlite::Result<ProjectData> {
        let conn = db.conn();
        let card_label_query = "SELECT card_id, label_id FROM card_label WHERE project_id = ?1";
        let mut card_label_stmt = conn.prepare(card_label_query)?;
        let card_label_iter = card_label_stmt.query_map([project_id], |r| {
            Ok(ProjectCardLabel {
                card_id: r.get(0)?,
                label_id: r.get(1)?,
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
                .insert(label.label_id);
        }

        Ok(project.clone())
    }

    fn db_get_card_subtasks(
        &self,

        db: &Database,
        project: &mut ProjectData,
        project_id: i32,
    ) -> rusqlite::Result<ProjectData> {
        let conn = db.conn();
        let card_subtask_query =
            "SELECT card_id, completed FROM card_subtask WHERE project_id = ?1";
        let mut card_subtask_stmt = conn.prepare(card_subtask_query)?;
        let card_subtask_iter = card_subtask_stmt.query_map([project_id], |r| {
            Ok(ProjectCardSubtask {
                card_id: r.get(0)?,
                // completed: r.get(0)?,
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

    fn db_delete_list(&mut self, db: &Database, selected_list_id: i32) -> rusqlite::Result<()> {
        let original_position = db.get_position("project_list", selected_list_id)?;

        let conn = db.conn();
        let query = "DELETE FROM project_list WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        stmt.execute([selected_list_id])?;

        db.update_positions("project_list", original_position)?;

        // Update the position of `selected_list_id` before `data` is updated.
        let selected_list_index = self.selected_list_index().unwrap_or(0);
        if self.data.lists.len() == 1 {
            self.selected_list_id = None;
        } else if selected_list_index != self.data.lists.len().saturating_sub(1) {
            self.selected_list_id = Some(self.data.lists[selected_list_index + 1].id);
        } else if selected_list_index != 0 {
            self.selected_list_id = Some(self.data.lists[selected_list_index.saturating_sub(1)].id);
        }

        Ok(())
    }

    fn db_delete_card(&mut self, db: &Database, selected_card_id: i32) -> rusqlite::Result<()> {
        let original_position = db.get_position("project_card", selected_card_id)?;

        let conn = db.conn();
        let query = "DELETE FROM project_card WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        stmt.execute([selected_card_id])?;

        db.update_positions("project_card", original_position)?;

        let list_index = self.selected_list_index().unwrap_or(0);
        let list = &self.data.lists[list_index];
        let selected_card_index = self.selected_card_index().unwrap_or(0);

        // Update the position of the selected card before `data` is updated.
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
        }

        Ok(())
    }

    fn db_toggle_card_completed(
        &mut self,
        db: &Database,
        card_id: i32,
        completed: bool,
    ) -> rusqlite::Result<()> {
        let conn = db.conn();
        let query = "UPDATE project_card SET completed = ?1, updated_at = ?2 WHERE id = ?3";
        let mut stmt = conn.prepare(query)?;
        stmt.execute((!completed, DateTime::now(), card_id))?;
        Ok(())
    }

    fn db_toggle_card_important(
        &mut self,
        db: &Database,
        card_id: i32,
        important: bool,
    ) -> rusqlite::Result<()> {
        let conn = db.conn();
        let query = "UPDATE project_card SET important = ?1, updated_at = ?2 WHERE id = ?3";
        let mut stmt = conn.prepare(query)?;
        stmt.execute((!important, DateTime::now(), card_id))?;
        Ok(())
    }
}

impl OpenProject {
    fn render_list(&self, app: &App, layout: Rect, index: usize) -> (impl Widget, Rect) {
        let colors = &app.config.colors;

        let list_width = layout.width as usize - 2;
        let list = &self.data.lists[index];
        let selected_list = self
            .projects_state
            .as_ref()
            .is_some_and(|s| s.module_pane == ProjectManagementPane::Main)
            && self.selected_list_id.is_some_and(|id| id == list.id);

        let list_block = Block::new()
            .title(Title::from(format!(" {} ", list.title)).alignment(Alignment::Center))
            .title_style(Style::new().fg(colors.fg))
            .padding(if list.cards.is_empty() {
                Padding::proportional(1)
            } else {
                Padding::zero()
            })
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
                let card = self.render_card(app, card, index, list_width, selected_list);
                text.push(card.0);
                text.push(card.1);
                text.push(Line::from(""));
            }
        }

        let content = Paragraph::new(Text::from(text)).block(list_block.clone());

        (content, layout)
    }

    fn render_card<'a>(
        &self,
        app: &App,
        card: &OpenProjectCard,
        list_index: usize,
        list_width: usize,
        selected_list: bool,
    ) -> (Line<'a>, Line<'a>) {
        let colors = &app.config.colors;

        let selected = self.selected_card_id() == Some(card.id);
        let unfocused_selected = self.selected_list_id.is_some_and(|list_id| {
            list_id != self.data.lists[list_index].id
                && self
                    .selected_card_ids
                    .get(&self.data.lists[list_index].id)
                    .is_some_and(|id| id == &Some(card.id))
        });

        let config = &app.config.modules.project_management;
        let status_char = if card.completed {
            &config.completed_char
        } else if card.overdue() {
            &config.overdue_char
        } else if card.due_soon() {
            &config.due_soon_char
        } else if card.in_progress() {
            &config.in_progress_char
        } else if card.important {
            &config.important_char
        } else {
            &config.default_char
        };

        let line_style = if selected_list && selected {
            Style::new().bold().fg(colors.bg).bg(colors.fg)
        } else if unfocused_selected {
            Style::new().bold().fg(colors.fg)
        } else {
            Style::new().fg(colors.secondary)
        };

        let title = Line::from(vec![
            Span::from(format!(" [{}] ", status_char)).fg(if selected_list && selected {
                colors.bg
            } else {
                colors.secondary
            }),
            Span::from(card.title.to_string()).fg(if selected_list && selected {
                colors.bg
            } else {
                colors.fg
            }),
            Span::from(" ".repeat(list_width.saturating_sub(card.title.chars().count() + 2))),
        ])
        .style(line_style);

        let mut details = vec![Span::from(" Labels: ")];

        for label in card.labels.iter() {
            details.push(
                Span::from("⬤ ").fg(Color::from_str(
                    &self
                        .data
                        .labels
                        .iter()
                        .find(|l| label == &l.id)
                        .unwrap()
                        .color,
                )
                .unwrap()),
            );
        }

        details.push(Span::from(" ".repeat(list_width.saturating_sub(
            card.labels.len() + if card.labels.is_empty() { 1 } else { 2 },
        ))));

        let details_line = Line::from(details).style(line_style);

        (title, details_line)
    }

    pub fn set_project_id(&mut self, project_id: i32) {
        self.project_id = Some(project_id);
        self.popups.new_list.project_id(project_id);
        self.popups.edit_list.project_id(project_id);
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
            self.data.lists[index]
                .cards
                .iter()
                .position(|p| self.selected_card_id() == Some(p.id))
        } else {
            None
        }
    }

    pub fn reset(&mut self, app: &mut App) {
        self.project_id = None;
        self.selected_list_id = None;
        self.selected_card_ids = HashMap::new();
        self.data = ProjectData::default();
        self.popup = OpenProjectPopup::None;
        self.popups.new_list.reset(app);
        self.popups.edit_list.reset(app);
        self.popups.new_card.reset();
        self.popups.edit_card.reset();
        self.delete_selection = DeleteSelection::None;
    }
}
