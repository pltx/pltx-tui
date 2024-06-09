use std::{collections::HashSet, str::FromStr, time::Instant};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, DefaultWidget, KeyEventHandler, Popup};
use pltx_database::Database;
use pltx_utils::{DateTime, WidgetMargin};
use pltx_widgets::{PopupSize, PopupWidget, Selection};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use tracing::{info, info_span};

use crate::open_project::ProjectLabel;

struct Subtask {
    id: i32,
    value: String,
    completed: bool,
}

struct CardData {
    id: i32,
    title: String,
    description: Option<String>,
    start_date: Option<DateTime>,
    due_date: Option<DateTime>,
    reminder: Option<DateTime>,
    position: i32,
    created_at: DateTime,
    updated_at: DateTime,
    labels: HashSet<i32>,
    subtasks: Vec<Subtask>,
}

pub struct CardViewer {
    id: Option<i32>,
    data: Option<CardData>,
    subtasks_selection: Selection<i32>,
    labels: Vec<ProjectLabel>,
}

impl Popup<Result<bool>> for CardViewer {
    fn init() -> Self {
        Self {
            id: None,
            data: None,
            subtasks_selection: Selection::new("Subtasks", vec![]).checklist(),
            labels: vec![],
        }
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        let _span = info_span!("project management", popup = "card viewer").entered();
        self.subtasks_selection.key_event_handler(app, key_event);

        match key_event.code {
            KeyCode::Char('q') => {
                app.view.default();
                self.reset();
                return Ok(false);
            }
            KeyCode::Char(' ') => {
                self.db_update_subtasks(&app.db)?;
                return Ok(true);
            }
            KeyCode::Char('i') => {
                self.db_update_subtasks(&app.db)?;
                return Ok(true);
            }
            KeyCode::Char('a') => {
                self.db_update_subtasks(&app.db)?;
                return Ok(true);
            }
            _ => {}
        }

        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        if let Some(data) = &self.data {
            let popup = PopupWidget::new(app, area)
                .title_top(&data.title)
                .size(
                    PopupSize::default()
                        .percentage_based_height()
                        .width(100)
                        .height(90),
                )
                .render(frame);

            let area = WidgetMargin::proportional(1).apply(popup.sub_area);

            let description_lines = if let Some(description) = &data.description {
                description.chars().count().div_ceil(area.width as usize) as u16
            } else {
                1
            };

            let spacing = 1;
            let [description_area, labels_area, subtasks_area, dates_area, metadata_area] =
                Layout::default()
                    .constraints([
                        Constraint::Length(description_lines + spacing),
                        Constraint::Length(1 + spacing),
                        Constraint::Length(if data.subtasks.is_empty() {
                            0
                        } else if data.subtasks.len() as u16 <= 5 {
                            data.subtasks.len() as u16 + spacing
                        } else {
                            5 + spacing
                        }),
                        Constraint::Length(3 + spacing),
                        Constraint::Length(4),
                    ])
                    .areas(area);

            let line_length = description_area.width as usize;
            let mut first_line_length = line_length;
            let description = if let Some(desc) = &data.description {
                if desc.chars().count() <= first_line_length {
                    first_line_length = 0;
                }
                desc[first_line_length..]
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(line_length)
                    .enumerate()
                    .flat_map(|(i, c)| {
                        let mut text = vec![];
                        if i == 0 {
                            text.push(Line::from(vec![Span::from(
                                desc[..first_line_length].to_string(),
                            )
                            .fg(colors.fg)]))
                        }
                        text.push(
                            Line::from(Span::from(c.iter().collect::<String>().trim().to_owned()))
                                .fg(colors.fg),
                        );
                        text
                    })
                    .collect::<Vec<Line>>()
            } else {
                vec![Line::from(vec![Span::styled(
                    "<empty description>",
                    Style::new().fg(colors.tertiary_fg),
                )])]
            };

            frame.render_widget(Paragraph::new(description), description_area);

            let labels = Paragraph::new(Line::from(
                self.labels
                    .iter()
                    .filter(|l| data.labels.contains(&l.id))
                    .enumerate()
                    .flat_map(|(i, l)| {
                        let mut span = vec![Span::from(l.title.to_string())
                            .fg(Color::from_str(&l.color).expect("failed to parse label color"))];
                        if i != 0 {
                            span.insert(0, Span::from(", ").fg(colors.secondary_fg));
                        }
                        span
                    })
                    .collect::<Vec<Span>>(),
            ));

            frame.render_widget(labels, labels_area);

            self.subtasks_selection
                .render(frame, app, subtasks_area, true);

            let dates = Paragraph::new(vec![
                Line::from(vec![
                    Span::from("Start Date: "),
                    if let Some(start_date) = &data.start_date {
                        Span::from(start_date.display())
                    } else {
                        Span::from("<empty>").fg(colors.tertiary_fg)
                    },
                ]),
                Line::from(vec![
                    Span::from("Due Date: "),
                    if let Some(due_date) = &data.due_date {
                        Span::from(due_date.display())
                    } else {
                        Span::from("<empty>").fg(colors.tertiary_fg)
                    },
                ]),
                Line::from(vec![
                    Span::from("Reminder: "),
                    if let Some(reminder) = &data.reminder {
                        Span::from(reminder.display())
                    } else {
                        Span::from("<empty>").fg(colors.tertiary_fg)
                    },
                ]),
            ])
            .fg(colors.secondary_fg);

            frame.render_widget(dates, dates_area);

            let metadata = Paragraph::new(vec![
                Line::from(vec![
                    Span::from("ID: ").bold(),
                    Span::from(data.id.to_string()),
                ]),
                Line::from(vec![
                    Span::from("Position in List: ").bold(),
                    Span::from(data.position.to_string()),
                ]),
                Line::from(vec![
                    Span::from("Created At: ").bold(),
                    Span::from(data.created_at.display_with_seconds()),
                ]),
                Line::from(vec![
                    Span::from("Updated: At: ").bold(),
                    Span::from(data.updated_at.display_with_seconds()),
                ]),
            ])
            .fg(colors.secondary_fg);

            frame.render_widget(metadata, metadata_area);
        }
    }
}

