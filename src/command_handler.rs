use crossterm::event::{KeyCode, KeyEvent};
use fst::{automaton::Levenshtein, IntoStreamer};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use crate::{
    components::{self, PopupSize, TextInput, TextInputEvent},
    state::{GlobalPopup, Mode, State},
    utils::{KeyEventHandler, RenderPopup},
    App,
};

#[derive(PartialEq, Clone)]
enum Command {
    Help,
    Quit,
    None,
}

#[derive(PartialEq)]
enum Display {
    CommandInput,
    // Output,
}

#[derive(PartialEq)]
enum FocusedPane {
    Input,
    Options,
}
pub struct CommandHandler {
    command: TextInput,
    size: PopupSize,
    display: Display,
    focused_pane: FocusedPane,
    command_set: fst::Set<Vec<u8>>,
    command_options: Vec<String>,
    selected_option: usize,
}

fn command_list<'a>() -> Vec<(Command, &'a str)> {
    fn get_command<'b>(cmd: Command) -> &'b str {
        match cmd {
            Command::Help => "help",
            Command::Quit => "quit",
            Command::None => "",
        }
    }

    // NOTE: This must be in lexicographic (alphabetical) order.
    let cmds = [Command::Help, Command::Quit];

    let mut list = vec![];
    for cmd in cmds {
        list.push((cmd.clone(), get_command(cmd)))
    }
    list
}

fn command_options_list<'a>() -> Vec<&'a str> {
    command_list().iter().map(|c| c.1).collect::<Vec<&str>>()
}

impl CommandHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CommandHandler {
        let command_set = fst::Set::from_iter(command_options_list()).unwrap();

        CommandHandler {
            command: TextInput::new(Mode::Command)
                .placeholder("Enter a command...")
                .max(50),
            size: PopupSize::new().width(60).height(20),
            display: Display::CommandInput,
            focused_pane: FocusedPane::Input,
            command_set,
            command_options: command_options_list()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            selected_option: 0,
        }
    }

    fn reset(&mut self) {
        self.command.reset();
        self.update_options();
        self.focused_pane = FocusedPane::Input;
    }

    fn parse_command(&self) -> Command {
        if self.command_options.is_empty() {
            return Command::None;
        }
        let command_str = self.command_options[self.selected_option].clone();
        for command in command_list() {
            if command.1.contains(&command_str) {
                return command.0;
            }
        }
        Command::None
    }

    fn execute_command(&mut self, app: &mut App) {
        let command = self.parse_command();
        match command {
            Command::Help => {
                app.state.mode = Mode::Popup;
                app.state.popup = GlobalPopup::Help;
            }
            Command::Quit => app.exit(),
            Command::None => {}
        }
        if command != Command::None {
            self.reset()
        }
    }

    fn update_options(&mut self) {
        let is_longer_than_longest_option = self.command.input_string().chars().count()
            > command_list()
                .iter()
                .map(|c| c.1.chars().count())
                .max()
                .unwrap_or(0);
        if is_longer_than_longest_option {
            self.command_options = vec![];
        } else if self.command.input_string().chars().count() == 0 {
            let command_list = command_list().iter().map(|c| c.1).collect::<Vec<&str>>();
            self.command_options = command_list.iter().map(|s| s.to_string()).collect();
        } else {
            let query = Levenshtein::new(&self.command.input_string(), 3).unwrap();
            let stream = self.command_set.search(query).into_stream();
            let keys = stream.into_strs().unwrap_or(vec![]);
            self.command_options = keys;
        }
        self.selected_option = 0;
    }
}

impl KeyEventHandler for CommandHandler {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) {
        if self.focused_pane == FocusedPane::Input {
            match self.command.key_event_handler(app, key_event) {
                TextInputEvent::OnChange => self.update_options(),
                TextInputEvent::None => {}
            }
        }

        if app.state.mode == Mode::Command {
            match key_event.code {
                KeyCode::Enter => self.execute_command(app),
                KeyCode::Char('q') => {
                    app.state.mode = Mode::Navigation;
                    self.reset();
                }
                KeyCode::Char('j') => {
                    if self.display == Display::CommandInput
                        && self.focused_pane == FocusedPane::Input
                    {
                        self.focused_pane = FocusedPane::Options;
                    }
                }
                KeyCode::Char('k') => {
                    if self.display == Display::CommandInput
                        && self.focused_pane == FocusedPane::Options
                    {
                        self.focused_pane = FocusedPane::Input;
                    }
                }
                _ => {}
            }
        } else if app.state.mode == Mode::CommandInsert {
            match key_event.code {
                KeyCode::Enter => self.execute_command(app),
                KeyCode::Esc => app.state.mode = Mode::Command,
                _ => {}
            }
        }
    }
}

impl RenderPopup for CommandHandler {
    fn render(&mut self, frame: &mut Frame, app: &App) {
        let colors = &app.config.colors;

        let popup = components::Popup::new(app, frame.size())
            .size(self.size.clone())
            .render(frame);

        let [input_layout, command_list_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(popup.area);

        frame.render_widget(
            self.command.render_block(
                app,
                self.size.width - 2,
                self.size.height - 2,
                self.focused_pane == FocusedPane::Input,
            ),
            input_layout,
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
                            Style::new().fg(colors.secondary)
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
                        colors.primary
                    } else {
                        colors.border
                    }),
                ),
        );

        frame.render_widget(command_list, command_list_layout);
    }
}
