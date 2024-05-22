use pltx_app::{
    state::{Display, GlobalPopup, ModuleState, Pane},
    App,
};
use pltx_config::ColorsConfig;
use pltx_dashboard::Dashboard;
use pltx_project_management::ProjectManagement;
use pltx_utils::{Module, Popup};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::*, Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{command_handler::CommandHandler, popups};

/// States for each module.
pub struct InterfaceModule {
    pub dashboard: Dashboard,
    pub project_management: ProjectManagement,
}

/// States for each popup.
pub struct PopupState {
    pub help: popups::Help,
}

pub struct Interface {
    pub modules: InterfaceModule,
    /// Global popups. Module popups are located within the modules own
    /// directories.
    pub popups: PopupState,
}

impl Interface {
    pub fn init(app: &mut App) -> Interface {
        Interface {
            modules: InterfaceModule {
                dashboard: Dashboard::init(app),
                project_management: ProjectManagement::init(app),
            },
            popups: PopupState {
                help: popups::Help::init(app),
            },
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        app: &mut App,
        command_handler: &mut CommandHandler,
    ) {
        let colors = &app.config.colors.clone();

        // Root layout
        let [title_bar_layout, main_layout, status_bar_layout] = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .areas(frame.size());

        frame.render_widget(self.title_bar(colors), title_bar_layout);

        // Navigation and editor layout
        let [navigation_layout, module_layout] = Layout::default()
            .constraints([Constraint::Length(30), Constraint::Min(1)])
            .direction(Direction::Horizontal)
            .areas(main_layout);

        frame.render_widget(
            self.navigation_pane(app, navigation_layout),
            navigation_layout,
        );

        self.status_bar(app, frame, status_bar_layout);

        let module_pane = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(
                if app.display.is_default() && app.pane == Pane::Module {
                    let primary_border = match &app.module {
                        // `true` to have the border primary color by default
                        // Otherwise the color will be the border color
                        ModuleState::Dashboard => false,
                        ModuleState::ProjectManagement => false,
                        _ => true,
                    };
                    if primary_border {
                        colors.primary
                    } else {
                        colors.border
                    }
                } else {
                    colors.border
                },
            ))
            .padding(Padding::horizontal(1))
            .bg(colors.bg);
        frame.render_widget(module_pane, module_layout);

        let [module_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Min(1)])
            .areas(module_layout);
        match app.module {
            ModuleState::Dashboard => self.modules.dashboard.render(app, frame, module_layout),
            ModuleState::ProjectManagement => {
                self.modules
                    .project_management
                    .render(app, frame, module_layout)
            }
            ModuleState::None => {}
        };

        if app.display.is_popup() {
            match app.popup {
                GlobalPopup::Help => self.popups.help.render(app, frame, frame.size()),
                GlobalPopup::None => {}
            }
        }

        if app.display.is_command() {
            command_handler.render(app, frame, frame.size());
        }
    }

    fn title_bar(&self, colors: &ColorsConfig) -> Paragraph {
        let title_bar_content = vec![Line::from(
            vec![Span::from(" Privacy Life Tracker ").bold()],
        )];
        Paragraph::new(title_bar_content)
            .alignment(Alignment::Center)
            .style(Style::new().fg(colors.title_bar_fg).bg(colors.title_bar_bg))
    }

    fn navigation_pane(&self, app: &App, rect: Rect) -> Paragraph {
        let module_list = app.module_list();
        let colors = &app.config.colors;

        let navigation_options = module_list
            .iter()
            .map(|s| {
                if s.0 == app.module {
                    Line::from(format!(
                        " {} {}",
                        s.1,
                        " ".repeat(rect.as_size().width as usize - s.1.len() - 4)
                    ))
                    .style(
                        Style::new()
                            .fg(colors.active_fg)
                            .bg(colors.active_bg)
                            .bold(),
                    )
                } else {
                    Line::from(format!(" {} ", s.1)).style(Style::new().fg(colors.secondary))
                }
            })
            .collect::<Vec<Line>>();

        Paragraph::new(navigation_options.clone()).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(match (&app.display, &app.pane) {
                    (Display::Default(_), Pane::Navigation) => colors.primary,
                    _ => colors.border,
                }))
                // .padding(Padding::symmetric(1, 0))
                .bg(colors.bg),
        )
    }

    fn status_bar(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;
        let mode = app.current_mode_data();

        let [left_layout, center_layout, right_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .areas(area);

        let mut mode_fg = mode.fg;
        let mut mode_bg = mode.bg;
        let mut status_bar_fg = colors.status_bar_fg;
        let mut status_bar_bg = colors.status_bar_bg;
        if app.is_delete_mode() {
            mode_fg = colors.status_bar_fg;
            mode_bg = colors.status_bar_bg;
            status_bar_fg = mode.fg;
            status_bar_bg = mode.bg;
        }
        let left_text = vec![Line::from(vec![
            Span::from(format!(" {} ", mode.text.to_uppercase()))
                .bold()
                .fg(mode_fg)
                .bg(mode_bg),
            Span::from("").fg(mode_bg),
            if app.is_delete_mode() {
                Span::from(" Confirm Deletion (y/n)").bold()
            } else {
                Span::from("")
            },
        ])];
        let left_content = Paragraph::new(left_text)
            .alignment(Alignment::Left)
            .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(left_content, left_layout);

        let center_text = vec![Line::from(vec![Span::from("")])];
        let center_content = Paragraph::new(center_text)
            .alignment(Alignment::Center)
            .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(center_content, center_layout);

        let right_text = vec![Line::from(vec![Span::from("Press ? for help ")])];
        let right_content = Paragraph::new(if app.is_delete_mode() {
            vec![Line::from("")]
        } else {
            right_text
        })
        .alignment(Alignment::Right)
        .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(right_content, right_layout);
    }
}
