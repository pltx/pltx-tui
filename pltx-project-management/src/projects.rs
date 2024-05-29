use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, Popup, Screen};
use ratatui::{layout::Rect, Frame};

use crate::{
    list_projects::ListProjects, open_project::OpenProject, popups::project_editor::ProjectEditor,
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
    new_project: ProjectEditor,
    edit_project: ProjectEditor,
    open_project: OpenProject,
}

pub struct Projects {
    page: Page,
    pages: Pages,
}

impl Screen<Result<()>> for Projects {
    fn init(app: &App) -> Result<Projects> {
        Ok(Projects {
            page: Page::ListProjects,
            pages: Pages {
                list_projects: ListProjects::init(app)?,
                new_project: ProjectEditor::init(),
                edit_project: ProjectEditor::init(),
                open_project: OpenProject::init(app)?,
            },
        })
    }

    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        if app.is_normal_mode() && self.page == Page::ListProjects {
            match key_event.code {
                KeyCode::Char('n') => {
                    self.page = Page::NewProject;
                    app.popup_display();
                }
                KeyCode::Char('e') => {
                    if let Some(id) = self.pages.list_projects.get_id() {
                        self.pages.edit_project.set_project(&app.db, id)?;
                        self.page = Page::EditProject;
                        app.popup_display();
                    }
                }
                KeyCode::Enter | KeyCode::Char('l') => {
                    if let Some(id) = self.pages.list_projects.get_id() {
                        self.pages.open_project.reset(app);
                        self.pages.open_project.set_project_id(id);
                        self.pages.open_project.db_get_project(app)?;
                        self.page = Page::OpenProject;
                    }
                }
                _ => {}
            }
        }

        let result: bool = match self.page {
            Page::ListProjects => self.pages.list_projects.key_event_handler(app, key_event)?,
            Page::NewProject => self.pages.new_project.key_event_handler(app, key_event)?,
            Page::EditProject => self.pages.edit_project.key_event_handler(app, key_event)?,
            Page::OpenProject => self.pages.open_project.key_event_handler(app, key_event)?,
        };

        if result {
            self.page = Page::ListProjects;
            self.pages.list_projects.db_get_projects(app)?;
        }

        Ok(())
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        match self.page {
            Page::ListProjects => self.pages.list_projects.render(app, frame, area),
            Page::NewProject => self.pages.new_project.render(app, frame, area),
            Page::EditProject => self.pages.edit_project.render(app, frame, area),
            Page::OpenProject => self.pages.open_project.render(app, frame, area),
        }
    }
}
