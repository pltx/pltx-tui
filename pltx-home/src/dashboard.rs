use std::vec;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, DefaultWidget, KeyEventHandler, Screen};
use pltx_utils::{symbols, DateTime, WidgetMargin};
use pltx_widgets::{CardCell, CardLayout, CardRow, Scrollable};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
    Frame,
};

const CARDS_MAX_WIDTH: u16 = 200;
const SMALL_HEIGHT: u16 = 35;
const MEDIUM_HEIGHT: u16 = 45;

struct Session {
    id: i32,
    started: Option<DateTime>,
    ended: Option<DateTime>,
    is_current: bool,
}

#[derive(PartialEq, Clone)]
enum Pane {
    Sessions,
    Tasks,
    Calendar,
}

pub struct Dashboard {
    pane: Pane,
    sessions: Vec<Session>,
    scrollable_sessions: Scrollable,
}

impl Screen for Dashboard {
    fn init(app: &App) -> Result<Self> {
        let sessions = Dashboard::db_get_sessions(app)?;

        Ok(Self {
            pane: Pane::Sessions,
            sessions,
            scrollable_sessions: Scrollable::default().cols([5, 10, 21, 21]),
        })
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        if self.pane == Pane::Sessions {
            self.scrollable_sessions.key_event_handler(app, key_event);
        }

        if app.display.is_default() {
            match key_event.code {
                KeyCode::Tab => {
                    self.pane = match self.pane {
                        Pane::Sessions => Pane::Tasks,
                        Pane::Tasks => Pane::Calendar,
                        Pane::Calendar => Pane::Sessions,
                    }
                }
                KeyCode::BackTab => {
                    self.pane = match self.pane {
                        Pane::Sessions => Pane::Calendar,
                        Pane::Tasks => Pane::Sessions,
                        Pane::Calendar => Pane::Tasks,
                    }
                }
                _ => {}
            }
        }
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let welcome_height = if area.height < SMALL_HEIGHT {
            7
        } else if area.height < MEDIUM_HEIGHT {
            9
        } else {
            11
        };

        let [welcome_layout, cards_layout] = Layout::default()
            .constraints([Constraint::Length(welcome_height), Constraint::Fill(1)])
            .areas(area);

        self.render_welcome(app, frame, welcome_layout, area);
        self.render_cards(app, frame, cards_layout, area);
    }
}

