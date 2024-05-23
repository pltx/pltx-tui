use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Mode, App, Screen};
use pltx_database::Database;
use pltx_utils::DateTime;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table},
    Frame,
};

use super::projects::ProjectsState;
use crate::ProjectManagementPane;

#[derive(Clone)]
struct Project {
    id: i32,
    title: String,
    description: Option<String>,
    position: i32,
    created_at: DateTime,
    updated_at: DateTime,
    labels: i32,
    lists: i32,
    total_cards: i32,
    cards_in_progress: i32,
    cards_due_soon: i32,
    cards_overdue: i32,
    important_cards: i32,
}

pub struct ListProjects {
    projects: Vec<Project>,
    pub selected_id: Option<i32>,
    projects_state: Option<ProjectsState>,
}

impl Screen<bool> for ListProjects {
    fn init(app: &App) -> ListProjects {
        let mut list_projects = ListProjects {
            projects: vec![],
            selected_id: None,
            projects_state: None,
        };

        list_projects.db_get_projects(app).unwrap();

        list_projects
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> bool {
        if let Some(selected_id) = self.selected_id {
            if app.is_normal_mode() {
                let selected_index = self
                    .projects
                    .iter()
                    .position(|p| p.id == selected_id)
                    .unwrap_or(0);

                match key_event.code {
                    KeyCode::Char('d') => {
                        app.delete_mode();
                    }
                    KeyCode::Char('j') => {
                        if selected_index != self.projects.len().saturating_sub(1) {
                            self.selected_id = Some(self.projects[selected_index + 1].id);
                        } else {
                            self.selected_id = Some(self.projects[0].id);
                        }
                    }
                    KeyCode::Char('k') => {
                        if selected_index != 0 {
                            self.selected_id =
                                Some(self.projects[selected_index.saturating_sub(1)].id);
                        } else {
                            self.selected_id =
                                Some(self.projects[self.projects.len().saturating_sub(1)].id);
                        }
                    }
                    _ => {}
                }
            }

            if app.mode() == Mode::Delete {
                match key_event.code {
                    KeyCode::Char('y') => {
                        self.db_delete_project(&app.db)
                            .unwrap_or_else(|e| panic!("{e}"));
                        self.db_get_projects(app).unwrap_or_else(|e| panic!("{e}"));
                        app.reset_display();
                    }
                    KeyCode::Char('n') => app.reset_display(),
                    _ => {}
                }
            }
        }
        false
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let [list_layout, info_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(40)])
            .areas(area);

        let colors = &app.config.colors.clone();
        let block = Block::new()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(
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
            );

        if let Some(selected_id) = self.selected_id {
            let rows = self
                .projects
                .iter()
                .enumerate()
                .map(|(_, p)| {
                    Row::new(vec![
                        Cell::new(format!(" {}", p.position)),
                        Cell::new(p.title.clone()),
                        Cell::new(p.total_cards.to_string()),
                        Cell::new(p.cards_due_soon.to_string()),
                        Cell::new(p.cards_in_progress.to_string()),
                        Cell::new(p.cards_overdue.to_string()),
                        Cell::new(p.important_cards.to_string()),
                    ])
                    .style(if self.selected_id.is_some_and(|id| id == p.id) {
                        Style::new()
                            .bold()
                            .fg(colors.active_fg)
                            .bg(colors.active_bg)
                    } else {
                        Style::new().fg(colors.fg).bg(colors.bg)
                    })
                })
                .collect::<Vec<Row>>();

            let widths = vec![
                Constraint::Length(5),
                Constraint::Max(50),
                Constraint::Length(7),
                Constraint::Length(13),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(8),
            ];
            let table = Table::new(rows, widths).block(block).header(
                Row::new(vec![
                    Cell::new(" "),
                    Cell::new("Title"),
                    Cell::new("Cards"),
                    Cell::new("In Progress"),
                    Cell::new("Due Soon"),
                    Cell::new("Overdue"),
                    Cell::new("Important"),
                ])
                .style(Style::new().bold().fg(colors.primary)),
            );
            frame.render_widget(table, list_layout);

            let project_index = self
                .projects
                .iter()
                .position(|p| p.id == selected_id)
                .unwrap();
            let project = &self.projects[project_index];

            let info_1 = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::new().fg(colors.secondary)),
                    Span::from(project.id.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Title: ", Style::new().fg(colors.secondary)),
                    Span::from(&project.title),
                ]),
            ];
            let line_length = info_layout.width as usize - 6;
            let mut first_line_length = line_length - "Description: ".chars().count();
            let description = if let Some(desc) = &project.description {
                if desc.chars().count() <= first_line_length {
                    first_line_length = 0;
                }
                desc[first_line_length..]
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(line_length)
                    .enumerate()
                    .flat_map(|(i, c)| {
                        let mut text = vec![];
                        if i == 0 {
                            text.push(Line::from(vec![
                                Span::styled("Description: ", Style::new().fg(colors.secondary)),
                                Span::from(desc[..first_line_length].to_string()),
                            ]))
                        }
                        text.push(Line::from(Span::from(
                            c.iter().collect::<String>().trim().to_owned(),
                        )));
                        text
                    })
                    .collect::<Vec<Line>>()
            } else {
                vec![Line::from(vec![
                    Span::styled("Description: ", Style::new().fg(colors.secondary)),
                    Span::styled("<empty>", Style::new().fg(colors.secondary)),
                ])]
            };
            let info_2 = vec![
                Line::from(vec![
                    Span::styled("Position: ", Style::new().fg(colors.secondary)),
                    Span::from(project.position.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Labels: ", Style::new().fg(colors.secondary)),
                    Span::from(project.labels.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Lists: ", Style::new().fg(colors.secondary)),
                    Span::from(project.lists.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards: ", Style::new().fg(colors.secondary)),
                    Span::from(project.total_cards.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards In Progress: ", Style::new().fg(colors.secondary)),
                    Span::from(project.cards_in_progress.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards Due Soon: ", Style::new().fg(colors.secondary)),
                    Span::from(project.cards_due_soon.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards Overdue: ", Style::new().fg(colors.secondary)),
                    Span::from(project.cards_overdue.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Important Cards: ", Style::new().fg(colors.secondary)),
                    Span::from(project.important_cards.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Created At: ", Style::new().fg(colors.secondary)),
                    Span::from(project.created_at.display()),
                ]),
                Line::from(vec![
                    Span::styled("Updated At: ", Style::new().fg(colors.secondary)),
                    Span::from(project.updated_at.display()),
                ]),
            ];
            let info_text = Text::from([info_1, description, info_2].concat());
            let info_content = Paragraph::new(info_text).block(
                Block::new()
                    .title(" Project Information ")
                    .title_style(Style::new().fg(colors.fg))
                    .padding(Padding::proportional(1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(colors.border),
            );
            frame.render_widget(info_content, info_layout);
        } else {
            let content = Paragraph::new(Text::from(vec![Line::from(vec![
                Span::from("You have no projects. Press "),
                Span::styled("n", Style::new().bold().fg(colors.keybind_key)),
                Span::from(" to create a new project."),
            ])]))
            .block(block);

            frame.render_widget(content, list_layout);
            frame.render_widget(Block::new(), info_layout);
        }
    }
}

impl ListProjects {
    pub fn projects_state(&mut self, projects_state: ProjectsState) {
        self.projects_state = Some(projects_state);
    }

    pub fn db_get_projects(&mut self, app: &App) -> rusqlite::Result<()> {
        let conn = app.db.conn();
        let project_query = "SELECT id, title, description, position, created_at, updated_at FROM \
                             project ORDER BY position";
        let mut project_stmt = conn.prepare(project_query).unwrap();
        let project_iter = project_stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    position: row.get(3)?,
                    created_at: DateTime::from_db(row.get(4)?),
                    updated_at: DateTime::from_db(row.get(5)?),
                    labels: 0,
                    lists: 0,
                    total_cards: 0,
                    cards_in_progress: 0,
                    cards_due_soon: 0,
                    cards_overdue: 0,
                    important_cards: 0,
                })
            })
            .unwrap();
        let mut projects = Vec::new();
        for p in project_iter {
            projects.push(p.unwrap())
        }

        projects = self.db_get_labels(&app.db, &mut projects).unwrap();
        projects = self.db_get_lists(&app.db, &mut projects).unwrap();
        projects = self.db_get_cards(app, &mut projects).unwrap();

        if !projects.is_empty() && self.selected_id.is_none() {
            self.selected_id = Some(projects[0].id);
        }

        self.projects = projects;

        Ok(())
    }

    fn db_get_labels(
        &self,
        db: &Database,
        projects: &mut [Project],
    ) -> rusqlite::Result<Vec<Project>> {
        let conn = db.conn();
        let query = "SELECT project_id FROM project_label ORDER BY position";
        let mut stmt = conn.prepare(query)?;
        let label_iter = stmt.query_map([], |row| row.get::<usize, i32>(0))?;
        for label_id in label_iter {
            let id = label_id.unwrap();
            let index = projects.iter().position(|p| p.id == id).unwrap();
            projects[index].labels += 1;
        }
        Ok(projects.to_vec())
    }

    fn db_get_lists(
        &self,
        db: &Database,
        projects: &mut [Project],
    ) -> rusqlite::Result<Vec<Project>> {
        let conn = db.conn();
        let query = "SELECT project_id FROM project_list";
        let mut stmt = conn.prepare(query)?;
        let id_iter = stmt.query_map([], |r| r.get::<usize, i32>(0))?;
        for list_id in id_iter {
            let id = list_id.unwrap();
            let index = projects.iter().position(|p| p.id == id).unwrap();
            projects[index].lists += 1;
        }
        Ok(projects.to_vec())
    }

    fn db_get_cards(&self, app: &App, projects: &mut [Project]) -> rusqlite::Result<Vec<Project>> {
        struct ListProjectCard {
            project_id: i32,
            start_date: Option<DateTime>,
            due_date: Option<DateTime>,
            important: bool,
        }

        let conn = app.db.conn();
        let query = "SELECT project_id, start_date, due_date, important FROM project_card ORDER \
                     BY position";
        let mut stmt = conn.prepare(query)?;
        let card_iter = stmt.query_map([], |row| {
            Ok(ListProjectCard {
                project_id: row.get(0)?,
                start_date: DateTime::from_db_option(row.get(1)?),
                due_date: DateTime::from_db_option(row.get(2)?),
                important: row.get(3)?,
            })
        })?;

        for c in card_iter {
            let card = c.unwrap();
            let index = projects
                .iter()
                .position(|p| p.id == card.project_id)
                .unwrap();
            projects[index].total_cards += 1;

            if card.start_date.as_ref().is_some_and(|d| d.is_past())
                && !card.due_date.as_ref().is_some_and(|d| d.is_past())
            {
                projects[index].cards_in_progress += 1;
            }

            if let Some(due_date) = card.due_date {
                if due_date.is_past_days(app.config.modules.project_management.due_soon_days)
                    && !due_date.is_past()
                {
                    projects[index].cards_due_soon += 1;
                }

                if due_date.is_past() {
                    projects[index].cards_overdue += 1;
                }
            }

            if card.important {
                projects[index].important_cards += 1;
            }
        }

        Ok(projects.to_vec())
    }
}

impl ListProjects {
    fn db_delete_project(&mut self, db: &Database) -> rusqlite::Result<()> {
        struct Select {
            position: i32,
        }
        let conn = db.conn();
        let select_query = "SELECT position FROM project WHERE id = ?1";
        let mut select_stmt = conn.prepare(select_query)?;
        let select = select_stmt.query_row([self.selected_id], |r| {
            Ok(Select {
                position: r.get(0)?,
            })
        })?;

        let query = "DELETE FROM project WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        stmt.execute([self.selected_id])?;

        let update_position_query =
            "UPDATE project SET position = position - 1, updated_at = ?1 WHERE position > ?2";
        let mut update_position_stmt = conn.prepare(update_position_query)?;
        update_position_stmt.execute((DateTime::now(), select.position))?;

        if let Some(selected_id) = self.selected_id {
            let selected_index = self
                .projects
                .iter()
                .position(|p| p.id == selected_id)
                .unwrap_or(0);

            if self.projects.len() == 1 {
                self.selected_id = None;
            } else if selected_index != self.projects.len().saturating_sub(1) {
                self.selected_id = Some(self.projects[selected_index + 1].id);
            } else if selected_index != 0 {
                self.selected_id = Some(self.projects[selected_index.saturating_sub(1)].id);
            } else {
                self.selected_id = Some(self.projects[0].id);
            }
        }

        Ok(())
    }
}
