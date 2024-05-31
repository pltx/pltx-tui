use std::str::FromStr;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::View, App, DefaultWidget, KeyEventHandler, Popup};
use pltx_database::Database;
use pltx_utils::DateTime;
use pltx_widgets::{PopupSize, PopupWidget, Scrollable, TextInput};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const PROJECT_TITLE_MAX_LENGTH: usize = 50;
const PROJECT_DESCRIPTION_MAX_LENGTH: usize = 160;
const LABEL_TITLE_MAX_LENTH: usize = 15;
const LABEL_COLOR_REQUIRED_LENTH: usize = 7;

#[derive(PartialEq)]
pub enum LabelView {
    Selection,
    Input,
}

struct Label<T = Option<i32>> {
    id: T,
    title: String,
    color: String,
}

struct LabelInputs {
    title: TextInput,
    color: TextInput,
}

#[derive(PartialEq)]
enum FocusedLabelInput {
    Title,
    Color,
}

pub struct LabelEditor {
    pub view: LabelView,
    labels: Vec<Label>,
    selection: Scrollable,
    inputs: LabelInputs,
    focused_input: FocusedLabelInput,
    has_id: bool,
}

impl LabelEditor {
    pub fn init() -> Self {
        Self {
            view: LabelView::Selection,
            labels: vec![],
            selection: Scrollable::default(),
            inputs: LabelInputs {
                title: TextInput::new("Label Title")
                    .view(View::Popup)
                    .max(LABEL_TITLE_MAX_LENTH)
                    .prompt(),
                color: TextInput::new("Label Color")
                    .view(View::Popup)
                    .max(LABEL_COLOR_REQUIRED_LENTH)
                    .prompt(),
            },
            focused_input: FocusedLabelInput::Title,
            has_id: false,
        }
    }
}

impl KeyEventHandler for LabelEditor {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        if self.view == LabelView::Selection {
            self.selection.key_event_handler(app, key_event);
        } else if self.view == LabelView::Input {
            match self.focused_input {
                FocusedLabelInput::Title => self.inputs.title.key_event_handler(app, key_event),
                FocusedLabelInput::Color => self.inputs.color.key_event_handler(app, key_event),
            }
        }

        match key_event.code {
            KeyCode::Char('n') => {
                if self.view == LabelView::Selection {
                    if app.mode.is_delete() {
                        app.mode.normal();
                    } else {
                        self.view = LabelView::Input;
                        app.mode.insert();
                    }
                }
            }
            KeyCode::Char('e') => {
                if self.view == LabelView::Selection && !self.labels.is_empty() {
                    self.has_id = true;
                    let label = &self.labels[self.selection.focused];
                    self.inputs.title.input(label.title.to_owned());
                    self.inputs.color.input(label.color.to_owned());
                    self.view = LabelView::Input;
                    app.mode.insert();
                }
            }
            KeyCode::Char('d') => {
                if self.view == LabelView::Selection && !self.labels.is_empty() {
                    app.mode.delete();
                }
            }
            KeyCode::Char('.') => {
                if self.view == LabelView::Selection && !self.labels.is_empty() {
                    let label = &self.labels[self.selection.focused];
                    self.labels.push(Label {
                        id: None,
                        title: label.title.to_owned(),
                        color: label.color.to_owned(),
                    });
                }
            }
            KeyCode::Char('y') => {
                if self.view == LabelView::Selection && app.mode.is_delete() {
                    self.labels.remove(self.selection.focused);
                    app.mode.normal();
                }
            }
            KeyCode::Char('[') => {
                if app.mode.is_normal() && self.view == LabelView::Input {
                    self.reset()
                }
            }
            KeyCode::Tab => {
                if self.view == LabelView::Input && self.focused_input == FocusedLabelInput::Title {
                    self.focused_input = FocusedLabelInput::Color;
                }
            }
            KeyCode::Enter => {
                if self.view == LabelView::Input {
                    if self.focused_input == FocusedLabelInput::Title {
                        self.focused_input = FocusedLabelInput::Color;
                    } else {
                        if self.has_id {
                            self.labels[self.selection.focused] = Label {
                                id: self.labels[self.selection.focused].id,
                                title: self.inputs.title.input_string(),
                                color: self.inputs.color.input_string(),
                            };
                        } else {
                            self.labels.push(Label {
                                id: None,
                                title: self.inputs.title.input_string(),
                                color: self.inputs.color.input_string(),
                            });
                        };
                        self.reset();
                        app.mode.normal();
                    }
                }
            }
            _ => {}
        }
    }
}

