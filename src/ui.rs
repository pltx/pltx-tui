use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::*, Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{
    config::ColorsConfig,
    popups, screens,
    state::{Mode, Pane, Screen},
    utils::{PaneTitleBottom, RenderPopup, RenderScreen},
    App, Popup,
};

/// States for each screen
pub struct ScreenState {
    pub dashboard: screens::Dashboard,
    pub project_management: screens::ProjectManagement,
    pub sleep: screens::Sleep,
    pub settings: screens::Settings,
}

/// States for each popup
pub struct PopupState {
    pub help: popups::Help,
}

pub struct Interface {
    pub screens: ScreenState,
    pub popups: PopupState,
}

impl Interface {
    pub fn init() -> Interface {
        Interface {
            screens: ScreenState {
                dashboard: screens::Dashboard::init(),
                project_management: screens::ProjectManagement::init(),
                sleep: screens::Sleep::init(),
                settings: screens::Settings::init(),
            },
            popups: PopupState {
                help: popups::Help::init(),
            },
        }
    }

    pub fn render(&mut self, frame: &mut Frame, app: &mut App) {
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

        frame.render_widget(self.navigation_pane(app), navigation_layout);
        frame.render_widget(self.status_bar(app), status_bar_layout);

        // Screen content
        let mut screen_pane = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(match (&app.state.mode, &app.state.pane) {
                (Mode::Navigation, Pane::Screen) => colors.primary,
                _ => colors.border,
            }))
            .padding(Padding::horizontal(1))
            .bg(colors.bg);
        // Custom title for a screen
        if app.state.screen == Screen::ProjectManagement {
            screen_pane =
                screen_pane.title_bottom(self.screens.project_management.pane_title_bottom(app))
        }
        frame.render_widget(screen_pane, screen_layout);

        let [screen_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Min(1)])
            .areas(screen_layout);
        match app.state.screen {
            Screen::Dashboard => self.screens.dashboard.render(frame, app, screen_layout),
            Screen::ProjectManagement => {
                self.screens
                    .project_management
                    .render(frame, app, screen_layout)
            }
            Screen::Sleep => self.screens.sleep.render(frame, app, screen_layout),
            Screen::Settings => self.screens.settings.render(frame, app, screen_layout),
        };

        // Popup
        if app.state.mode == Mode::Popup {
            match app.state.popup {
                Popup::Help => self.popups.help.render(frame, app),
                Popup::None => {}
            }
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

    fn navigation_pane(&self, app: &App) -> Paragraph {
        let screen_list = app.get_screen_list();
        let colors = &app.config.colors;

        let navigation_options = screen_list
            .iter()
            .map(|s| {
                let mut style = Style::new();
                if s.0 == app.state.screen {
                    style = style.fg(colors.active).bold()
                } else {
                    style = style.fg(colors.secondary)
                };
                Line::from(s.1).style(style)
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
                .padding(Padding::symmetric(1, 0))
                .bg(colors.bg),
        )
    }

    fn status_bar(&self, app: &App) -> Paragraph {
        let colors = &app.config.colors;
        let mode = app.get_mode();
        let status_bar_content = vec![Line::from(vec![
            Span::from(format!(" {} ", mode.0.to_uppercase()))
                .bold()
                .fg(mode.1)
                .bg(mode.2),
            Span::from("").fg(mode.2),
        ])];
        Paragraph::new(status_bar_content)
            .alignment(Alignment::Left)
            .style(
                Style::new()
                    .fg(colors.status_bar_fg)
                    .bg(colors.status_bar_bg),
            )
    }
}
