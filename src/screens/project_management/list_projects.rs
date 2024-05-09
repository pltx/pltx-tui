use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table},
};

use super::{projects::ProjectsState, screen::ScreenPane};
use crate::{
    state::{Mode, State},
    utils::{pane_title_bottom, Init, InitData, KeyEventHandlerReturn, RenderPage, ScreenKeybinds},
    App,
};

struct Card {
    project_id: i32,
    due_date: Option<String>,
}

struct Project {
    id: i32,
    title: String,
    description: Option<String>,
    position: i32,
    created_at: String,
    updated_at: String,
    cards: Vec<Card>,
}

pub struct ListProjects {
    projects: Vec<Project>,
    pub selected_id: i32,
}

impl Init for ListProjects {
    fn init(_: &mut App) -> ListProjects {
        ListProjects {
            projects: vec![],
            selected_id: 0,
        }
    }
}

impl InitData for ListProjects {
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()> {
        self.db_get_projects(app)
    }
}

impl ListProjects {
    pub fn db_get_projects(&mut self, app: &mut App) -> rusqlite::Result<()> {
        let project_query = "SELECT id, title, description, position, created_at, updated_at FROM \
                             project ORDER BY position";
        let mut project_stmt = app.db.conn.prepare(project_query).unwrap();
        let project_iter = project_stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    position: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                    cards: vec![],
                })
            })
            .unwrap();
        let mut projects = Vec::new();
        for p in project_iter {
            projects.push(p.unwrap())
        }

        let card_query = "SELECT project_id, due_date FROM project_card";
        let mut card_stmt = app.db.conn.prepare(card_query).unwrap();
        let card_iter = card_stmt
            .query_map([], |row| {
                Ok(Card {
                    project_id: row.get(0)?,
                    due_date: row.get(1)?,
                })
            })
            .unwrap();
        for c in card_iter {
            let card = c.unwrap();
            let index = projects
                .iter()
                .position(|p| p.id == card.project_id)
                .unwrap();
            projects[index].cards.push(card);
        }

        if !projects.is_empty() && self.selected_id == 0 {
            self.selected_id = projects[0].id;
        }

        self.projects = projects;

        Ok(())
    }
}

impl ListProjects {
    fn db_delete_project(&mut self, app: &App) -> rusqlite::Result<()> {
        struct Select {
            position: i32,
        }
        let select_query = "SELECT position FROM project WHERE id = ?1";
        let mut select_stmt = app.db.conn.prepare(select_query)?;
        let select = select_stmt.query_row([self.selected_id], |r| {
            Ok(Select {
                position: r.get(0)?,
            })
        })?;

        let query = "DELETE FROM project WHERE id = ?1";
        let mut stmt = app.db.conn.prepare(query)?;
        stmt.execute([self.selected_id])?;

        let update_position_query =
            "UPDATE project SET position = position - 1 WHERE position > ?1";
        let mut update_position_stmt = app.db.conn.prepare(update_position_query)?;
        update_position_stmt.execute([select.position])?;

        let selected_index = self
            .projects
            .iter()
            .position(|p| p.id == self.selected_id)
            .unwrap_or(0);

        if self.projects.len() == 1 {
            self.selected_id = 0;
        } else if selected_index != self.projects.len().saturating_sub(1) {
            self.selected_id = self.projects[selected_index + 1].id;
        } else if selected_index != 0 {
            self.selected_id = self.projects[selected_index.saturating_sub(1)].id;
        } else {
            self.selected_id = self.projects[0].id;
        }

        Ok(())
    }
}

impl ScreenKeybinds for ListProjects {
    fn screen_keybinds<'a>(&self) -> Vec<(&'a str, &'a str)> {
        vec![
            ("n", "New Project"),
            ("e", "Edit Project"),
            ("d", "Delete Project"),
        ]
    }
}

