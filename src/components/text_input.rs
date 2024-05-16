use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use crate::{state::Mode, utils::normal_to_insert, App};

pub enum TextInputEvent {
    OnChange,
    None,
}

#[derive(Clone)]
pub struct TextInput {
    pub input: Vec<String>,
    pub cursor_position: (usize, usize),
    title: Option<String>,
    placeholder: Option<String>,
    // multiline: bool,
    required: bool,
    min: Option<usize>,
    max: Option<usize>,
    mode: Mode,
}

impl TextInput {
    #[allow(clippy::new_without_default)]
    pub fn new(mode: Mode) -> TextInput {
        TextInput {
            input: vec![String::from("")],
            cursor_position: (0, 0),
            title: None,
            placeholder: None,
            required: false,
            min: None,
            max: None,
            mode,
        }
    }

    pub fn input(&mut self, input: String) {
        self.input = input.split('\n').map(|s| s.to_string()).collect();
        self.cursor_end_line();
    }

    pub fn input_string(&self) -> String {
        self.input.join("\n")
    }

    pub fn reset(&mut self) {
        self.input = vec![String::new()];
        self.cursor_position = (0, 0);
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    pub fn is_min(&self) -> bool {
        if let Some(min) = self.min {
            self.input[self.cursor_position.1].chars().count() >= min
        } else {
            true
        }
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    pub fn required_len(mut self, length: usize) -> Self {
        self.min = Some(length);
        self.max = Some(length);
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
            if self.input[self.cursor_position.1].chars().count() == max {
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
                .take(self.cursor_position.0.saturating_sub(1));
            let after_char_to_delete = self.input[self.cursor_position.1]
                .chars()
                .skip(self.cursor_position.0);
            self.input[self.cursor_position.1] =
                before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input[self.cursor_position.1].chars().count())
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
        self.cursor_position.0 = self.input[self.cursor_position.1].chars().count()
    }

    fn enter_insert_mode(&self, app: &mut App) {
        app.state.mode = normal_to_insert(app.state.mode);
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
    pub fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) -> TextInputEvent {
        let mut event = TextInputEvent::None;

        if app.state.mode == normal_to_insert(self.mode) {
            match key_event.code {
                KeyCode::Char(to_insert) => {
                    self.enter_char(to_insert);
                    event = TextInputEvent::OnChange;
                }
                KeyCode::Backspace => {
                    self.delete_char();
                    event = TextInputEvent::OnChange;
                }
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                _ => {}
            }
        }

        if (self.mode == Mode::Navigation && app.state.mode == Mode::Navigation)
            || (self.mode == Mode::Popup && app.state.mode == Mode::Popup)
            || (self.mode == Mode::Command && app.state.mode == Mode::Command)
        {
            match key_event.code {
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
                    self.enter_insert_mode(app);
                    self.cursor_end_line()
                }
                _ => {}
            }
        }

        event
    }

    fn render_lines<'a>(
        &self,
        app: &App,
        width: u16,
        _: u16,
        focused: bool,
        side_space: bool,
    ) -> Vec<Line<'a>> {
        let colors = &app.config.colors;

        let input = if !self.input.is_empty()
            && self.input[self.cursor_position.1].is_empty()
            && self.cursor_position == (0, 0)
        {
            if !focused {
                if let Some(placeholder) = &self.placeholder {
                    vec![Line::from(vec![
                        Span::from(if side_space { " " } else { "" }),
                        Span::from(placeholder.clone()).style(Style::new().fg(colors.secondary)),
                    ])]
                } else {
                    vec![]
                }
            } else {
                vec![Line::from(vec![
                    Span::from(if side_space { " " } else { "" }),
                    Span::from(" ").style(if self.is_insert_mode(app) {
                        Style::new().fg(colors.bg).bg(colors.fg)
                    } else {
                        Style::new().fg(colors.bg).bg(colors.secondary)
                    }),
                ])]
            }
        } else {
            // TODO: Implement multiline by spliting \n
            // TODO: Scrollable view (not with the widget) for when there are more lines to
            // display than `height`

            // Reduce by 2 for the border width and 1 for the cursor.
            let border_width = 2;
            let cursor_width = 1;
            let side_space_width = if side_space { 1 } else { 0 };
            let line_length =
                width.saturating_sub(border_width + cursor_width + side_space_width) as usize;

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
                        if self.is_insert_mode(app) {
                            style = style
                                .fg(colors.input_cursor_insert_fg)
                                .bg(colors.input_cursor_insert_bg)
                        } else {
                            style = style.fg(colors.input_cursor_fg).bg(colors.input_cursor_bg)
                        }
                    }
                    let mut span = vec![Span::from(character.to_string()).style(style)];

                    let is_last_char = real_line_x_value
                        == self.input[self.cursor_position.1]
                            .chars()
                            .count()
                            .saturating_sub(1);
                    let cursor_not_at_start = self.cursor_position != (0, 0);
                    let char_is_before_cursor =
                        (real_line_x_value + 1, line_index) == self.cursor_position;

                    // Render the cursor (only if last character)
                    if focused && is_last_char && cursor_not_at_start && char_is_before_cursor {
                        span.push(Span::from(" ").style(if self.is_insert_mode(app) {
                            Style::new()
                                .fg(colors.input_cursor_insert_fg)
                                .bg(colors.input_cursor_insert_bg)
                        } else {
                            Style::new()
                                .fg(colors.input_cursor_fg)
                                .bg(colors.input_cursor_bg)
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
                            let mut line = chunk
                                .iter()
                                .enumerate()
                                .flat_map(|(i, c)| {
                                    render_char(((line_index, line), (chunk_index, chunk), (i, c)))
                                })
                                .collect::<Vec<Span>>();
                            if side_space {
                                line.insert(0, Span::from(" "));
                                line.push(Span::from(" "));
                            }
                            Line::from(line)
                        })
                        .collect::<Vec<Line>>()
                })
                .collect::<Vec<Line>>()
        };

        input
    }

    pub fn render_text<'a>(&self, app: &App, width: u16, height: u16, focused: bool) -> Text<'a> {
        let colors = &app.config.colors;
        let input_lines = self.render_lines(app, width, height, focused, true);
        Text::from(input_lines).style(if focused {
            Style::new()
                .fg(colors.input_focus_fg)
                .bg(colors.input_focus_bg)
        } else {
            Style::new().fg(colors.input_fg).bg(colors.input_bg)
        })
    }

    pub fn render_block(&self, app: &App, width: u16, height: u16, focused: bool) -> impl Widget {
        let colors = &app.config.colors;

        let text = self.render_lines(app, width, height, focused, false);
        Paragraph::new(text).block(
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
                    if self.is_insert_mode(app) {
                        colors.status_bar_insert_mode_bg
                    } else {
                        colors.primary
                    }
                } else {
                    colors.border
                })),
        )
    }

    fn is_insert_mode(&self, app: &App) -> bool {
        app.state.mode == normal_to_insert(self.mode)
    }
}
