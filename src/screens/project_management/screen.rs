use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
    Frame,
};

use super::create_project::CreateProject;
use crate::{
    config::ColorsConfig,
    state::{Mode, Pane, State},
    utils::{
        pane_title_bottom, Init, InitData, KeyEventHandler, RenderPopup, RenderScreen,
        ScreenKeybinds, ScreenKeybindsTitle,
    },
    App,
};

#[derive(PartialEq, Clone)]
enum Tab {
    Planned,
    Projects,
    Important,
}

#[derive(PartialEq)]
enum ScreenPane {
    Tabs,
    Main,
    None,
}

pub struct Popups {
    create_project: CreateProject,
}

pub struct ProjectManagement {
    tab: Tab,
    hover_tab: Tab,
    screen_pane: ScreenPane,
    popups: Popups,
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

impl ScreenKeybinds for ProjectManagement {
    fn screen_keybinds<'a>(&mut self) -> [(&'a str, &'a str); 3] {
        [("n", "New"), ("e", "Edit"), ("d", "Delete")]
    }
}

impl ScreenKeybindsTitle for ProjectManagement {
    fn screen_keybinds_title(&mut self, app: &mut App) -> Line {
        pane_title_bottom(app, self.screen_keybinds())
    }
}

impl KeyEventHandler for ProjectManagement {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) {
        if app.state.pane != Pane::Screen {
            return;
        };

        if app.state.mode == Mode::Navigation {
            let tabs = self.get_tabs();
            let tab_index = tabs.iter().position(|t| t.0 == self.hover_tab).unwrap();
            match key_event.code {
                KeyCode::Char('l') => {
                    if self.screen_pane == ScreenPane::Tabs {
                        if tab_index == tabs.len() - 1 {
                            self.hover_tab = tabs[0].0.clone();
                        } else {
                            self.hover_tab = tabs[tab_index + 1].0.clone();
                        }
                    }
                }
                KeyCode::Char('h') => {
                    if self.screen_pane == ScreenPane::Tabs {
                        if tab_index == 0 {
                            self.hover_tab = tabs[tabs.len() - 1].0.clone();
                        } else {
                            self.hover_tab = tabs[tab_index - 1].0.clone();
                        }
                    }
                }
                KeyCode::Enter => match self.screen_pane {
                    ScreenPane::None => self.screen_pane = ScreenPane::Tabs,
                    ScreenPane::Tabs => {
                        self.tab = self.hover_tab.clone();
                        self.screen_pane = ScreenPane::Main;
                    }
                    ScreenPane::Main => {}
                },
                KeyCode::Backspace => match self.screen_pane {
                    ScreenPane::Main => self.screen_pane = ScreenPane::Tabs,
                    ScreenPane::Tabs => {
                        if event_state.pane == Pane::Screen {
                            self.screen_pane = ScreenPane::None;
                            app.state.pane = Pane::Navigation;
                        }
                    }
                    ScreenPane::None => {}
                },
                KeyCode::Char('n') => {
                    // TODO: Create project
                }
                _ => {}
            }
        }
    }
}

impl Init for ProjectManagement {
    fn init(app: &mut App) -> ProjectManagement {
        ProjectManagement {
            tab: Tab::Planned,
            hover_tab: Tab::Planned,
            screen_pane: ScreenPane::None,
            popups: Popups {
                create_project: CreateProject::init(app),
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
                list_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                important BOOLEAN NOT NULL CHECK (important IN (0, 1)),
                due_date DATETIME,
                reminder BOOLEAN NOT NULL CHECK (important IN (0, 1)),
                labels
                checklist
                position INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (list_id)
                    REFERENCES project_list (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE
            )",
            (),
        )?;
        Ok(())
    }
}

impl RenderScreen for ProjectManagement {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let [navigation_layout, content_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(area);
        frame.render_widget(self.navigation(colors), navigation_layout);

        self.popups.create_project.render(frame, app);

        let content = Block::new();
        frame.render_widget(content, content_layout)
    }
}

impl ProjectManagement {
    fn navigation(&mut self, colors: &ColorsConfig) -> impl Widget {
        let navigation_line = vec![Line::from(
            self.get_tabs()
                .iter()
                .map(|t| {
                    let mut style = Style::new();
                    if t.0 == self.tab {
                        style = style.fg(colors.active_fg).bg(colors.active_bg).bold()
                    } else if t.0 == self.hover_tab {
                        style = style.fg(colors.hover_fg).bg(colors.hover_bg)
                    } else {
                        style = style.fg(colors.secondary)
                    };
                    Span::from(format!(" {} ", t.1)).style(style)
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