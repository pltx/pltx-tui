//! The Project Management Modules - Similar to Trello or GitHub Projects.

use color_eyre::Result;
use crossterm::event::KeyEvent;
use pltx_app::{App, DefaultWidget, KeyEventHandler, Module, Screen};
use pltx_database::Database;
use pltx_widgets::Tabs;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

mod list_projects;
mod open_project;
pub mod popups;
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
    tabs: Tabs<Tab>,
    screens: Screens,
}

impl Module<Result<()>> for ProjectManagement {
    fn init(app: &App) -> Result<Self> {
        ProjectManagement::init_data(&app.db)?;

        Ok(Self {
            tabs: Tabs::from([
                (Tab::Planned, "Planned"),
                (Tab::Projects, "Projects"),
                (Tab::Important, "Important"),
            ]),
            screens: Screens {
                projects: Projects::init(app)?,
            },
        })
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        self.tabs.key_event_handler(app, key_event);

        // Should be run before the rest.
        match self.tabs.active {
            Tab::Planned => {}
            Tab::Projects => self.screens.projects.key_event_handler(app, key_event)?,
            Tab::Important => {}
        }

        Ok(())
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let [tabs_layout, content_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(area);

        self.tabs.render(frame, app, tabs_layout, true);

        match self.tabs.active {
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
