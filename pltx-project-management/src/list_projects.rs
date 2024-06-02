use std::time::Instant;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, KeyEventHandler, Screen};
use pltx_database::Database;
use pltx_utils::{centered_rect, DateTime};
use pltx_widgets::Scrollable;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};
use tracing::{info, info_span};

#[derive(Clone)]
pub struct Project {
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
    pub selection: Scrollable,
    pub projects: Vec<Project>,
}

impl Screen<Result<bool>> for ListProjects {
    fn init(app: &App) -> Result<ListProjects> {
        let mut list_projects = ListProjects {
            projects: vec![],
            selection: Scrollable::default().cols([5, 50, 7, 13, 10, 9, 9, 8]),
        };

        list_projects.db_get_projects(app)?;

        Ok(list_projects)
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        self.selection.key_event_handler(app, key_event);

        if app.mode.is_normal() && key_event.code == KeyCode::Char('d') {
            app.mode.delete();
        }

        if app.mode.is_delete() {
            match key_event.code {
                KeyCode::Char('y') => {
                    self.db_delete_project(&app.db)?;
                    self.db_get_projects(app)?;
                    app.mode.normal();
                }
                KeyCode::Char('n') => app.mode.normal(),
                _ => {}
            }
        }
        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors.clone();

        let [list_side_layout, info_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(40)])
            .areas(area);

        let centered_list_layout = centered_rect(
            (
                self.selection
                    .col_lengths
                    .to_owned()
                    .expect("failed to get col lengths for list projects")
                    .iter()
                    .sum(),
                false,
            ),
            (list_side_layout.height, false),
            list_side_layout,
        );

        let [list_layout] = Layout::default()
            .vertical_margin(1)
            .constraints([Constraint::Fill(1)])
            .areas(centered_list_layout);

        if self.projects.is_empty() {
            let content = Paragraph::new(Text::from(vec![Line::from(vec![
                Span::from("You have no projects. Press "),
                Span::styled("n", Style::new().bold().fg(colors.keybind_key)),
                Span::from(" to create a new project."),
            ])]));

            frame.render_widget(content, list_layout);
            frame.render_widget(Block::new(), info_layout);
        } else {
            let header = [
                Paragraph::new(" "),
                Paragraph::new("Title"),
                Paragraph::new("Cards"),
                Paragraph::new("In Progress"),
                Paragraph::new("Due Soon"),
                Paragraph::new("Overdue"),
                Paragraph::new("Important"),
            ]
            .into_iter()
            .map(|p| p.fg(colors.secondary_fg))
            .collect::<Vec<Paragraph>>();

            let table = self
                .projects
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    vec![
                        Paragraph::new(format!(" {}", p.position)).fg(colors.secondary_fg),
                        Paragraph::new(p.title.clone()),
                        Paragraph::new(if p.total_cards > 0 {
                            p.total_cards.to_string()
                        } else {
                            String::from("-")
                        })
                        .fg(if p.total_cards > 0 {
                            colors.secondary_fg
                        } else {
                            colors.tertiary_fg
                        }),
                        Paragraph::new(if p.cards_in_progress > 0 {
                            p.cards_in_progress.to_string()
                        } else {
                            String::from("-")
                        })
                        .fg(if p.cards_in_progress > 0 {
                            colors.success
                        } else {
                            colors.tertiary_fg
                        }),
                        Paragraph::new(if p.cards_due_soon > 0 {
                            p.cards_due_soon.to_string()
                        } else {
                            String::from("-")
                        })
                        .fg(if p.cards_due_soon > 0 {
                            colors.warning
                        } else {
                            colors.tertiary_fg
                        }),
                        Paragraph::new(if p.cards_overdue > 0 {
                            p.cards_overdue.to_string()
                        } else {
                            String::from("-")
                        })
                        .fg(if p.cards_overdue > 0 {
                            colors.danger
                        } else {
                            colors.tertiary_fg
                        }),
                        Paragraph::new(if p.important_cards > 0 {
                            p.important_cards.to_string()
                        } else {
                            String::from("-")
                        })
                        .fg(if p.important_cards > 0 {
                            colors.primary
                        } else {
                            colors.tertiary_fg
                        }),
                    ]
                    .into_iter()
                    .map(|widget| {
                        if self.selection.focused == i {
                            widget.style(
                                Style::new()
                                    .bold()
                                    .fg(colors.active_fg)
                                    .bg(colors.active_bg),
                            )
                        } else {
                            widget
                        }
                    })
                    .collect::<Vec<Paragraph>>()
                })
                .collect::<Vec<Vec<Paragraph>>>();

            self.selection
                .render_with_cols(frame, list_layout, header, table);

            let project = &self.projects[self.selection.focused];

            let info_1 = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.id.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Title: ", Style::new().fg(colors.secondary_fg)),
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
                                Span::styled("Description: ", Style::new().fg(colors.secondary_fg)),
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
                    Span::styled("Description: ", Style::new().fg(colors.secondary_fg)),
                    Span::styled("<empty>", Style::new().fg(colors.secondary_fg)),
                ])]
            };
            let info_2 = vec![
                Line::from(vec![
                    Span::styled("Position: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.position.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Labels: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.labels.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Lists: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.lists.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.total_cards.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards In Progress: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.cards_in_progress.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards Due Soon: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.cards_due_soon.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Cards Overdue: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.cards_overdue.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Important Cards: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.important_cards.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Created At: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.created_at.display()),
                ]),
                Line::from(vec![
                    Span::styled("Updated At: ", Style::new().fg(colors.secondary_fg)),
                    Span::from(project.updated_at.display()),
                ]),
            ];
            let info_text = Text::from([info_1, description, info_2].concat());
            let info_content = Paragraph::new(info_text).block(
                Block::new()
                    .title(" Project Information ")
                    .title_style(Style::new().fg(colors.fg))
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(colors.border),
            );
            frame.render_widget(info_content, info_layout);
        }
    }
}

