use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
    Frame,
};

use super::projects::{Projects, ProjectsState};
use crate::{
    config::ColorsConfig,
    state::{Mode, Pane, State},
    utils::{Init, InitData, KeyEventHandler, RenderPage, RenderScreen},
    App,
};

#[derive(PartialEq, Clone)]
enum Tab {
    Planned,
    Projects,
    Important,
}

#[derive(PartialEq)]
enum Popup {
    None,
}

#[derive(PartialEq, Clone)]
pub enum ScreenPane {
    Tabs,
    Main,
    None,
}

struct Pages {
    projects: Projects,
}

pub struct ProjectManagement {
    tab: Tab,
    popup: Popup,
    last_screen_pane: ScreenPane,
    pub screen_pane: ScreenPane,
    pages: Pages,
}

impl Init for ProjectManagement {
    fn init(app: &mut App) -> ProjectManagement {
        ProjectManagement {
            tab: Tab::Projects,
            popup: Popup::None,
            last_screen_pane: ScreenPane::Main,
            screen_pane: ScreenPane::None,
            pages: Pages {
                projects: Projects::init(app),
            },
        }
    }
}

impl InitData for ProjectManagement {
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()> {
        app.db.conn.execute(
            "CREATE TABLE IF NOT EXISTS project (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                position INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            (),
        )?;

        app.db.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_label (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                color TEXT,
                position INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            )",
            (),
        )?;

        app.db.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_list (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                color TEXT,
                position INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id)
                    REFERENCES project (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            )",
            (),
        )?;

        app.db.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_card (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                list_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                important BOOLEAN NOT NULL CHECK (important IN (0, 1)),
                due_date DATETIME,
                reminder DATETIME,
                position INTEGER NOT NULL,
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
            )",
            (),
        )?;

        app.db.conn.execute(
            "CREATE TABLE IF NOT EXISTS card_label (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                card_id INTEGER NOT NULL,
                label_id INTEGER NOT NULL,
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
            )",
            (),
        )?;

        app.db.conn.execute(
            "CREATE TABLE IF NOT EXISTS card_subtask (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                card_id INTEGER NOT NULL,
                value TEXT NOT NULL,
                completed BOOLEAN NOT NULL CHECK (completed IN (0, 1)),
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
            )",
            (),
        )?;

        self.pages.projects.init_data(app)?;

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

impl KeyEventHandler for ProjectManagement {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) {
        if app.state.pane != Pane::Screen {
            return;
        };

        // Page key event handlers should run before anything for the entire screen
        if self.screen_pane == ScreenPane::Main {
            match self.tab {
                Tab::Planned => {}
                Tab::Projects => self
                    .pages
                    .projects
                    .key_event_handler(app, key_event, event_state),
                Tab::Important => {}
            }
        }

        if app.state.mode == Mode::Navigation {
            let tabs = self.get_tabs();
            let tab_index = tabs.iter().position(|t| t.0 == self.tab).unwrap();
            match key_event.code {
                KeyCode::Char('l') => {
                    if self.screen_pane == ScreenPane::Tabs
                        && tab_index != tabs.len().saturating_sub(1)
                    {
                        self.tab = tabs[tab_index + 1].0.clone();
                    }
                }
                KeyCode::Char('h') => {
                    if self.screen_pane == ScreenPane::Tabs && tab_index != 0 {
                        self.tab = tabs[tab_index.saturating_sub(1)].0.clone();
                    }
                }
                KeyCode::Char('H') => {
                    self.last_screen_pane = self.screen_pane.clone();
                    self.screen_pane = ScreenPane::None;
                    app.state.pane = Pane::Navigation;
                }
                KeyCode::Char('J') => {
                    if self.screen_pane == ScreenPane::Tabs {
                        self.screen_pane = ScreenPane::Main
                    }
                }
                KeyCode::Char('K') => {
                    if self.screen_pane == ScreenPane::Main {
                        self.screen_pane = ScreenPane::Tabs
                    }
                }
                KeyCode::Enter | KeyCode::Char('L') => {
                    if self.screen_pane == ScreenPane::None {
                        self.screen_pane = self.last_screen_pane.clone()
                    }
                }
                _ => {}
            }
        }
    }
}

impl RenderScreen for ProjectManagement {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let [navigation_layout, content_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(area);
        frame.render_widget(self.navigation(colors), navigation_layout);

        match self.tab {
            Tab::Planned => {}
            Tab::Projects => self.pages.projects.render(
                app,
                frame,
                content_layout,
                ProjectsState {
                    screen_pane: self.screen_pane.clone(),
                },
            ),
            Tab::Important => {}
        }

        if app.state.mode == Mode::Popup {
            match self.popup {
                Popup::None => {}
            }
        }
    }
}

impl ProjectManagement {
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
                .border_style(Style::new().fg(if self.screen_pane == ScreenPane::Tabs {
                    colors.primary
                } else {
                    colors.border
                })),
        )
    }
}