impl DefaultWidget for LabelEditor {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, _: bool) {
        let colors = &app.config.colors;

        if self.view == LabelView::Selection {
            if self.labels.is_empty() {
                let paragraph =
                    Paragraph::new("Press n to create a new label.").fg(colors.secondary_fg);
                frame.render_widget(paragraph, area);
            }

            let table = self
                .labels
                .iter()
                .enumerate()
                .map(|(i, label)| {
                    Paragraph::new(label.title.to_owned())
                        .fg(Color::from_str(&label.color).unwrap_or(colors.fg))
                        .bg(if self.selection.focused == i {
                            colors.input_focus_bg
                        } else {
                            colors.popup_bg
                        })
                })
                .collect::<Vec<Paragraph>>();

            self.selection.render(frame, area, table);
        } else if self.view == LabelView::Input {
            let [preview_layout, title_layout, color_layout] = Layout::default()
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .areas(area);

            let preview = Paragraph::new(Line::from(vec![
                Span::from("Preview: ").fg(colors.secondary_fg),
                Span::from(self.inputs.title.input_string()).fg(
                    if self.inputs.color.input_string().chars().count()
                        == LABEL_COLOR_REQUIRED_LENTH
                    {
                        Color::from_str(&self.inputs.color.input_string()).unwrap_or(colors.fg)
                    } else {
                        colors.fg
                    },
                ),
            ]));

            frame.render_widget(preview, preview_layout);

            self.inputs.title.render(
                frame,
                app,
                title_layout,
                self.focused_input == FocusedLabelInput::Title,
            );
            self.inputs.color.render(
                frame,
                app,
                color_layout,
                self.focused_input == FocusedLabelInput::Color,
            );
        }
    }
}

impl LabelEditor {
    pub fn reset(&mut self) {
        self.view = LabelView::Selection;
        self.focused_input = FocusedLabelInput::Title;
        self.inputs.title.reset();
        self.inputs.color.reset();
    }
}

#[derive(PartialEq)]
enum EditorView {
    Selection,
    Input,
}

struct Inputs {
    title: TextInput,
    description: TextInput,
    label_editor: LabelEditor,
}

struct ProjectData {
    id: i32,
    title: String,
    description: Option<String>,
    labels: Vec<Label<i32>>,
}

#[derive(Clone, PartialEq)]
enum Input {
    Title,
    Description,
    Labels,
    None,
}

pub struct ProjectEditor {
    /// If original data is Some(), then the project data will be updated in the
    /// database, otherwise, a new entry will be created.
    original_data: Option<ProjectData>,
    inputs: Vec<(Input, String)>,
    input_widgets: Inputs,
    view: EditorView,
    selection: Scrollable,
    current_input: Input,
    default_size: PopupSize,
    size: PopupSize,
}

impl Popup<Result<bool>> for ProjectEditor {
    fn init() -> Self {
        let size = PopupSize::default().width(70).height(7);

        Self {
            original_data: None,
            inputs: vec![
                (Input::Title, String::from("Title")),
                (Input::Description, String::from("Description")),
                (Input::Labels, String::from("Labels")),
            ],
            input_widgets: Inputs {
                title: TextInput::new("Title")
                    .view(View::Popup)
                    .max(PROJECT_TITLE_MAX_LENGTH)
                    .prompt(),
                description: TextInput::new("Description")
                    .view(View::Popup)
                    .max(PROJECT_DESCRIPTION_MAX_LENGTH)
                    .prompt()
                    .prompt_lines(3),
                label_editor: LabelEditor::init(),
            },
            view: EditorView::Selection,
            selection: Scrollable::default(),
            current_input: Input::None,
            default_size: size,
            size,
        }
    }