impl ListProjects {
    pub fn db_get_projects(&mut self, app: &App) -> Result<()> {
        let _guard = info_span!("project management", screen = "list projects").entered();

        let start = Instant::now();

        let conn = app.db.conn();
        let project_query = "SELECT id, title, description, position, created_at, updated_at FROM \
                             project ORDER BY position";
        let mut project_stmt = conn.prepare(project_query)?;
        let project_iter = project_stmt.query_map([], |row| {
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
        })?;
        let mut projects = Vec::new();
        for p in project_iter {
            projects.push(p?)
        }
        info!("get projects query executed in {:?}", start.elapsed());

        projects = self.db_get_labels(&app.db, &mut projects)?;
        projects = self.db_get_lists(&app.db, &mut projects)?;
        projects = self.db_get_cards(app, &mut projects)?;

        self.projects = projects;

        info!(
            "get projects query durations totaled at {:?}",
            start.elapsed()
        );

        Ok(())
    }

    fn db_get_labels(&self, db: &Database, projects: &mut [Project]) -> Result<Vec<Project>> {
        let start = Instant::now();
        let conn = db.conn();
        let query = "SELECT project_id FROM project_label ORDER BY position";
        let mut stmt = conn.prepare(query)?;
        let label_iter = stmt.query_map([], |row| row.get::<usize, i32>(0))?;
        for label_id in label_iter {
            let id = label_id?;
            let index = projects
                .iter()
                .position(|p| p.id == id)
                .expect("failed to get project index");
            projects[index].labels += 1;
        }
        info!("get project labels query executed in {:?}", start.elapsed());
        Ok(projects.to_vec())
    }

    fn db_get_lists(&self, db: &Database, projects: &mut [Project]) -> Result<Vec<Project>> {
        let start = Instant::now();
        let conn = db.conn();
        let query = "SELECT project_id FROM project_list";
        let mut stmt = conn.prepare(query)?;
        let id_iter = stmt.query_map([], |r| r.get::<usize, i32>(0))?;
        for list_id in id_iter {
            let id = list_id?;
            let index = projects
                .iter()
                .position(|p| p.id == id)
                .expect("failed to get project index");
            projects[index].lists += 1;
        }
        info!("get project lists query executed in {:?}", start.elapsed());
        Ok(projects.to_vec())
    }

    fn db_get_cards(&self, app: &App, projects: &mut [Project]) -> Result<Vec<Project>> {
        let start = Instant::now();
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
            let card = c?;
            let index = projects
                .iter()
                .position(|p| p.id == card.project_id)
                .expect("failed to get project index");
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

        info!("get project cards query executed in {:?}", start.elapsed());

        Ok(projects.to_vec())
    }
}

impl ListProjects {
    pub fn get_id(&self) -> Option<i32> {
        if self.projects.is_empty() {
            return None;
        }

        Some(self.projects[self.selection.focused].id)
    }

    fn db_delete_project(&mut self, db: &Database) -> Result<()> {
        if let Some(id) = self.get_id() {
            let start = Instant::now();
            struct Select {
                position: i32,
            }
            let conn = db.conn();
            let select_query = "SELECT position FROM project WHERE id = ?1";
            let mut select_stmt = conn.prepare(select_query)?;
            let select = select_stmt.query_row([id], |r| {
                Ok(Select {
                    position: r.get(0)?,
                })
            })?;

            let query = "DELETE FROM project WHERE id = ?1";
            let mut stmt = conn.prepare(query)?;
            stmt.execute([id])?;

            let update_position_query =
                "UPDATE project SET position = position - 1, updated_at = ?1 WHERE position > ?2";
            let mut update_position_stmt = conn.prepare(update_position_query)?;
            update_position_stmt.execute((DateTime::now(), select.position))?;

            if self.selection.focused == self.selection.row_count.borrow().saturating_sub(1) {
                self.selection.focused -= 1;
            }

            info!("delete project query executed in {:?}", start.elapsed());
        }

        Ok(())
    }
}
