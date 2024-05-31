//! The Project Management Modules - Similar to Trello or GitHub Projects.

use color_eyre::Result;
use crossterm::event::KeyEvent;
use pltx_app::{App, DefaultWidget, KeyEventHandler, Module, Screen};
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

include!("generated_sql.rs");

/// Project management tab.
#[derive(PartialEq, Clone)]
pub enum Tab {
    Projects,
    Planned,
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
        app.db.conn().execute_batch(SQL)?;

        Ok(Self {
            tabs: Tabs::from([
                (Tab::Projects, "Projects"),
                (Tab::Planned, "Planned"),
                (Tab::Important, "Important"),
            ]),
            screens: Screens {
                projects: Projects::init(app)?,
            },
        })
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        self.tabs.key_event_handler(app, key_event);

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
