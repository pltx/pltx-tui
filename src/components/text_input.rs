use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use crate::{state::Mode, trace_debug, App};

pub struct TextInput {
    pub input: Vec<String>,
    // (x, y)
    pub cursor_position: (usize, usize),
    title: Option<String>,
    multiline: bool,
    min: Option<usize>,
    max: Option<usize>,
}

impl TextInput {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TextInput {
        TextInput {
            input: vec![String::from("")],
            cursor_position: (0, 0),
            title: None,
            multiline: false,
            min: None,
            max: None,
        }
    }

    pub fn set_input(&mut self, input: Vec<String>) {
        self.input = input;
    }

    pub fn reset(&mut self) {
        self.input = vec![String::new()];
        self.cursor_position = (0, 0);
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
            self.input[self.cursor_position.1].len() >= min
        } else {
            true
        }
    }

    pub fn set_max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.0.saturating_sub(1);
        self.cursor_position.0 = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        if !(self.input[self.cursor_position.1].is_empty() && self.cursor_position.0 == 0) {
            let cursor_moved_right = self.cursor_position.0.saturating_add(1);
            self.cursor_position.0 = self.clamp_cursor(cursor_moved_right);
        }
    }

    fn enter_char(&mut self, new_char: char) {
        if let Some(max) = self.max {
            if self.input[self.cursor_position.1].len() == max {
                return;
            }
        }
        self.input[self.cursor_position.1].insert(self.cursor_position.0, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.cursor_position != (0, 0) {
            let before_char_to_delete = self.input[self.cursor_position.1]
                .chars()
                .take(self.cursor_position.0 - 1);
            let after_char_to_delete = self.input[self.cursor_position.1]
                .chars()
                .skip(self.cursor_position.0);
            self.input[self.cursor_position.1] =
                before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input[self.cursor_position.1].len())
    }

    fn cursor_next_word(&mut self) {
        let next_word = self.input[self.cursor_position.1][..self.cursor_position.0]
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if c.to_string() == " " {
                    i
                } else {
                    self.cursor_position.0
                }
            })
            .collect::<Vec<usize>>();
        self.cursor_position.0 = next_word[0];
    }

    fn cursor_start_line(&mut self) {
        self.cursor_position.0 = 0;
    }

    pub fn cursor_end_line(&mut self) {
        self.cursor_position.0 = self.input[self.cursor_position.1].len()
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

    pub fn render(&self, app: &App, width: u16, _: u16, focused: bool) -> impl Widget {
        let colors = &app.config.colors;

        let input = if focused
            && !self.input.is_empty()
            && self.input[self.cursor_position.1].is_empty()
            && self.cursor_position == (0, 0)
        {
            vec![Line::from(vec![Span::from(" ").style(
                if app.state.mode == Mode::Insert {
                    Style::new().fg(colors.bg).bg(colors.fg)
                } else {
                    Style::new().fg(colors.bg).bg(colors.secondary)
                },
            )])]
        } else {
            // TODO: Implement multiline by spliting \n
            // TODO: Scrollable view (not with the widget) for when there are more lines to
            // display than `height`

            // Reduce by 2 for the border width and 1 for the cursor.
            let line_length = width as usize - 2 - 1;

            type RenderCharType<'a> = ((usize, &'a String), (usize, &'a [char]), (usize, &'a char));
            let render_char =
                |((line_index, line), (chunk_index, _), (index, character)): RenderCharType| {
                    let mut style = Style::new();

                    let line_chars = line.chars().collect::<Vec<char>>();
                    let line_chunks = line_chars.chunks(line_length).collect::<Vec<&[char]>>();
                    let prev_chunks = line_chunks[..chunk_index]
                        .iter()
                        .map(|c| c.len())
                        .collect::<Vec<usize>>();
                    let prev_chunks_total = prev_chunks.iter().sum::<usize>();
                    let real_line_x_value = prev_chunks_total + index;

                    let cursor_on_char = (real_line_x_value, line_index) == self.cursor_position;
                    if focused && cursor_on_char {
                        style = style.fg(colors.bg).bg(if app.state.mode == Mode::Insert {
                            colors.fg
                        } else {
                            colors.secondary
                        });
                    }
                    let mut span = vec![Span::from(character.to_string()).style(style)];

                    let is_last_char =
                        real_line_x_value == self.input[self.cursor_position.1].len() - 1;
                    let cursor_not_at_start = self.cursor_position != (0, 0);
                    let char_is_before_cursor =
                        (real_line_x_value + 1, line_index) == self.cursor_position;

                    // Render the cursor (only if last character)
                    if focused && is_last_char && cursor_not_at_start && char_is_before_cursor {
                        span.push(Span::from(" ").style(if app.state.mode == Mode::Insert {
                            Style::new().fg(colors.bg).bg(colors.fg)
                        } else {
                            Style::new().fg(colors.bg).bg(colors.secondary)
                        }));
                    }

                    span
                };
            self.input
                .iter()
                .enumerate()
                .flat_map(|(line_index, line)| {
                    line.chars()
                        .collect::<Vec<char>>()
                        .chunks(line_length)
                        .enumerate()
                        .map(|(chunk_index, chunk)| {
                            let line = chunk
                                .iter()
                                .enumerate()
                                .flat_map(|(i, c)| {
                                    render_char(((line_index, line), (chunk_index, chunk), (i, c)))
                                })
                                .collect::<Vec<Span>>();
                            Line::from(line)
                        })
                        .collect::<Vec<Line>>()
                })
                .collect::<Vec<Line>>()
        };

        Paragraph::new(input).block(
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
