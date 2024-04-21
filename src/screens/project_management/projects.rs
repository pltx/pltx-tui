use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use super::{new_project::NewProject, screen::ScreenPane};
use crate::{
    state::{Mode, State},
    utils::{Init, KeyEventHandler, KeyEventHandlerReturn, RenderPage},
    App,
};

#[derive(PartialEq)]
enum Page {
    ListProjects,
    CreateProject,
    EditProject,
    OpenProject,
}

struct Pages {
    create_project: NewProject,
}

pub struct Projects {
    page: Page,
    pages: Pages,
}

impl Init for Projects {
    fn init(app: &mut App) -> Projects {
        Projects {
            page: Page::ListProjects,
            pages: Pages {
                create_project: NewProject::init(app),
            },
        }
    }
}

impl KeyEventHandler for Projects {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) {
        if app.state.mode == Mode::Navigation {
            match self.page {
                Page::ListProjects => match key_event.code {
                    KeyCode::Char('n') => {
                        self.page = Page::CreateProject;
                    }
                    KeyCode::Char('e') => self.page = Page::EditProject,
                    KeyCode::Char('d') => {}
                    _ => {}
                },
                Page::CreateProject => {}
                Page::EditProject => {}
                Page::OpenProject => {}
            }
        }

        match self.page {
            Page::ListProjects => {}
            Page::CreateProject => {
                if self
                    .pages
                    .create_project
                    .key_event_handler(app, key_event, event_state)
                {
                    self.page = Page::ListProjects;
                }
            }
            Page::EditProject => {}
            Page::OpenProject => {}
        };
    }
}

pub struct ProjectsState {
    pub screen_pane: ScreenPane,
}

impl RenderPage<ProjectsState> for Projects {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect, state: ProjectsState) {
        let colors = &app.config.colors;

        match self.page {
            Page::ListProjects => {
                let content = Paragraph::new(Text::from(vec![Line::from(vec![
                    Span::from("You have no projects. Press "),
                    Span::styled("n", Style::new().bold().fg(colors.keybind_key)),
                    Span::from(" to create a new project."),
                ])]))
                .block(
                    Block::new()
                        .padding(Padding::horizontal(1))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::new().fg(if state.screen_pane == ScreenPane::Main {
                            colors.primary
                        } else {
                            colors.border
                        })),
                );
                frame.render_widget(content, area)
            }
            Page::CreateProject => self.pages.create_project.render(app, frame, area, state),
            Page::EditProject => {
                let content = Paragraph::new("Edit Project...");
                frame.render_widget(content, area);
            }
            Page::OpenProject => {
                let content = Paragraph::new("Project...");
                frame.render_widget(content, area);
            }
        }
    }
}
