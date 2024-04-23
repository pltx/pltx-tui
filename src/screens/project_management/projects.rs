use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use super::{
    list_projects::ListProjects, new_project::NewProject, open_project::OpenProject,
    screen::ScreenPane,
};
use crate::{
    state::{Mode, State},
    utils::{Init, InitData, KeyEventHandler, KeyEventHandlerReturn, RenderPage},
    App,
};

#[derive(PartialEq)]
enum Page {
    ListProjects,
    NewProject,
    EditProject,
    OpenProject,
}

struct Pages {
    list_projects: ListProjects,
    create_project: NewProject,
    open_project: OpenProject,
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
                list_projects: ListProjects::init(app),
                create_project: NewProject::init(app),
                open_project: OpenProject::init(app),
            },
        }
    }
}

impl InitData for Projects {
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()> {
        self.pages.list_projects.init_data(app)?;
        Ok(())
    }
}

impl KeyEventHandler for Projects {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) {
        if app.state.mode == Mode::Navigation {
            match self.page {
                Page::ListProjects => {
                    self.pages
                        .list_projects
                        .key_event_handler(app, key_event, event_state);
                    match key_event.code {
                        // Create a new project
                        KeyCode::Char('n') => {
                            self.page = Page::NewProject;
                        }
                        // Edit the selected project
                        KeyCode::Char('e') => self.page = Page::EditProject,
                        // Delete the selected project
                        KeyCode::Char('d') => {}
                        // Open the selected project
                        KeyCode::Enter => {
                            self.pages
                                .open_project
                                .set_project_id(self.pages.list_projects.selected_id);
                            self.pages
                                .open_project
                                .db_get_project(app)
                                .unwrap_or_else(|e| panic!("{e}"));
                            self.page = Page::OpenProject;
                        }
                        _ => {}
                    }
                }
                Page::NewProject => {}
                Page::EditProject => {}
                Page::OpenProject => {}
            }
        }

        let result: bool = match self.page {
            Page::ListProjects => false,
            Page::NewProject => {
                self.pages
                    .create_project
                    .key_event_handler(app, key_event, event_state)
            }
            Page::EditProject => false,
            Page::OpenProject => {
                self.pages
                    .open_project
                    .key_event_handler(app, key_event, event_state)
            }
        };
        if result {
            self.page = Page::ListProjects;
            self.pages
                .list_projects
                .db_get_projects(app)
                .unwrap_or_else(|e| panic!("{e}"));
        }
    }
}

pub struct ProjectsState {
    pub screen_pane: ScreenPane,
}

impl RenderPage<ProjectsState> for Projects {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect, state: ProjectsState) {
        match self.page {
            Page::ListProjects => self.pages.list_projects.render(app, frame, area, state),
            Page::NewProject => self.pages.create_project.render(app, frame, area, state),
            Page::EditProject => {
                let content = Paragraph::new("Edit Project...");
                frame.render_widget(content, area);
            }
            Page::OpenProject => self.pages.open_project.render(app, frame, area, state),
        }
    }
}
