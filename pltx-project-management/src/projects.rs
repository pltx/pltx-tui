use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{
    state::{Mode, State},
    App,
};
use pltx_tracing::trace_panic;
use pltx_utils::{Init, InitData, KeyEventHandler, RenderPage};
use ratatui::{layout::Rect, Frame};

use super::{
    list_projects::ListProjects, open_project::OpenProject, project_editor::ProjectEditor,
};
use crate::ScreenPane;

#[derive(PartialEq)]
enum Page {
    ListProjects,
    NewProject,
    EditProject,
    OpenProject,
}

struct Pages {
    list_projects: ListProjects,
    new_project: ProjectEditor,
    edit_project: ProjectEditor,
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
                new_project: ProjectEditor::init(app).set_new(),
                edit_project: ProjectEditor::init(app),
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
        if app.state.mode == Mode::Navigation && self.page == Page::ListProjects {
            match key_event.code {
                KeyCode::Char('n') => self.page = Page::NewProject,
                KeyCode::Char('e') => {
                    if let Some(selected_id) = self.pages.list_projects.selected_id {
                        self.pages
                            .edit_project
                            .set_project(app, selected_id)
                            .unwrap_or_else(|e| panic!("{e}"));
                        self.page = Page::EditProject;
                    }
                }
                KeyCode::Enter | KeyCode::Char('l') => {
                    if let Some(selected_id) = self.pages.list_projects.selected_id {
                        self.pages.open_project.reset(app);
                        self.pages.open_project.set_project_id(selected_id);
                        self.pages
                            .open_project
                            .db_get_project(app)
                            .unwrap_or_else(|e| trace_panic!("{e}"));
                        self.page = Page::OpenProject;
                    }
                }
                _ => {}
            }
        }

        let result: bool = match self.page {
            Page::ListProjects => {
                self.pages
                    .list_projects
                    .key_event_handler(app, key_event, event_state)
            }
            Page::NewProject => {
                self.pages
                    .new_project
                    .key_event_handler(app, key_event, event_state)
            }
            Page::EditProject => {
                self.pages
                    .edit_project
                    .key_event_handler(app, key_event, event_state)
            }
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
            Page::NewProject => self.pages.new_project.render(app, frame, area, state),
            Page::EditProject => self.pages.edit_project.render(app, frame, area, state),
            Page::OpenProject => self.pages.open_project.render(app, frame, area, state),
        }
    }
}
