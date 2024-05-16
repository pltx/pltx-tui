use pltx_app::{
    state::{GlobalPopup, Mode, Pane, Screen},
    App,
};
use pltx_config::ColorsConfig;
use pltx_dashboard::Dashboard;
use pltx_project_management::ProjectManagement;
use pltx_utils::{Init, InitData, RenderPopup, RenderScreen, RenderScrollPopup};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::*, Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{command_handler::CommandHandler, popups};

/// States for each screen
pub struct ScreenState {
    pub dashboard: Dashboard,
    pub project_management: ProjectManagement,
}

/// States for each popup
pub struct PopupState {
    pub help: popups::Help,
}

pub struct Interface {
    pub screens: ScreenState,
    /// These are only global popups. Screen popups should be contained within
    /// the screen's own directory.
    pub popups: PopupState,
}

impl Interface {
    pub fn init(app: &mut App) -> Interface {
        Interface {
            screens: ScreenState {
                dashboard: Dashboard::init(app),
                project_management: ProjectManagement::init(app),
            },
            popups: PopupState {
                help: popups::Help::init(app),
            },
        }
    }

    /// Call the `init_data()` functions for popups and screens. The
    /// `init_data()` functions ensure that the tables required exist, and
    /// if not, are created.
    pub fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()> {
        self.screens.project_management.init_data(app)?;
        Ok(())
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
        let [navigation_layout, screen_layout] = Layout::default()
            .constraints([Constraint::Length(30), Constraint::Min(1)])
            .direction(Direction::Horizontal)
            .areas(main_layout);

        frame.render_widget(
            self.navigation_pane(app, navigation_layout),
            navigation_layout,
        );

        self.status_bar(app, frame, status_bar_layout);

        // Screen content
        let screen_pane = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(
                if app.state.mode == Mode::Navigation && app.state.pane == Pane::Screen {
                    let primary_border = match &app.state.screen {
                        // `true` to have the border primary color by default
                        // Otherwise the color will be the border color
                        Screen::Dashboard => false,
                        Screen::ProjectManagement => false,
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
        frame.render_widget(screen_pane, screen_layout);

        let [screen_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Min(1)])
            .areas(screen_layout);
        match app.state.screen {
            Screen::Dashboard => self.screens.dashboard.render(app, frame, screen_layout),
            Screen::ProjectManagement => {
                self.screens
                    .project_management
                    .render(app, frame, screen_layout)
            }
            Screen::None => {}
        };

        if app.state.mode == Mode::Popup || app.state.mode == Mode::PopupInsert {
            match app.state.popup {
                GlobalPopup::Help => self.popups.help.render(frame, app),
                GlobalPopup::None => {}
            }
        }

        if app.state.mode == Mode::Command || app.state.mode == Mode::CommandInsert {
            command_handler.render(frame, app);
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
        let screen_list = app.get_screen_list();
        let colors = &app.config.colors;

        let navigation_options = screen_list
            .iter()
            .map(|s| {
                if s.0 == app.state.screen {
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
                .border_style(Style::new().fg(match (&app.state.mode, &app.state.pane) {
                    (Mode::Navigation, Pane::Navigation) => colors.primary,
                    _ => colors.border,
                }))
                // .padding(Padding::symmetric(1, 0))
                .bg(colors.bg),
        )
    }

    fn status_bar(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;
        let mode = app.get_mode_colors();

        let [left_layout, center_layout, right_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .areas(area);

        let mut mode_fg = mode.1;
        let mut mode_bg = mode.2;
        let mut status_bar_fg = colors.status_bar_fg;
        let mut status_bar_bg = colors.status_bar_bg;
        if app.state.mode == Mode::Delete {
            mode_fg = colors.status_bar_fg;
            mode_bg = colors.status_bar_bg;
            status_bar_fg = mode.1;
            status_bar_bg = mode.2;
        }
        let left_text = vec![Line::from(vec![
            Span::from(format!(" {} ", mode.0.to_uppercase()))
                .bold()
                .fg(mode_fg)
                .bg(mode_bg),
            Span::from("").fg(mode_bg),
            match app.state.mode {
                Mode::Delete => Span::from(" Confirm Deletion (y/n)").bold(),
                _ => Span::from(""),
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
        let right_content = Paragraph::new(match app.state.mode {
            Mode::Delete => vec![Line::from("")],
            _ => right_text,
        })
        .alignment(Alignment::Right)
        .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(right_content, right_layout);
    }
}