    // Returns whether the project data in the database was modified. And the page
    // should be set to list projects.
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> Result<bool> {
        self.selection.key_event_handler(app, key_event);

        if app.mode.is_normal() && key_event.code == KeyCode::Char('q') {
            self.reset();
            app.view.default();
            return Ok(true);
        }

        if self.view == EditorView::Selection {
            match key_event.code {
                KeyCode::Enter | KeyCode::Char('l') => {
                    self.view = EditorView::Input;
                    self.current_input = self.inputs[self.selection.focused].0.clone();
                    let height = match self.current_input {
                        Input::Title => 6,
                        Input::Description => 9,
                        Input::Labels => 10,
                        Input::None => 0,
                    };
                    if self.current_input != Input::None {
                        self.size = PopupSize::default()
                            .width(self.default_size.width)
                            .height(height);
                    }
                    if [Input::Title, Input::Description].contains(&self.current_input) {
                        app.mode.insert();
                    }
                }
                KeyCode::Char('s') => {
                    if self.view == EditorView::Selection {
                        if self.original_data.is_some() {
                            self.db_edit_project(&app.db)?;
                        } else {
                            self.db_new_project(&app.db)?;
                        }
                        self.reset()
                    }

                    return Ok(true);
                }
                _ => {}
            }
        } else if self.view == EditorView::Input {
            if self.input_widgets.label_editor.view != LabelView::Input {
                match key_event.code {
                    KeyCode::Char('[') => {
                        if app.mode.is_normal() {
                            self.view = EditorView::Selection;
                            self.size = self.default_size;
                        }
                    }
                    KeyCode::Enter => {
                        {
                            self.view = EditorView::Selection;
                            app.mode.normal();
                            self.size = self.default_size;
                        }
                    }
                    _ => {}
                }
            }

            match self.current_input {
                Input::Title => self.input_widgets.title.key_event_handler(app, key_event),
                Input::Description => self
                    .input_widgets
                    .description
                    .key_event_handler(app, key_event),
                Input::Labels => self
                    .input_widgets
                    .label_editor
                    .key_event_handler(app, key_event),
                Input::None => {}
            };
        }

        Ok(false)
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let popup = PopupWidget::new(app, area)
            .title_top(if self.original_data.is_some() {
                "Edit Project"
            } else {
                "New Project"
            })
            .size(self.size)
            .render(frame);

        let area = popup.sub_area;

        if self.view == EditorView::Selection {
            let table = self
                .inputs
                .iter()
                .enumerate()
                .map(|(i, input)| {
                    if self.selection.focused == i {
                        Paragraph::new(format!(" {} ", input.1))
                            .style(Style::new().bg(colors.input_focus_bg))
                    } else {
                        Paragraph::new(format!(" {} ", input.1))
                            .style(Style::new().fg(colors.secondary_fg))
                    }
                })
                .collect::<Vec<Paragraph>>();

            self.selection.render(frame, area, table);
        } else {
            let [input_layout] = Layout::default()
                .margin(1)
                .constraints([Constraint::Fill(1)])
                .areas(area);

            match self.current_input {
                Input::Title => self
                    .input_widgets
                    .title
                    .render(frame, app, input_layout, true),
                Input::Description => {
                    self.input_widgets
                        .description
                        .render(frame, app, input_layout, true)
                }
                Input::Labels => self
                    .input_widgets
                    .label_editor
                    .render(frame, app, area, true),
                Input::None => {}
            }
        }
    }
}

impl ProjectEditor {
    pub fn set_project(&mut self, db: &Database, project_id: i32) -> Result<()> {
        let conn = db.conn();
        let project_query = "SELECT id, title, description FROM project WHERE id = ?1";
        let mut project_stmt = conn.prepare(project_query)?;
        let mut project = project_stmt.query_row([project_id], |r| {
            Ok(ProjectData {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                labels: vec![],
            })
        })?;

        project.labels = self.db_get_labels(db, project_id)?;

        self.original_data = Some(project);

        if let Some(original_data) = &self.original_data {
            self.input_widgets.title.input(original_data.title.clone());
            self.input_widgets.description.input(
                if let Some(description) = &original_data.description {
                    description.clone()
                } else {
                    String::from("")
                },
            );
        }

        Ok(())
    }

