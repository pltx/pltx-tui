use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use crate::{state::Mode, App};

pub struct TextInput {
    pub input: String,
    pub cursor_position: usize,
    title: Option<String>,
    min: Option<usize>,
    max: Option<usize>,
}

impl TextInput {
    pub const fn new() -> TextInput {
        TextInput {
            input: String::new(),
            cursor_position: 0,
            title: None,
            min: None,
            max: None,
        }
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn reset(&mut self) {
        self.input = String::new();
        self.cursor_position = 0;
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn set_min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    pub fn is_min(&self) -> bool {
        if let Some(min) = self.min {
            self.input.len() >= min
        } else {
            true
        }
    }

    pub fn set_max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        if let Some(max) = self.max {
            if self.input.len() == max {
                return;
            }
        }
        self.input.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.cursor_position != 0 {
            let before_char_to_delete = self.input.chars().take(self.cursor_position - 1);
            let after_char_to_delete = self.input.chars().skip(self.cursor_position);
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn cursor_next_word(&mut self) {
        let next_word = self.input[..self.cursor_position]
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.to_string() == " " {
                    i
                } else {
                    self.cursor_position
                }
            })
            .collect::<Vec<usize>>();
        self.cursor_position = next_word[0];
    }

    fn cursor_start_line(&mut self) {
        self.cursor_position = 0;
    }

    pub fn cursor_end_line(&mut self) {
        self.cursor_position = self.input.len()
    }

    fn enter_insert_mode(&self, app: &mut App) {
        app.state.mode = match app.state.mode {
            Mode::Popup => Mode::PopupInsert,
            _ => Mode::Insert,
        }
    }

    // TODO:
    // j = move up (if multiline)
    // k = move down (if multiline)
    // w = next word
    // b = prev word
    // x = delete char in navigation mode
    // dd = delete line
    // u = undo
    // ctrl + r = redo
    // o = newline + insert mode
    pub fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) {
        match app.state.mode {
            Mode::Insert | Mode::PopupInsert => match key_event.code {
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                _ => {}
            },
            Mode::Navigation | Mode::Popup => match key_event.code {
                KeyCode::Char('h') => self.move_cursor_left(),
                KeyCode::Char('l') => self.move_cursor_right(),
                KeyCode::Char('w') => self.cursor_next_word(),
                KeyCode::Char('b') => {}
                KeyCode::Char('0') => self.cursor_start_line(),
                KeyCode::Char('$') => self.cursor_end_line(),
                KeyCode::Char('i') => self.enter_insert_mode(app),
                KeyCode::Char('I') => {
                    self.enter_insert_mode(app);
                    self.cursor_start_line()
                }
                KeyCode::Char('a') => {
                    self.enter_insert_mode(app);
                    self.move_cursor_right()
                }
                KeyCode::Char('A') => {
                    app.state.mode = Mode::Insert;
                    self.cursor_end_line()
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn render(&self, app: &App, focused: bool) -> impl Widget {
        let colors = &app.config.colors;

        let input = if focused && self.input.is_empty() && self.cursor_position == 0 {
            vec![Span::from(" ").style(if app.state.mode == Mode::Insert {
                Style::new().fg(colors.bg).bg(colors.fg)
            } else {
                Style::new().fg(colors.bg).bg(colors.secondary)
            })]
        } else {
            self.input
                .to_string()
                .chars()
                .enumerate()
                .flat_map(|(i, c)| {
                    let mut style = Style::new();
                    if focused && i == self.cursor_position {
                        style = style.fg(colors.bg).bg(if app.state.mode == Mode::Insert {
                            colors.fg
                        } else {
                            colors.secondary
                        });
                    }
                    let mut span = vec![Span::from(c.to_string()).style(style)];
                    if focused
                        && i == self.input.len() - 1
                        && self.cursor_position != 0
                        && i == self.cursor_position - 1
                    {
                        span.push(Span::from(" ").style(if app.state.mode == Mode::Insert {
                            Style::new().fg(colors.bg).bg(colors.fg)
                        } else {
                            Style::new().fg(colors.bg).bg(colors.secondary)
                        }));
                    }
                    span
                })
                .collect::<Vec<Span>>()
        };

        Paragraph::new(Line::from(input)).block(
            Block::new()
                .padding(Padding::horizontal(1))
                .title(if let Some(title) = &self.title {
                    format!(" {title} ")
                } else {
                    "".to_string()
                })
                .title_style(Style::new().fg(if focused { colors.fg } else { colors.secondary }))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(if focused {
                    if app.state.mode == Mode::Insert || app.state.mode == Mode::PopupInsert {
                        colors.status_bar_insert_mode_bg
                    } else {
                        colors.primary
                    }
                } else {
                    colors.border
                })),
        )
    }
}
