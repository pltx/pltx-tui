use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
    Frame,
};

use crate::{
    components::{self, TextInput},
    config::ColorsConfig,
    state::{Mode, State},
    utils::{Init, KeyEventHandlerReturn, RenderPopup},
    App,
};

#[derive(PartialEq)]
enum FocusedPane {
    Title,
    Actions,
}

#[derive(PartialEq)]
enum Action {
    Create,
    Cancel,
}

struct Inputs {
    title: TextInput,
}

pub struct NewList {
    pub width: u16,
    pub height: u16,
    project_id: Option<i32>,
    focused_pane: FocusedPane,
    action: Action,
    inputs: Inputs,
}

impl Init for NewList {
    fn init(_: &mut crate::App) -> NewList {
        NewList {
            width: 60,
            height: 9,
            project_id: None,
            focused_pane: FocusedPane::Title,
            action: Action::Create,
            inputs: Inputs {
                title: TextInput::new().set_title("Title").set_max(50),
            },
        }
    }
}

impl NewList {
    pub fn set_project_id(&mut self, project_id: i32) {
        self.project_id = Some(project_id)
    }

    fn db_new_list(&self, app: &mut App) -> rusqlite::Result<()> {
        if self.project_id.is_none() {
            panic!("project_id was not set")
        }

        struct ProjectQuery {
            position: i32,
        }
        let mut stmt = app.db.conn.prepare("SELECT position from project_list")?;
        let project_iter = stmt.query_map([], |r| {
            Ok(ProjectQuery {
                position: r.get(0)?,
            })
        })?;
        let mut highest_position = 0;
        for project in project_iter {
            let project_pos = project.unwrap().position;
            if project_pos > highest_position {
                highest_position = project_pos;
            }
        }
        app.db.conn.execute(
            "INSERT INTO project_list (project_id, title, position) VALUES (?1, ?2, ?3)",
            (
                Some(&self.project_id),
                &self.inputs.title.input,
                highest_position,
            ),
        )?;
        Ok(())
    }
}

impl KeyEventHandlerReturn<bool> for NewList {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) -> bool {
        if self.focused_pane == FocusedPane::Title {
            self.inputs.title.handle_key_event(app, key_event)
        }

        if app.state.mode == Mode::Popup {
            match key_event.code {
                KeyCode::Char('j') => match self.focused_pane {
                    FocusedPane::Title => self.focused_pane = FocusedPane::Actions,
                    FocusedPane::Actions => {
                        if self.action == Action::Create {
                            self.action = Action::Cancel;
                        }
                    }
                },
                KeyCode::Char('k') => match self.focused_pane {
                    FocusedPane::Title => {}
                    FocusedPane::Actions => {
                        if self.action == Action::Create {
                            self.focused_pane = FocusedPane::Title;
                        } else if self.action == Action::Cancel {
                            self.action = Action::Create;
                        }
                    }
                },
                KeyCode::Enter => {
                    if self.focused_pane == FocusedPane::Actions {
                        if self.action == Action::Create {
                            self.db_new_list(app).unwrap_or_else(|e| panic!("{e}"));
                            app.state.mode = Mode::Navigation;
                            self.inputs.title.reset();
                        } else if self.action == Action::Cancel {
                            app.state.mode = Mode::Navigation;
                            self.inputs.title.reset();
                        }
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl RenderPopup for NewList {
    fn render(&mut self, frame: &mut Frame, app: &App) {
        let popup = components::Popup::new(app, frame.size())
            .set_title_top("New List")
            .set_size(self.width, self.height)
            .render(frame);

        let colors = &app.config.colors.clone();

        let [title_layout, actions_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Length(3), Constraint::Length(4)])
            .areas(popup.area);

        frame.render_widget(self.title(app), title_layout);
        frame.render_widget(self.actions(colors), actions_layout);
    }
}

impl NewList {
    fn title(&self, app: &App) -> impl Widget {
        let focused = self.focused_pane == FocusedPane::Title;
        self.inputs.title.render(app, focused)
    }

    fn actions(&self, colors: &ColorsConfig) -> impl Widget {
        Paragraph::new(Text::from(vec![
            Line::styled(
                " Create New List ",
                if self.focused_pane == FocusedPane::Actions {
                    if self.action == Action::Create {
                        Style::new()
                            .bold()
                            .fg(colors.active_fg)
                            .bg(colors.active_bg)
                    } else {
                        Style::new().fg(colors.secondary)
                    }
                } else {
                    Style::new().fg(colors.secondary)
                },
            ),
            Line::styled(
                " Cancel ",
                if self.focused_pane == FocusedPane::Actions {
                    if self.action == Action::Cancel {
                        Style::new()
                            .bold()
                            .fg(colors.active_fg)
                            .bg(colors.active_bg)
                    } else {
                        Style::new().fg(colors.secondary)
                    }
                } else {
                    Style::new().fg(colors.secondary)
                },
            ),
        ]))
        .centered()
        .block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::new().fg(if self.focused_pane == FocusedPane::Actions {
                        colors.primary
                    } else {
                        colors.border
                    }),
                ),
        )
    }
}
