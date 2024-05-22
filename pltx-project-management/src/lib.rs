use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Pane, App};
use pltx_config::ColorsConfig;
use pltx_utils::{Module, Screen};
use projects::Projects;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
    Frame,
};

mod card_editor;
mod list_editor;
mod list_projects;
mod open_project;
mod project_editor;
mod projects;

#[derive(PartialEq, Clone)]
enum Tab {
    Planned,
    Projects,
    Important,
}

#[derive(PartialEq, Clone)]
pub enum ProjectManagementPane {
    Tabs,
    Main,
    None,
}

struct Pages {
    projects: Projects,
}

pub struct ProjectManagement {
    tab: Tab,
    last_pane: ProjectManagementPane,
    pane: ProjectManagementPane,
    pages: Pages,
}

impl Module for ProjectManagement {
    fn init(app: &App) -> ProjectManagement {
        let project_management = ProjectManagement {
            tab: Tab::Projects,
            last_pane: ProjectManagementPane::Main,
            pane: ProjectManagementPane::None,
            pages: Pages {
                projects: Projects::init(app),
            },
        };

        project_management.init_data(app).unwrap();

        project_management
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        if app.pane != Pane::Module {
            return;
        };

        // Should be run before the rest.
        if self.pane == ProjectManagementPane::Main {
            match self.tab {
                Tab::Planned => {}
                Tab::Projects => self.pages.projects.key_event_handler(app, key_event),
                Tab::Important => {}
            }
        }

        if app.display.is_default() {
            let tabs = self.get_tabs();
            let tab_index = tabs.iter().position(|t| t.0 == self.tab).unwrap();
            match key_event.code {
                KeyCode::Char('l') => {
                    if self.pane == ProjectManagementPane::Tabs
                        && tab_index != tabs.len().saturating_sub(1)
                    {
                        self.tab = tabs[tab_index + 1].0.clone();
                    } else if self.pane == ProjectManagementPane::None {
                        self.pane(self.last_pane.clone());
                    }
                }
                KeyCode::Char('h') => {
                    if self.pane == ProjectManagementPane::Tabs && tab_index != 0 {
                        self.tab = tabs[tab_index.saturating_sub(1)].0.clone();
                    }
                }
                KeyCode::Char('H') => {
                    self.last_pane = self.pane.clone();
                    self.pane(ProjectManagementPane::None);
                    app.pane = Pane::Navigation;
                }
                KeyCode::Char('J') | KeyCode::Char('j') => {
                    if self.pane == ProjectManagementPane::Tabs {
                        self.pane(ProjectManagementPane::Main);
                    }
                }
                KeyCode::Char('K') => {
                    if self.pane == ProjectManagementPane::Main {
                        self.pane(ProjectManagementPane::Tabs);
                    }
                }
                KeyCode::Char('L') => {
                    if self.pane == ProjectManagementPane::None {
                        self.pane(self.last_pane.clone())
                    }
                }
                KeyCode::Enter => {
                    if self.pane == ProjectManagementPane::None {
                        self.pane(self.last_pane.clone())
                    } else if self.pane == ProjectManagementPane::Tabs {
                        self.pane(ProjectManagementPane::Main)
                    }
                }
                _ => {}
            }
        }
    }

    fn render(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let [navigation_layout, content_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(area);
        frame.render_widget(self.navigation(colors), navigation_layout);

        match self.tab {
            Tab::Planned => {}
            Tab::Projects => self.pages.projects.render(app, frame, content_layout),
            Tab::Important => {}
        }
    }
}

impl ProjectManagement {
    fn init_data(&self, app: &App) -> rusqlite::Result<()> {
        app.db.conn.execute_batch(
            "BEGIN;

            CREATE TABLE IF NOT EXISTS project (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                position INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS project_label (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                color TEXT NOT NULL,
                position INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            );

            CREATE TABLE IF NOT EXISTS project_list (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                position INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            );

            CREATE TABLE IF NOT EXISTS project_card (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                list_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                important BOOLEAN NOT NULL CHECK (important IN (0, 1)),
                start_date DATETIME,
                due_date DATETIME,
                reminder INTEGER,
                completed BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                position INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (list_id)
                    REFERENCES project_list (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            );

            CREATE TABLE IF NOT EXISTS card_label (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                card_id INTEGER NOT NULL,
                label_id INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE,
                FOREIGN KEY (card_id)
                    REFERENCES project_card (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE,
                FOREIGN KEY (label_id)
                    REFERENCES project_label (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            );

            CREATE TABLE IF NOT EXISTS card_subtask (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                card_id INTEGER NOT NULL,
                value TEXT NOT NULL,
                completed BOOLEAN NOT NULL CHECK (completed IN (0, 1)),
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE,
                FOREIGN KEY (card_id)
                    REFERENCES project_card (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            );

            COMMIT;",
        )?;

        Ok(())
    }
}

impl ProjectManagement {
    fn get_tabs<'a>(&self) -> Vec<(Tab, &'a str)> {
        vec![
            (Tab::Planned, "Planned"),
            (Tab::Projects, "Projects"),
            (Tab::Important, "Important"),
        ]
    }
}

impl ProjectManagement {
    fn pane(&mut self, pane: ProjectManagementPane) {
        self.pages.projects.projects_state(projects::ProjectsState {
            module_pane: pane.clone(),
        });
        self.pane = pane;
    }

    fn navigation(&mut self, colors: &ColorsConfig) -> impl Widget {
        let navigation_line = vec![Line::from(
            self.get_tabs()
                .iter()
                .enumerate()
                .flat_map(|(i, t)| {
                    let mut style = Style::new();
                    if t.0 == self.tab {
                        style = style.fg(colors.active_fg).bg(colors.active_bg).bold()
                    } else {
                        style = style.fg(colors.secondary)
                    };
                    let mut span = vec![Span::from(format!(" {} ", t.1)).style(style)];
                    if i != self.get_tabs().len().saturating_sub(1) {
                        span.push(Span::styled(" | ", Style::new().fg(colors.border)))
                    }
                    span
                })
                .collect::<Vec<Span>>(),
        )];
        Paragraph::new(navigation_line).block(
            Block::new()
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::new().fg(if self.pane == ProjectManagementPane::Tabs {
                        colors.primary
                    } else {
                        colors.border
                    }),
                ),
        )
    }
}
