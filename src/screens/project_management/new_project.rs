use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
    Frame,
};

use super::{projects::ProjectsState, screen::ScreenPane};
use crate::{
    components::TextInput,
    config::ColorsConfig,
    state::{Mode, State},
    utils::{Init, KeyEventHandler, RenderPage},
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
    Create,
    Cancel,
}

struct Inputs {
    title: TextInput,
    description: TextInput,
}

pub struct NewProject {
    focused_pane: FocusedPane,
    action: Action,
    inputs: Inputs,
}

impl Init for NewProject {
    fn init(_: &mut App) -> NewProject {
        NewProject {
            focused_pane: FocusedPane::Title,
            action: Action::Create,
            inputs: Inputs {
                title: TextInput::new().set_title("Title").set_max(100),
                description: TextInput::new().set_title("Description").set_max(2000),
            },
        }
    }
}

impl KeyEventHandler for NewProject {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) {
        match self.focused_pane {
            FocusedPane::Title => self.inputs.title.handle_key_event(app, key_event),
            FocusedPane::Description => self.inputs.description.handle_key_event(app, key_event),
            _ => {}
        }

        if app.state.mode == Mode::Navigation {
            match key_event.code {
                KeyCode::Char('j') => match self.focused_pane {
                    FocusedPane::Title => self.focused_pane = FocusedPane::Description,
                    FocusedPane::Description => self.focused_pane = FocusedPane::Actions,
                    FocusedPane::Actions => {
                        if self.action == Action::Create {
                            self.action = Action::Cancel;
                        }
                    }
                },
                KeyCode::Char('k') => match self.focused_pane {
                    FocusedPane::Title => {}
                    FocusedPane::Description => self.focused_pane = FocusedPane::Title,
                    FocusedPane::Actions => {
                        if self.action == Action::Create {
                            self.focused_pane = FocusedPane::Description;
                        } else {
                            self.action = Action::Create;
                        }
                    }
                },
                _ => {}
            }
        }
    }
}

impl RenderPage<ProjectsState> for NewProject {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect, state: ProjectsState) {
        let colors = &app.config.colors.clone();
        let main_sp = state.screen_pane == ScreenPane::Main;

        let block = Block::new()
            .title(" New Project ")
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

        frame.render_widget(self.title(app, main_sp), title_layout);
        frame.render_widget(self.description(app, main_sp), description_layout);
        let actions = self.actions(colors, actions_layout, main_sp);
        frame.render_widget(Block::new(), actions.1 .0);
        frame.render_widget(actions.0, actions.1 .1);
        frame.render_widget(Block::new(), actions.1 .0);
    }
}

impl NewProject {
    fn title(&self, app: &mut App, main_sp: bool) -> impl Widget {
        let focused = self.focused_pane == FocusedPane::Title && main_sp;
        self.inputs.title.render(app, focused)
    }

    fn description(&self, app: &mut App, main_sp: bool) -> impl Widget {
        let focused = self.focused_pane == FocusedPane::Description && main_sp;
        self.inputs.description.render(app, focused)
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
                    " Create New Project ",
                    if self.focused_pane == FocusedPane::Actions {
                        if self.action == Action::Create {
                            Style::new().fg(colors.hover_fg).bg(colors.hover_bg)
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
                            Style::new().fg(colors.hover_fg).bg(colors.hover_bg)
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
}
