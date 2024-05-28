//! The Project Management Modules - Similar to Trello or GitHub Projects.

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, Module, Screen};
use pltx_config::ColorsConfig;
use pltx_database::Database;
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

use projects::Projects;

/// Project management tab.
#[derive(PartialEq, Clone)]
pub enum Tab {
    Planned,
    Projects,
    Important,
}

struct Screens {
    projects: Projects,
}

pub struct ProjectManagement {
    tab: Tab,
    screens: Screens,
}

impl Module<Result<()>> for ProjectManagement {
    fn init(app: &App) -> Result<Self> {
        ProjectManagement::init_data(&app.db)?;

        Ok(Self {
            tab: Tab::Projects,
            screens: Screens {
                projects: Projects::init(app)?,
            },
        })
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        // Should be run before the rest.
        match self.tab {
            Tab::Planned => {}
            Tab::Projects => self.screens.projects.key_event_handler(app, key_event)?,
            Tab::Important => {}
        }

        if app.display.is_default() {
            let tabs = self.get_tabs();
            let tab_index = tabs
                .iter()
                .position(|t| t.0 == self.tab)
                .expect("failed to get tab index");
            match key_event.code {
                KeyCode::Char('}') => {
                    if tab_index != tabs.len().saturating_sub(1) {
                        self.tab = tabs[tab_index + 1].0.clone();
                    }
                }
                KeyCode::Char('{') => {
                    if tab_index != 0 {
                        self.tab = tabs[tab_index.saturating_sub(1)].0.clone();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let [navigation_layout, content_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(area);

        frame.render_widget(self.navigation(colors), navigation_layout);

        match self.tab {
            Tab::Planned => {}
            Tab::Projects => self.screens.projects.render(app, frame, content_layout),
            Tab::Important => {}
        }
    }
}

impl ProjectManagement {
    pub fn init_data(db: &Database) -> Result<()> {
        db.conn().execute_batch(
            "BEGIN;

            CREATE TABLE IF NOT EXISTS project (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                position INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_label (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                color TEXT NOT NULL,
                position INTEGER NOT NULL,
                archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
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
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
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
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
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
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
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
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
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
    fn navigation(&self, colors: &ColorsConfig) -> impl Widget {
        let navigation_line = vec![Line::from(
            self.get_tabs()
                .iter()
                .enumerate()
                .flat_map(|(i, t)| {
                    let mut style = Style::new();
                    if t.0 == self.tab {
                        style = style.fg(colors.active_fg).bg(colors.active_bg).bold()
                    } else {
                        style = style.fg(colors.secondary_fg)
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
                .border_style(Style::new().fg(colors.border)),
        )
    }
}
