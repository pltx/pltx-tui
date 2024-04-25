use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
    Frame,
};

use super::{projects::ProjectsState, screen::ScreenPane};
use crate::{
    components::TextInput,
    config::ColorsConfig,
    state::{Mode, State},
    utils::{Init, KeyEventHandlerReturn, RenderPage},
    App,
};

#[derive(PartialEq)]
enum FocusedPane {
    Title,
    Description,
    Actions,
}

#[derive(PartialEq)]
enum Action {
    Save,
    Cancel,
}

struct Inputs {
    title: TextInput,
    description: TextInput,
}

struct ProjectData {
    id: i32,
    title: String,
    description: Option<String>,
}

pub struct ProjectEditor {
    new: bool,
    data: Option<ProjectData>,
    focused_pane: FocusedPane,
    action: Action,
    inputs: Inputs,
}

impl Init for ProjectEditor {
    fn init(_: &mut App) -> ProjectEditor {
        ProjectEditor {
            new: false,
            data: None,
            focused_pane: FocusedPane::Title,
            action: Action::Save,
            inputs: Inputs {
                title: TextInput::new().set_title("Title").set_max(100),
                description: TextInput::new().set_title("Description").set_max(500),
            },
        }
    }
}

impl ProjectEditor {
    fn db_new_project(&self, app: &mut App) -> rusqlite::Result<()> {
        struct Highest {
            position: i32,
        }

        let mut stmt = app.db.conn.prepare(
            "SELECT position from project WHERE position = (SELECT MAX(position) FROM project)",
        )?;
        let highest = stmt
            .query_row([], |r| {
                Ok(Highest {
                    position: r.get(0)?,
                })
            })
            .unwrap_or(Highest { position: -1 });

        let mut description = Some(&self.inputs.description.input[0]);
        if self.inputs.description.input[0].chars().count() == 0 {
            description = None;
        }

        app.db.conn.execute(
            "INSERT INTO project (title, description, position) VALUES (?1, ?2, ?3)",
            (
                &self.inputs.title.input[0],
                description,
                highest.position + 1,
            ),
        )?;

        Ok(())
    }

    fn db_edit_project(&self, app: &mut App) -> rusqlite::Result<()> {
        if let Some(data) = &self.data {
            let query = "UPDATE project SET title = ?1, description = ?2 WHERE id = ?3";
            let mut stmt = app.db.conn.prepare(query)?;
            stmt.execute(rusqlite::params![
                &self.inputs.title.input[0],
                &self.inputs.description.input[0],
                data.id,
            ])?;
        } else {
            panic!("project data was not set")
        }

        Ok(())
    }
}

impl KeyEventHandlerReturn<bool> for ProjectEditor {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) -> bool {
        match self.focused_pane {
            FocusedPane::Title => self.inputs.title.handle_key_event(app, key_event),
            FocusedPane::Description => self.inputs.description.handle_key_event(app, key_event),
            _ => {}
        }

        if app.state.mode == Mode::Navigation {
            match key_event.code {
                KeyCode::Char('n') | KeyCode::Char('e') => {
                    if self.focused_pane == FocusedPane::Title {
                        app.state.mode = Mode::Insert
                    }
                }
                KeyCode::Char('q') => self.reset(),
                KeyCode::Char('j') => self.prev_pane(),
                KeyCode::Char('k') => self.next_pane(),
                KeyCode::Enter => {
                    if self.save_project(app) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl ProjectEditor {
    fn prev_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => self.focused_pane = FocusedPane::Description,
            FocusedPane::Description => self.focused_pane = FocusedPane::Actions,
            FocusedPane::Actions => {
                if self.action == Action::Save {
                    self.action = Action::Cancel;
                }
            }
        }
    }

    fn next_pane(&mut self) {
        match self.focused_pane {
            FocusedPane::Title => {}
            FocusedPane::Description => self.focused_pane = FocusedPane::Title,
            FocusedPane::Actions => {
                if self.action == Action::Save {
                    self.focused_pane = FocusedPane::Description;
                } else if self.action == Action::Cancel {
                    self.action = Action::Save;
                }
            }
        }
    }

    fn save_project(&mut self, app: &mut App) -> bool {
        if self.focused_pane == FocusedPane::Actions {
            if self.action == Action::Save {
                if self.new {
                    self.db_new_project(app).unwrap_or_else(|e| panic!("{e}"));
                } else {
                    self.db_edit_project(app).unwrap_or_else(|e| panic!("{e}"));
                }
                self.reset()
            } else if self.action == Action::Cancel {
                self.reset()
            }
            return true;
        }
        false
    }
}

impl RenderPage<ProjectsState> for ProjectEditor {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect, state: ProjectsState) {
        let colors = &app.config.colors.clone();
        let main_sp = state.screen_pane == ScreenPane::Main;

        let block = Block::new()
            .title(if self.new {
                " New Project "
            } else {
                " Edit Project "
            })
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(colors.border));
        frame.render_widget(block, area);

        let [title_layout, description_layout, actions_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(6),
            ])
            .areas(area);