impl Dashboard {
    fn db_get_sessions(app: &App) -> Result<Vec<Session>> {
        let query = "SELECT id, started, ended FROM session ORDER BY started DESC LIMIT 20";
        let conn = app.db.conn();
        let mut stmt = conn.prepare(query)?;
        let sessions_iter = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                started: DateTime::from_db_option(row.get(1)?),
                ended: DateTime::from_db_option(row.get(2)?),
                is_current: false,
            })
        })?;

        let mut sessions = Vec::new();
        for s in sessions_iter {
            let mut session = s?;
            if app.db.session_id.is_some_and(|id| id == session.id) {
                session.is_current = true;
            }

            sessions.push(session)
        }

        Ok(sessions)
    }

    fn render_welcome(&self, app: &App, frame: &mut Frame, area: Rect, parent_area: Rect) {
        let vertical_spacing = if parent_area.height < SMALL_HEIGHT {
            1
        } else if parent_area.height < MEDIUM_HEIGHT {
            2
        } else {
            3
        };

        let [top_space, center_layout, bottom_space] = Layout::default()
            .constraints([
                Constraint::Length(vertical_spacing),
                Constraint::Fill(1),
                Constraint::Length(vertical_spacing),
            ])
            .areas(area);

        frame.render_widget(self.render_dots(top_space, app), top_space);
        frame.render_widget(self.render_dots(bottom_space, app), bottom_space);

        let [left_space, content_layout, right_space] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .areas(center_layout);

        frame.render_widget(self.render_dots(left_space, app), left_space);
        frame.render_widget(
            self.render_welcome_content(app, content_layout),
            content_layout,
        );
        frame.render_widget(self.render_dots(right_space, app), right_space);
    }

    fn render_welcome_content(&self, app: &App, area: Rect) -> impl Widget {
        let home = &app.config.modules.home;
        let colors = &app.config.colors;
        let padding = 1;

        Paragraph::new(vec![
            Line::from(home.dashboard_title.to_string()).bold(),
            Line::from(
                symbols::border::HORIZONTAL
                    .repeat((area.width as usize).saturating_sub(padding * 2 + 8)),
            )
            .fg(colors.border),
            Line::from(home.dashboard_message.to_string()).fg(colors.highlight_fg),
        ])
        .block(Block::new().padding(Padding::uniform(padding as u16)))
        .centered()
    }

    fn render_dots(&self, area: Rect, app: &App) -> impl Widget {
        Paragraph::new(
            (0..area.height)
                .map(|_| {
                    Line::from(
                        (0..area.width)
                            .map(|i| if i % 2 == 0 { "覆" } else { "複" })
                            .collect::<Vec<&str>>()
                            .join(""),
                    )
                })
                .collect::<Vec<Line>>(),
        )
        .fg(app.config.colors.tertiary_fg)
    }

    fn render_cards(&self, app: &App, frame: &mut Frame, area: Rect, parent_area: Rect) {
        let row_1 = CardRow::new(vec![
            CardCell::new("Sessions").focused(self.pane == Pane::Sessions),
            CardCell::new("Tasks").focused(self.pane == Pane::Tasks),
            CardCell::new("Calendar").focused(self.pane == Pane::Calendar),
        ])
        .height(13);

        let row_2 = CardRow::new(vec![
            CardCell::new("Screentime").constraint(Constraint::Percentage(75)),
            CardCell::new("App Info").constraint(Constraint::Percentage(25)),
        ])
        .height(13);

        let margin = if parent_area.height < SMALL_HEIGHT {
            0
        } else if parent_area.height < MEDIUM_HEIGHT {
            1
        } else {
            2
        };

        let card_layout = CardLayout::new([row_1, row_2])
            .max_width(CARDS_MAX_WIDTH)
            .row_margin(WidgetMargin::top(margin))
            .card_margin(WidgetMargin::horizontal(margin));

        card_layout.render(frame, app, area, true);

        let [row_1_layouts, row_2_layouts] = card_layout.layouts(area);

        self.render_sessions(frame, app, row_1_layouts[0]);
        frame.render_widget(self.render_tasks(app), row_1_layouts[1]);
        frame.render_widget(self.render_calendar(app), row_1_layouts[2]);
        frame.render_widget(self.render_screentime(app), row_2_layouts[0]);
        frame.render_widget(self.render_app_info(app), row_2_layouts[1]);
    }

    fn render_sessions(&self, frame: &mut Frame, app: &App, area: Rect) {
        let colors = &app.config.colors;

        let header = [
            Paragraph::new("ID").bold(),
            Paragraph::new("Duration").bold(),
            Paragraph::new("Started").bold(),
            Paragraph::new("Ended").bold(),
        ];

        let table = self
            .sessions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                vec![
                    Paragraph::new(s.id.to_string()).fg(colors.fg).bg(
                        if self.pane == Pane::Sessions && self.scrollable_sessions.focused == i {
                            colors.input_focus_bg
                        } else {
                            colors.bg
                        },
                    ),
                    Paragraph::new(if let Some(started) = &s.started {
                        if s.is_current {
                            DateTime::new().duration_since(started).to_string()
                        } else if let Some(ended) = &s.ended {
                            ended.duration_since(started).to_string()
                        } else {
                            "<unknown>".to_string()
                        }
                    } else {
                        "<pending>".to_string()
                    })
                    .fg(colors.success)
                    .bg(
                        if self.pane == Pane::Sessions && self.scrollable_sessions.focused == i {
                            colors.input_focus_bg
                        } else {
                            colors.bg
                        },
                    ),
                    Paragraph::new(if let Some(started) = &s.started {
                        Line::from(vec![
                            Span::from(started.display_date()).fg(colors.date_fg),
                            Span::from(" "),
                            Span::from(started.display_time_with_seconds()).fg(colors.time_fg),
                        ])
                    } else {
                        Line::from("<pending>".to_string())
                    })
                    .fg(if s.started.is_some() {
                        colors.date_fg
                    } else {
                        colors.secondary_fg
                    })
                    .bg(
                        if self.pane == Pane::Sessions && self.scrollable_sessions.focused == i {
                            colors.input_focus_bg
                        } else {
                            colors.bg
                        },
                    ),
                    Paragraph::new(if s.is_current {
                        Line::from(vec![
                            Span::from(DateTime::new().display_date()).fg(colors.date_fg),
                            Span::from(" "),
                            Span::from(DateTime::new().display_time_with_seconds())
                                .fg(colors.time_fg),
                        ])
                    } else if let Some(ended) = &s.ended {
                        Line::from(vec![
                            Span::from(ended.display_date()).fg(colors.date_fg),
                            Span::from(" "),
                            Span::from(ended.display_time_with_seconds()).fg(colors.time_fg),
                        ])
                    } else {
                        Line::from("<empty>".to_string())
                    })
                    .fg(if s.ended.is_some() {
                        colors.time_fg
                    } else {
                        colors.secondary_fg
                    })
                    .bg(
                        if self.pane == Pane::Sessions && self.scrollable_sessions.focused == i {
                            colors.input_focus_bg
                        } else {
                            colors.bg
                        },
                    ),
                ]
            })
            .collect::<Vec<Vec<Paragraph>>>();

        self.scrollable_sessions
            .render_with_cols(frame, area, header.into(), table);
    }

    fn render_tasks(&self, app: &App) -> impl Widget {
        let colors = &app.config.colors;

        let tasks = vec![Line::from("You don't have any planned tasks.").fg(colors.secondary_fg)];
        Paragraph::new(tasks)
    }

    fn render_calendar(&self, app: &App) -> impl Widget {
        let colors = &app.config.colors;

        let tasks = vec![Line::from("You don't have any planned events.").fg(colors.secondary_fg)];
        Paragraph::new(tasks)
    }

    fn render_screentime(&self, app: &App) -> impl Widget {
        let colors = &app.config.colors;

        let tasks = vec![Line::from("Coming soon!").fg(colors.secondary_fg)];
        Paragraph::new(tasks)
    }

    fn render_app_info(&self, app: &App) -> impl Widget {
        let colors = &app.config.colors;

        Paragraph::new(vec![
            Line::from(format!("Version: {}", env!("CARGO_PKG_VERSION"))),
            Line::from(vec![
                Span::from("Up to date: "),
                Span::from(symbols::CHECK).fg(colors.success),
            ]),
            Line::from(vec![
                Span::from("Auto service running: "),
                Span::from(symbols::CROSS).fg(colors.danger),
            ]),
            Line::from(vec![
                Span::from("Server storage: "),
                Span::from(symbols::CROSS).fg(colors.danger),
            ]),
            Line::from(vec![
                Span::from("Connected to server: "),
                Span::from(symbols::CROSS).fg(colors.danger),
            ]),
            Line::from(vec![
                Span::from("Encrypted: "),
                Span::from(symbols::CROSS).fg(colors.danger),
            ]),
        ])
    }
}
