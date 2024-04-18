use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{block::Title, Block, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::{database::Session, utils::RenderScreen, App};

pub struct Dashboard {
    sessions: Vec<Session>,
}

impl Dashboard {
    pub fn init(app: &mut App) -> Dashboard {
        let query = "SELECT id, started, ended FROM session";
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

        Dashboard { sessions }
    }
}

impl RenderScreen for Dashboard {
    fn render(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let text = Paragraph::new(format!("{}", area.height));
        frame.render_widget(text, area);

        let sessions_rows = self
            .sessions
            .iter()
            .map(|s| {
                Row::new(vec![
                    Cell::new(s.id.to_string()),
                    Cell::new(s.started.to_string()),
                    Cell::new(if let Some(ended) = &s.ended {
                        ended.to_string()
                    } else {
                        "empty".to_string()
                    }),
                ])
            })
            .collect::<Vec<Row>>();
        // Row::new(vec![Cell::new("test"), Cell::new("test2")])];
        let sessions_widths = [
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];
        let sessions_table = Table::new(sessions_rows, sessions_widths)
            .header(
                Row::new(vec![
                    Cell::new("ID"),
                    Cell::new("Started"),
                    Cell::new("Ended"),
                ])
                .style(Style::new().bold().fg(colors.secondary)),
            )
            .block(
                Block::new()
                    .title(Title::from("List of Sessions"))
                    .title_style(Style::new().bold().fg(colors.primary)),
            );
        frame.render_widget(sessions_table, area);
    }
}
