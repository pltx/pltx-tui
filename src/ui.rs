use std::{str::FromStr, time::Instant};

use color_eyre::Result;
use pltx_app::{
    state::{AppModule, AppPopup},
    App, DebugPosition, Module,
};
use pltx_config::ColorsConfig;
use pltx_home::Home;
use pltx_project_management::ProjectManagement;
use pltx_utils::DateTime;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::command_handler::CommandHandler;

/// States for each module.
pub struct InterfaceModule {
    pub home: Home,
    pub project_management: ProjectManagement,
}

/// States for each popup.
pub struct PopupState {}

pub struct Interface {
    pub modules: InterfaceModule,
    /// Global popups. Module popups are located within the modules own
    /// directories.
    // TODO: Remove underscore when popups are added.
    pub _popups: PopupState,
}

impl Interface {
    pub fn init(app: &mut App) -> Result<Self> {
        let start = Instant::now();
        let interface = Self {
            modules: InterfaceModule {
                home: Home::init(app)?,
                project_management: ProjectManagement::init(app)?,
            },
            _popups: PopupState {},
        };
        tracing::info!("initialized interface in {:?}", start.elapsed());
        Ok(interface)
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        app: &mut App,
        command_handler: &mut CommandHandler,
    ) {
        let colors = &app.config.colors.clone();
        let area = if app.debug.enabled && app.debug.show && app.debug.min_preview {
            // Minimum support size.
            let width = 100;
            let height = 30;
            Rect::new(
                frame.size().width / 2 - width / 2,
                frame.size().height / 2 - height / 2,
                width,
                height,
            )
        } else {
            frame.size()
        };

        let [title_bar_layout, module_layout, status_bar_layout] = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .areas(area);

        frame.render_widget(self.title_bar(colors), title_bar_layout);
        frame.render_widget(Block::new().bg(colors.bg).fg(colors.fg), module_layout);

        self.status_bar(app, frame, status_bar_layout);

        match app.module {
            AppModule::Home => self.modules.home.render(app, frame, module_layout),
            AppModule::ProjectManagement => {
                self.modules
                    .project_management
                    .render(app, frame, module_layout)
            }
            AppModule::None => {}
        };

        if app.view.is_popup() {
            match app.popup {
                AppPopup::None => {}
            }
        }

        if app.view.is_command() {
            command_handler.render(app, frame, area);
        }

        if app.debug.enabled && app.debug.show {
            let debug_lines = vec![
                Line::from("~ = rotate position, ! = toggle min preview"),
                Line::from(format!("Version: {}", env!("CARGO_PKG_VERSION"))),
                Line::from(format!("Frame Size: {}x{}", area.width, area.height)),
                Line::from(format!("RUST_BACKTRACE: {}", env!("RUST_BACKTRACE"))),
                Line::from(format!("Min Preview: {}", app.debug.min_preview)),
                Line::from(format!("Profile: {}", app.profile.name)),
                Line::from(format!("Config File: {}", app.profile.config_file)),
                Line::from(format!("DB File: {}", app.profile.db_file)),
                Line::from(format!("Log File: {}", app.profile.log_file)),
                Line::from(format!("View: {}", app.view)),
                Line::from(format!("View iscmd: {}", app.view.is_command())),
            ];

            let area = frame.size();
            let height = debug_lines.len() as u16 + 2;
            let width = 50;
            let x = match app.debug.position {
                DebugPosition::Top | DebugPosition::Bottom => area.width / 2 - width / 2,
                DebugPosition::TopRight | DebugPosition::Right | DebugPosition::BottomRight => {
                    area.width - width
                }
                DebugPosition::BottomLeft | DebugPosition::Left | DebugPosition::TopLeft => 0,
            };
            let y = match app.debug.position {
                DebugPosition::Top | DebugPosition::TopRight | DebugPosition::TopLeft => 0,
                DebugPosition::Right | DebugPosition::Left => area.height / 2 - height / 2,
                DebugPosition::BottomRight | DebugPosition::Bottom | DebugPosition::BottomLeft => {
                    area.height - height
                }
            };

            let debug_block = Paragraph::new(debug_lines).block(
                Block::new()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(Color::from_str("#ff0000").unwrap()))
                    .fg(Color::from_str("#ff0000").unwrap())
                    .bg(Color::from_str("#000000").unwrap()),
            );
            let debug_area = Rect::new(x, y, width, height);
            frame.render_widget(Clear, debug_area);
            frame.render_widget(debug_block, debug_area)
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

    fn status_bar(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;
        let mode_colors = app.mode_colors();

        let [left_layout, center_layout, right_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .areas(area);

        let mut mode_fg = mode_colors.fg;
        let mut mode_bg = mode_colors.bg;
        let mut status_bar_fg = colors.status_bar_fg;
        let mut status_bar_bg = colors.status_bar_bg;
        if app.mode.is_delete() {
            mode_fg = colors.status_bar_fg;
            mode_bg = colors.status_bar_bg;
            status_bar_fg = mode_colors.fg;
            status_bar_bg = mode_colors.bg;
        }
        let left_text = vec![Line::from(vec![
            Span::from(format!(" {} ", app.mode.to_string().to_uppercase()))
                .bold()
                .fg(mode_fg)
                .bg(mode_bg),
            Span::from("").fg(mode_bg),
            if app.mode.is_delete() {
                Span::from(" Confirm Deletion (y/n)").bold()
            } else {
                Span::from("")
            },
        ])];
        let left_content = Paragraph::new(left_text)
            .alignment(Alignment::Left)
            .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(left_content, left_layout);

        let center_text = vec![Line::from(vec![Span::from(format!(
            "Session duration: {}",
            if let Some(started) = &app.db.started {
                DateTime::new().duration_since(started).to_string()
            } else {
                "<pending>".to_string()
            }
        ))])];
        let center_content = Paragraph::new(center_text)
            .alignment(Alignment::Center)
            .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(center_content, center_layout);

        let right_text = vec![Line::from(vec![Span::from("Press ? for help ")])];
        let right_content = Paragraph::new(if app.mode.is_delete() {
            vec![Line::from("")]
        } else {
            right_text
        })
        .alignment(Alignment::Right)
        .style(Style::new().fg(status_bar_fg).bg(status_bar_bg));
        frame.render_widget(right_content, right_layout);
    }
}