        frame.render_widget(self.title(app, title_layout, main_sp), title_layout);
        frame.render_widget(
            self.description(app, description_layout, main_sp),
            description_layout,
        );
        let actions = self.actions(colors, actions_layout, main_sp);
        frame.render_widget(Block::new(), actions.1 .0);
        frame.render_widget(actions.0, actions.1 .1);
        frame.render_widget(Block::new(), actions.1 .0);
    }
}

impl ProjectEditor {
    fn title(&self, app: &mut App, area: Rect, main_sp: bool) -> impl Widget {
        let focused = self.focused_pane == FocusedPane::Title && main_sp;
        self.inputs
            .title
            .render(app, area.width - 2, area.height - 2, focused)
    }

    fn description(&self, app: &mut App, area: Rect, main_sp: bool) -> impl Widget {
        let focused = self.focused_pane == FocusedPane::Description && main_sp;
        self.inputs
            .description
            .render(app, area.width - 2, area.height - 2, focused)
    }

    fn actions(
        &self,
        colors: &ColorsConfig,
        area: Rect,
        main_sp: bool,
    ) -> (impl Widget, (Rect, Rect, Rect)) {
        let width = 30;
        let [space_1, layout, space_2] = Layout::default()
            .vertical_margin(1)
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width - width) / 2),
                Constraint::Length(width),
                Constraint::Length((area.width - width) / 2),
            ])
            .areas(area);

        (
            Paragraph::new(Text::from(vec![
                Line::styled(
                    if self.new {
                        " Create New Project "
                    } else {
                        " Save Project "
                    },
                    if self.focused_pane == FocusedPane::Actions {
                        if self.action == Action::Save {
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
                    .border_style(Style::new().fg(
                        if self.focused_pane == FocusedPane::Actions && main_sp {
                            colors.primary
                        } else {
                            colors.border
                        },
                    )),
            ),
            (space_1, layout, space_2),
        )
    }

    pub fn set_new(mut self) -> Self {
        self.new = true;
        self
    }

    pub fn set_project(&mut self, app: &App, project_id: i32) -> rusqlite::Result<()> {
        let project_query = "SELECT id, title, description FROM project WHERE id = ?1";
        let mut project_stmt = app.db.conn.prepare(project_query)?;
        let project = project_stmt.query_row([project_id], |r| {
            Ok(ProjectData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
            })
        })?;

        self.data = Some(ProjectData {
            id: project.id,
            title: project.title.clone(),
            description: project.description.clone(),
        });

        self.inputs.title.set_input(vec![project.title]);
        self.inputs.title.cursor_end_line();
        self.inputs
            .description
            .set_input(if let Some(desc) = project.description {
                vec![desc]
            } else {
                vec![String::from("")]
            });
        self.inputs.description.cursor_end_line();

        Ok(())
    }

    fn reset(&mut self) {
        self.focused_pane = FocusedPane::Title;
        self.action = Action::Save;
        self.inputs.title.reset();
        self.inputs.description.reset();
    }
}
