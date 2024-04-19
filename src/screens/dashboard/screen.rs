use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Cell, Padding, Row, Table, Widget},
    Frame,
};

use crate::{
    database::Session,
    state::{Mode, Pane, State},
    utils::{KeyEventHandler, RenderScreen},
    App,
};

#[derive(PartialEq, Clone)]
enum ScreenPane {
    Sessions,
    None,
}

pub struct Dashboard {
    sessions: Vec<Session>,
    screen_pane: ScreenPane,
    screen_pane_hover: ScreenPane,
}

impl Dashboard {
    pub fn init(app: &mut App) -> Dashboard {
        let query = "SELECT id, started, ended FROM session ORDER BY started DESC LIMIT 20";
        let mut stmt = app.db.conn.prepare(query).unwrap();
        let sessions_iter = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    started: row.get(1)?,
                    ended: row.get(2)?,
                })
            })
            .unwrap();
        let mut sessions = Vec::new();
        for s in sessions_iter {
            sessions.push(s.unwrap())
        }

        Dashboard {
            sessions,
            screen_pane: ScreenPane::None,
            screen_pane_hover: ScreenPane::None,
        }
    }
}

impl KeyEventHandler for Dashboard {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) {
        if app.state.mode == Mode::Navigation && app.state.pane == Pane::Screen {
            match key_event.code {
                KeyCode::Enter => match self.screen_pane_hover {
                    ScreenPane::None => self.screen_pane = ScreenPane::Sessions,
                    ScreenPane::Sessions => {}
                },
                KeyCode::Backspace => match self.screen_pane {
                    ScreenPane::Sessions => {
                        if event_state.pane == Pane::Screen {
                            self.screen_pane = ScreenPane::None;
                            app.state.pane = Pane::Navigation;
                        }
                    }
                    ScreenPane::None => {}
                },
                _ => {}
            }
        }
    }
}

impl RenderScreen for Dashboard {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.sessions_table(app), area);
    }
}

impl Dashboard {
    fn sessions_table(&self, app: &mut App) -> impl Widget {
        let colors = &app.config.colors;

        let sessions_rows = self
            .sessions
            .iter()
            .map(|s| {
                Row::new(vec![
                    Cell::new(s.id.to_string()),
                    Cell::new(" "),
                    Cell::new(s.started.to_string()),
                    Cell::new(if let Some(ended) = &s.ended {
                        ended.to_string()
                    } else {
                        "empty".to_string()
                    }),
                ])
            })
            .collect::<Vec<Row>>();
        let sessions_widths = [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(25),
            Constraint::Length(25),
        ];
        Table::new(sessions_rows, sessions_widths)
            .header(
                Row::new(vec![
                    Cell::new("Session ID"),
                    Cell::new("Duration"),
                    Cell::new("Started"),
                    Cell::new("Ended"),
                ])
                .style(Style::new().bold().fg(colors.secondary)),
            )
            .block(
                Block::new()
                    .padding(Padding::horizontal(2))
                    .title(Title::from("List of Sessions"))
                    .title_style(Style::new().bold().fg(colors.primary))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(
                        Style::new().fg(if self.screen_pane == ScreenPane::Sessions {
                            colors.primary
                        } else {
                            colors.border
                        }),
                    ),
            )
    }
}