impl CardViewer {
    pub fn labels(&mut self, labels: Vec<ProjectLabel>) {
        self.labels = labels;
    }

    pub fn id(&mut self, card_id: i32) {
        self.id = Some(card_id);
    }

    pub fn set_data(&mut self, db: &Database, card_id: i32) -> Result<()> {
        let start = Instant::now();

        let conn = db.conn();

        let query_start = Instant::now();
        let query = "SELECT id, title, description, start_date, due_date, reminder, position, \
                     created_at, updated_at FROM project_card WHERE id = ?1";
        let mut stmt = conn.prepare(query)?;
        let mut card = stmt.query_row([card_id], |r| {
            Ok(CardData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                start_date: DateTime::from_db_option(r.get(3)?),
                due_date: DateTime::from_db_option(r.get(4)?),
                reminder: DateTime::from_db_option(r.get(5)?),
                position: r.get(6)?,
                created_at: DateTime::from_db(r.get(7)?),
                updated_at: DateTime::from_db(r.get(8)?),
                labels: HashSet::new(),
                subtasks: vec![],
            })
        })?;
        info!(
            "get card data query executed in {:?}",
            query_start.elapsed()
        );

        self.db_get_card_labels(db, &mut card)?;
        self.db_get_subtasks(db, &mut card)?;
        self.data = Some(card);

        info!("set card data in {:?}", start.elapsed());

        Ok(())
    }

    fn db_get_card_labels(&mut self, db: &Database, data: &mut CardData) -> Result<()> {
        let start = Instant::now();
        let conn = db.conn();
        let query = "SELECT label_id from card_label WHERE card_id = ?1";
        let mut stmt = conn.prepare(query)?;
        let label_id_iter = stmt.query_map([data.id], |r| r.get::<usize, i32>(0))?;

        for label in label_id_iter {
            let label_id = label?;
            data.labels.insert(label_id);
        }

        info!("get card labels query executed in {:?}", start.elapsed());

        Ok(())
    }

    fn db_get_subtasks(&mut self, db: &Database, data: &mut CardData) -> Result<()> {
        let start = Instant::now();
        let conn = db.conn();
        let query = "SELECT id, value, completed FROM card_subtask WHERE card_id = ?1";
        let mut stmt = conn.prepare(query)?;
        let subtask_iter = stmt.query_map([data.id], |r| {
            Ok(Subtask {
                id: r.get(0)?,
                value: r.get(1)?,
                completed: r.get(2)?,
            })
        })?;

        for (i, st) in subtask_iter.enumerate() {
            let st = st?;
            data.subtasks.push(Subtask {
                id: st.id,
                value: st.value.clone(),
                completed: st.completed,
            });

            self.subtasks_selection
                .options
                .push((st.id, Span::from(st.value)));

            if st.completed {
                self.subtasks_selection.selected.insert(i);
            }
        }

        info!("get card subtasks query executed in {:?}", start.elapsed());

        Ok(())
    }

    pub fn reset(&mut self) {
        self.data = None;
        self.subtasks_selection.reset();
        self.subtasks_selection.options.clear();
    }

    fn db_update_subtasks(&self, db: &Database) -> Result<()> {
        let start = Instant::now();
        if let Some(data) = &self.data {
            for (_, subtask) in data.subtasks.iter().enumerate().filter(|(i, st)| {
                (st.completed && !self.subtasks_selection.selected.contains(i))
                    || (!st.completed && self.subtasks_selection.selected.contains(i))
            }) {
                let query = "UPDATE card_subtask SET completed = ?1, updated_at = ?2 WHERE \
                             card_id = ?3 AND id = ?4";
                let params = (
                    !subtask.completed,
                    DateTime::now(),
                    self.id.unwrap(),
                    subtask.id,
                );
                db.execute(query, params)?;
            }
        }
        info!(
            "update card subtasks query executed in {:?}",
            start.elapsed()
        );
        Ok(())
    }
}
