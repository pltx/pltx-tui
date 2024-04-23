use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table},
};

use super::{projects::ProjectsState, screen::ScreenPane};
use crate::{
    state::{Mode, State},
    utils::{pane_title_bottom, Init, InitData, KeyEventHandler, RenderPage, ScreenKeybinds},
    App,
};

struct Card {
    project_id: i32,
    due_date: Option<String>,
}

struct Project {
    id: i32,
    title: String,
    position: i32,
    created_at: String,
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
        let project_query = "SELECT id, title, position, created_at FROM project ORDER BY position";
        let mut project_stmt = app.db.conn.prepare(project_query).unwrap();
        let project_iter = project_stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    position: row.get(2)?,
                    created_at: row.get(3)?,
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

        if !projects.is_empty() {
            self.selected_id = projects[0].id;
        }
        self.projects = projects;
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

impl KeyEventHandler for ListProjects {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) {
        if app.state.mode == Mode::Navigation {
            let selected_index = self
                .projects
                .iter()
                .position(|p| p.id == self.selected_id)
                .unwrap_or(0);

            match key_event.code {
                KeyCode::Char('j') => {
                    if selected_index != self.projects.len() - 1 {
                        self.selected_id = self.projects[selected_index + 1].id;
                    } else {
                        self.selected_id = self.projects[0].id;
                    }
                }
                KeyCode::Char('k') => {
                    if selected_index != 0 {
                        self.selected_id = self.projects[selected_index - 1].id;
                    } else {
                        self.selected_id = self.projects[self.projects.len() - 1].id;
                    }
                }
                _ => {}
            }
        }
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
            frame.render_widget(content, area)
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
            // let rows = vec![Row::new(vec![Cell::new("something")])];
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
            frame.render_widget(table, area);
        }
    }
}