impl KeyEventHandlerReturn<bool> for ListProjects {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) -> bool {
        if app.state.mode == Mode::Navigation {
            let selected_index = self
                .projects
                .iter()
                .position(|p| p.id == self.selected_id)
                .unwrap_or(0);

            match key_event.code {
                KeyCode::Char('d') => {
                    app.state.mode = Mode::Delete;
                }
                KeyCode::Char('j') => {
                    if selected_index != self.projects.len().saturating_sub(1) {
                        self.selected_id = self.projects[selected_index + 1].id;
                    } else {
                        self.selected_id = self.projects[0].id;
                    }
                }
                KeyCode::Char('k') => {
                    if selected_index != 0 {
                        self.selected_id = self.projects[selected_index.saturating_sub(1)].id;
                    } else {
                        self.selected_id = self.projects[self.projects.len().saturating_sub(1)].id;
                    }
                }
                _ => {}
            }
        }

        if app.state.mode == Mode::Delete {
            match key_event.code {
                KeyCode::Char('y') => {
                    app.state.mode = Mode::Navigation;
                    self.db_delete_project(app)
                        .unwrap_or_else(|e| panic!("{e}"));
                    self.db_get_projects(app).unwrap_or_else(|e| panic!("{e}"));
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

impl RenderPage<ProjectsState> for ListProjects {
    fn render(
        &mut self,
        app: &mut App,
        frame: &mut ratatui::prelude::Frame,
        area: ratatui::prelude::Rect,
        state: ProjectsState,
    ) {
        let [list_layout, info_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(40)])
            .areas(area);

        let colors = &app.config.colors.clone();
        let block = Block::new()
            .title_bottom(pane_title_bottom(
                colors,
                self.screen_keybinds(),
                state.screen_pane == ScreenPane::Main,
            ))
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(if state.screen_pane == ScreenPane::Main {
                colors.primary
            } else {
                colors.border
            }));

        if self.projects.is_empty() {
            let content = Paragraph::new(Text::from(vec![Line::from(vec![
                Span::from("You have no projects. Press "),
                Span::styled("n", Style::new().bold().fg(colors.keybind_key)),
                Span::from(" to create a new project."),
            ])]))
            .block(block);

            frame.render_widget(content, list_layout);
            frame.render_widget(Block::new(), info_layout);
        } else {
            let rows = self
                .projects
                .iter()
                .enumerate()
                .map(|(_, p)| {
                    Row::new(vec![
                        Cell::new(format!(" {}", p.position)),
                        Cell::new(p.title.clone()),
                        Cell::new(p.cards.len().to_string()),
                        // TODO: Implement due soon
                        Cell::new("0"),
                        // TODO: Implement overdue
                        Cell::new("0"),
                        Cell::new(p.created_at.clone()),
                    ])
                    .style(if p.id == self.selected_id {
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
                Constraint::Length(10),
                Constraint::Max(50),
                Constraint::Length(7),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(20),
            ];
            let table = Table::new(rows, widths).block(block).header(
                Row::new(vec![
                    Cell::new(" Position"),
                    Cell::new("Title"),
                    Cell::new("Cards"),
                    Cell::new("Due Soon"),
                    Cell::new("Overdue"),
                    Cell::new("Created At "),
                ])
                .style(Style::new().bold().fg(colors.primary)),
            );
            frame.render_widget(table, list_layout);

            let project_index = self
                .projects
                .iter()
                .position(|p| p.id == self.selected_id)
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
                    Span::styled("Cards: ", Style::new().fg(colors.secondary)),
                    Span::from(project.cards.len().to_string()),
                ]),
                Line::from(vec![
                    Span::styled("Created At: ", Style::new().fg(colors.secondary)),
                    Span::from(&project.created_at),
                ]),
                Line::from(vec![
                    Span::styled("Updated At: ", Style::new().fg(colors.secondary)),
                    Span::from(&project.updated_at),
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
        }
    }
}