    fn db_get_labels(&mut self, db: &Database, project_id: i32) -> Result<Vec<Label<i32>>> {
        let conn = db.conn();
        let query =
            "SELECT id, title, color FROM project_label WHERE project_id = ?1 ORDER BY position";
        let mut stmt = conn.prepare(query)?;
        let labels_iter = stmt.query_map([project_id], |r| {
            Ok(Label::<i32> {
                id: r.get(0)?,
                title: r.get(1)?,
                color: r.get(2)?,
            })
        })?;

        let mut labels = vec![];
        for l in labels_iter {
            let label = l?;

            let label_position = self
                .input_widgets
                .label_editor
                .labels
                .iter()
                .position(|p| p.id.is_some_and(|id| id == label.id));

            if let Some(pos) = label_position {
                self.input_widgets.label_editor.labels[pos]
                    .title
                    .clone_from(&label.title);
                self.input_widgets.label_editor.labels[pos]
                    .color
                    .clone_from(&label.color);
            } else {
                self.input_widgets.label_editor.labels.push(Label {
                    id: Some(label.id),
                    title: label.title.to_owned(),
                    color: label.color.to_owned(),
                })
            }

            labels.push(label);
        }

        Ok(labels)
    }

    fn db_new_project(&self, db: &Database) -> Result<()> {
        let description = if self
            .input_widgets
            .description
            .input_string()
            .chars()
            .count()
            == 0
        {
            None
        } else {
            Some(self.input_widgets.description.input_string())
        };

        let highest_position = db.get_highest_position("project")?;
        db.conn().execute(
            "INSERT INTO project (title, description, position, created_at, updated_at) VALUES \
             (?1, ?2, ?3, ?4, ?5)",
            (
                self.input_widgets.title.input_string(),
                description,
                highest_position + 1,
                DateTime::now(),
                DateTime::now(),
            ),
        )?;

        let new_project_id = db.last_row_id("project")?;

        self.db_new_labels(db, new_project_id)?;

        Ok(())
    }

    fn db_new_labels(&self, db: &Database, project_id: i32) -> Result<()> {
        for (i, label) in self.input_widgets.label_editor.labels.iter().enumerate() {
            let query = "INSERT INTO project_label (project_id, title, color, position, \
                         created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
            db.conn().execute(
                query,
                (
                    project_id,
                    label.title.to_owned(),
                    label.color.to_owned(),
                    i,
                    DateTime::now(),
                    DateTime::now(),
                ),
            )?;
        }

        Ok(())
    }

    fn db_edit_project(&self, db: &Database) -> Result<()> {
        if let Some(data) = &self.original_data {
            let conn = db.conn();
            let query =
                "UPDATE project SET title = ?1, description = ?2, updated_at = ?3 WHERE id = ?4";
            let mut stmt = conn.prepare(query)?;
            stmt.execute((
                self.input_widgets.title.input_string(),
                self.input_widgets.description.input_string(),
                DateTime::now(),
                data.id,
            ))?;
            self.db_edit_labels(db, data.id)?;
        } else {
            panic!("project data was not set")
        }

        Ok(())
    }

    fn db_edit_labels(&self, db: &Database, project_id: i32) -> Result<()> {
        let conn = db.conn();
        for (i, label) in self.input_widgets.label_editor.labels.iter().enumerate() {
            if let Some(label_id) = label.id {
                let query = "UPDATE project_label SET title = ?1, color = ?2, updated_at = ?3 \
                             WHERE project_id = ?4 and id = ?5";
                let mut stmt = conn.prepare(query)?;
                stmt.execute((
                    label.title.to_owned(),
                    label.color.to_owned(),
                    DateTime::now(),
                    project_id,
                    label_id,
                ))?;
            } else {
                let query = "INSERT INTO project_label (project_id, title, color, position, \
                             created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
                conn.execute(
                    query,
                    (
                        project_id,
                        label.title.to_owned(),
                        label.color.to_owned(),
                        i,
                        DateTime::now(),
                        DateTime::now(),
                    ),
                )?;
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.original_data = None;
        self.view = EditorView::Selection;
        self.current_input = Input::None;
        self.size = self.default_size;
        self.input_widgets.title.reset();
        self.input_widgets.description.reset();
        self.input_widgets.label_editor.reset();
    }
}
