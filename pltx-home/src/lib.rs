use color_eyre::Result;
use crossterm::event::KeyEvent;
use pltx_app::{App, DefaultWidget, KeyEventHandler, Module, Screen};
use pltx_widgets::Tabs;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

mod dashboard;

use dashboard::Dashboard;

#[derive(Clone, PartialEq)]
enum Tab {
    Dashboard,
    Settings,
    Help,
}

pub struct Screens {
    dashboard: Dashboard,
}

pub struct Home {
    tabs: Tabs<Tab>,
    screens: Screens,
}

impl Module for Home {
    fn init(app: &App) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            tabs: Tabs::from(vec![
                (Tab::Dashboard, "Dashboard"),
                (Tab::Settings, "Settings"),
                (Tab::Help, "Help"),
            ]),
            screens: Screens {
                dashboard: Dashboard::init(app)?,
            },
        })
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        self.tabs.key_event_handler(app, key_event);

        match self.tabs.active {
            Tab::Dashboard => self.screens.dashboard.key_event_handler(app, key_event),
            Tab::Settings => {}
            Tab::Help => {}
        }
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let [tabs_layout, screen_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(area);

        self.tabs.render(frame, app, tabs_layout, true);

        match self.tabs.active {
            Tab::Dashboard => self.screens.dashboard.render(app, frame, screen_layout),
            Tab::Settings => {}
            Tab::Help => {}
        }
    }
}
