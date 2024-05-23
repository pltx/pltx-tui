use chrono::{DateTime, Timelike, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Pane, App, Module};
use pltx_utils::{db_datetime_option, display_timestamp_seconds};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Cell, Padding, Row, Table, Widget},
    Frame,
};

struct Session {
    id: i32,
    started: Option<DateTime<Utc>>,
    ended: Option<DateTime<Utc>>,
    is_current: bool,
}

#[derive(PartialEq, Clone)]
enum DashboardPane {
    Sessions,
    None,
}

pub struct Dashboard {
    sessions: Vec<Session>,
    pane: DashboardPane,
    pane_hover: DashboardPane,
}

impl Module for Dashboard {
    fn init(app: &App) -> Dashboard {
        let query = "SELECT id, started, ended FROM session ORDER BY started DESC LIMIT 20";
        let conn = app.db.conn();
        let mut stmt = conn.prepare(query).unwrap();
        let sessions_iter = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    // Parsed as Option<DateTime<Utc>> instead of DateTime<Utc> because the current
                    // session won't have a ended datetime when this data is fetched.
                    started: db_datetime_option(row.get(1)?),
                    ended: db_datetime_option(row.get(2)?),
                    is_current: false,
                })
            })
            .unwrap();

        let mut sessions = Vec::new();
        for s in sessions_iter {
            let mut session = s.unwrap();
            if app.db.session_id.is_some_and(|id| id == session.id) {
                session.is_current = true;
            }

            sessions.push(session)
        }

        Dashboard {
            sessions,
            pane: DashboardPane::None,
            pane_hover: DashboardPane::None,
        }
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        if app.display.is_default() && app.pane == Pane::Module {
            match key_event.code {
                KeyCode::Enter => match self.pane_hover {
                    DashboardPane::None => self.pane = DashboardPane::Sessions,
                    DashboardPane::Sessions => {}
                },
                KeyCode::Backspace => match self.pane {
                    DashboardPane::Sessions => {
                        if app.pane == Pane::Module {
                            self.pane = DashboardPane::None;
                            app.pane = Pane::Navigation;
                        }
                    }
                    DashboardPane::None => {}
                },
                _ => {}
            }
        }
    }

    fn render(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.sessions_table(app), area);
    }
}

impl Dashboard {
    fn sessions_table(&self, app: &App) -> impl Widget {
        let colors = &app.config.colors;

        let sessions_rows = self
            .sessions
            .iter()
            .map(|s| {
                Row::new(vec![
                    Cell::new(s.id.to_string()),
                    Cell::new(if let Some(started) = s.started {
                        if let Some(ended) = s.ended {
                            let days = ended.hour().saturating_sub(started.hour());
                            let hours = ended.hour().saturating_sub(started.hour());
                            let minutes = ended.minute().saturating_sub(started.minute());
                            let seconds = ended.second().saturating_sub(started.second());
                            if days > 0 {
                                format!("{}d {}h {}m", days, hours, minutes)
                            } else if hours > 0 {
                                format!("{}h {}m {}s", hours, minutes, seconds)
                            } else if minutes != 0 {
                                format!("{}m {}s", minutes, seconds)
                            } else {
                                format!("{}s", seconds)
                            }
                        } else {
                            "<unknown>".to_string()
                        }
                    } else {
                        "<pending>".to_string()
                    }),
                    Cell::new(if let Some(started) = s.started {
                        display_timestamp_seconds(started)
                    } else {
                        "<pending>".to_string()
                    }),
                    Cell::new(if s.is_current {
                        "<current>".to_string()
                    } else if let Some(ended) = s.ended {
                        display_timestamp_seconds(ended)
                    } else {
                        "<empty>".to_string()
                    }),
                ])
            })
            .collect::<Vec<Row>>();
        let sessions_widths = [
            Constraint::Length(4),
            Constraint::Length(13),
            Constraint::Length(21),
            Constraint::Length(21),
        ];
        Table::new(sessions_rows, sessions_widths)
            .header(
                Row::new(vec![
                    Cell::new("ID"),
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
                    .border_style(Style::new().fg(if self.pane == DashboardPane::Sessions {
                        colors.primary
                    } else {
                        colors.border
                    })),
            )
    }
}
