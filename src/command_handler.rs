use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use nucleo::{
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
    Matcher,
};
use pltx_app::{
    state::{AppModule, View},
    App, DefaultWidget, KeyEventHandler,
};
use pltx_widgets::{PopupSize, PopupWidget, TextInput};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use crate::ui::Interface;

#[derive(PartialEq, Clone)]
/// The list of available commands. Each must be added to the [`command_data`]
/// function.
enum Command {
    Dashboard,
    Help,
    Home,
    ProjectManagement,
    Quit,
    Settings,
    None,
}

#[derive(PartialEq)]
enum CommandView {
    Input,
    // Output,
}

#[derive(PartialEq)]
enum FocusedPane {
    Input,
    Options,
}
pub struct CommandHandler<'a> {
    command: TextInput,
    size: PopupSize,
    command_view: CommandView,
    focused_pane: FocusedPane,
    command_options: Vec<&'a str>,
    selected_option: usize,
    matcher: Matcher,
}

// NOTE: Add commands here.
fn command_data<'a>() -> [(Command, &'a str); 6] {
    [
        (Command::Dashboard, "dashboard"),
        (Command::Help, "help"),
        (Command::Home, "home"),
        (Command::ProjectManagement, "project management"),
        (Command::Settings, "settings"),
        (Command::Quit, "quit"),
    ]
}

impl<'a> CommandHandler<'a> {
    pub fn init() -> CommandHandler<'a> {
        let start = Instant::now();
        let size = PopupSize::default().width(60).height(20);
        let command_handler = CommandHandler {
            command: TextInput::new("Command")
                .view(View::Command)
                .size((size.width - 2, size.height - 2))
                .placeholder("Enter a command...")
                .max(50),
            size,
            command_view: CommandView::Input,
            focused_pane: FocusedPane::Input,
            command_options: command_data().iter().map(|s| s.1).collect(),
            selected_option: 0,
            matcher: Matcher::default(),
        };
        tracing::info!("initialized command handler in {:?}", start.elapsed());
        command_handler
    }

    pub fn key_event_handler(
        &mut self,
        app: &mut App,
        interface: &mut Interface,
        key_event: KeyEvent,
    ) {
        if self.focused_pane == FocusedPane::Input {
            self.command.key_event_handler(app, key_event);
            self.update_options();
        }

        if app.mode.is_normal() {
            match key_event.code {
                KeyCode::Enter => self.execute_command(app, interface),
                KeyCode::Char('q') => {
                    app.view.default();
                    self.reset();
                }
                KeyCode::Char('j') => {
                    if self.command_view == CommandView::Input {
                        if self.focused_pane == FocusedPane::Input {
                            self.focused_pane = FocusedPane::Options;
                        } else if self.selected_option + 1 != self.command_options.len() {
                            self.selected_option += 1;
                        }
                    }
                }
                KeyCode::Char('k') => {
                    if self.command_view == CommandView::Input
                        && self.focused_pane == FocusedPane::Options
                    {
                        if self.selected_option != 0 {
                            self.selected_option -= 1;
                        } else {
                            self.focused_pane = FocusedPane::Input;
                        }
                    }
                }
                _ => {}
            }
        } else if app.mode.is_insert() {
            match key_event.code {
                KeyCode::Enter => self.execute_command(app, interface),
                KeyCode::Esc => app.view.command(),
                _ => {}
            }
        }
    }

    pub fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let popup = PopupWidget::new(app, area).size(self.size).render(frame);

        let [input_layout, command_list_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(popup.popup_area);

        self.command.render(
            frame,
            app,
            input_layout,
            self.focused_pane == FocusedPane::Input,
        );

        let text = if self.command_options.is_empty() {
            Text::from("No commands found.")
        } else {
            Text::from(
                self.command_options
                    .iter()
                    .enumerate()
                    .map(|(i, o)| {
                        Line::from(format!(" {o} ")).style(if i == self.selected_option {
                            Style::new()
                                .bold()
                                .fg(colors.active_fg)
                                .bg(colors.active_bg)
                        } else {
                            Style::new().fg(colors.secondary_fg)
                        })
                    })
                    .collect::<Vec<Line>>(),
            )
        };

        let command_list = Paragraph::new(text).block(
            Block::new()
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::new().fg(if self.focused_pane == FocusedPane::Options {
                        colors.border_active
                    } else {
                        colors.border
                    }),
                ),
        );

        frame.render_widget(command_list, command_list_layout);
    }
}

impl<'a> CommandHandler<'a> {
    fn reset(&mut self) {
        self.focused_pane = FocusedPane::Input;
        self.command.reset();
        self.update_options();
    }

    fn parse_command(&self) -> (Command, &str) {
        if self.command_options.is_empty() {
            return (Command::None, "none");
        }
        let command_str = self.command_options[self.selected_option];
        for command in command_data() {
            if command.1.contains(command_str) {
                return (command.0, command_str);
            }
        }
        (Command::None, "none")
    }

    fn execute_command(&mut self, app: &mut App, interface: &mut Interface) {
        let start = Instant::now();

        let (command, command_str) = self.parse_command();

        let _span = tracing::info_span!("command handler", command = command_str).entered();

        match command {
            Command::Dashboard => {
                app.view.default();
                app.mode.normal();
                app.module = AppModule::Home;
                interface.modules.home.dashboard();
            }
            Command::Settings => {
                app.view.default();
                app.mode.normal();
                app.module = AppModule::Home;
                interface.modules.home.settings();
            }
            Command::Help => {
                app.view.default();
                app.mode.normal();
                app.module = AppModule::Home;
                interface.modules.home.help();
            }
            Command::Home => {
                app.view.default();
                app.mode.normal();
                app.module = AppModule::Home;
                interface.modules.home.dashboard();
            }
            Command::ProjectManagement => {
                app.view.default();
                app.mode.normal();
                app.module = AppModule::ProjectManagement;
            }
            Command::Quit => app.exit(),
            Command::None => {}
        }

        if command != Command::None {
            self.reset();
            tracing::info!("executed command in {:?}", start.elapsed());
        }
    }

    fn update_options(&mut self) {
        self.selected_option = 0;
        let is_longer_than_longest_option = self.command.input_string().chars().count()
            > command_data()
                .iter()
                .map(|c| c.1.chars().count())
                .max()
                .unwrap_or(0);
        if is_longer_than_longest_option {
            self.command_options = vec![];
        } else if self.command.input_string().chars().count() == 0 {
            self.command_options = command_data().iter().map(|c| c.1).collect::<Vec<&str>>();
        } else {
            let pattern = Atom::new(
                &self.command.input_string(),
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Fuzzy,
                false,
            );
            self.command_options = pattern
                .match_list(
                    command_data().iter().map(|c| c.1).collect::<Vec<&str>>(),
                    &mut self.matcher,
                )
                .iter()
                .map(|s| s.0)
                .collect::<Vec<&str>>();
        }
    }
}
