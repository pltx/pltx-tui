use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{state::Display, App, DefaultWidget, FormWidget, KeyEventHandler};
use pltx_utils::DateTime;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

const WORD_SEPARATORS: [char; 1] = [' '];

// pub enum TextInputEvent {
//     OnChange,
//     None,
// }

#[derive(Clone)]
enum TextInputType {
    Text,
    Date,
}

#[derive(Clone, Default)]
struct TextInputSize {
    width: u16,
    // height: u16,
}

#[derive(Clone, Default)]
struct CursorPosition {
    x: usize,
    y: usize,
}

impl CursorPosition {
    pub fn at_start(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn is(&self, x: usize, y: usize) -> bool {
        x == self.x && y == self.y
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }
}

#[derive(Clone, Default)]
struct KeyManager {
    command_keys: Vec<KeyCode>,
    count: Option<usize>,
}

impl KeyManager {
    pub fn add_key(&mut self, key_code: KeyCode) {
        self.command_keys.push(key_code)
    }

    pub fn key_is(&self, key_code: KeyCode) -> bool {
        !self.command_keys.is_empty() && self.command_keys[0] == key_code
    }

    pub fn clear(&mut self) {
        self.command_keys.clear();
        self.count = None;
    }
}

/// TextInput widget
#[derive(Clone)]
pub struct TextInput {
    input: Vec<String>,
    cursor_position: CursorPosition,
    input_type: TextInputType,
    title: String,
    max_title_len: u16,
    placeholder: Option<String>,
    title_as_placeholder: bool,
    required: bool,
    min: Option<usize>,
    max: Option<usize>,
    display: Display,
    inline: bool,
    form_input: bool,
    use_size: bool,
    size: TextInputSize,
    keys: KeyManager,
}

impl DefaultWidget for TextInput {
    fn render(
        &self,
        frame: &mut ratatui::prelude::Frame,
        app: &App,
        area: ratatui::prelude::Rect,
        focused: bool,
    ) {
        if self.inline {
            let [title_layout, input_layout] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(self.max_title_len + 2),
                    Constraint::Min(0),
                ])
                .areas(area);

            if !self.title_as_placeholder {
                frame.render_widget(Paragraph::new(format!("{}: ", self.title)), title_layout);
            }
            let widget = self.render_text(app, input_layout, focused);
            frame.render_widget(
                widget,
                if self.title_as_placeholder {
                    area
                } else {
                    input_layout
                },
            );
        } else {
            let widget = self.render_block(app, area, focused);
            frame.render_widget(widget, area);
        }
    }
}

impl FormWidget for TextInput {
    fn form_compatible(&mut self) {
        self.inline = true;
        self.form_input = true;
    }

    fn display(&mut self, display: Display) {
        self.display = display;
    }

    fn title_len(&self) -> u16 {
        self.title.chars().count() as u16
    }

    fn max_title_len(&mut self, max_title: u16) {
        self.max_title_len = max_title;
    }

    fn reset(&mut self) {
        self.input = vec![String::new()];
        self.cursor_position.at_start();
    }
}

impl KeyEventHandler for TextInput {
    // TODO:
    // add a view only option
    // provide a way for users to deal with text input events like OnChange
    // j = move up (if multiline)
    // k = move down (if multiline)
    // u = undo
    // ctrl + r = redo
    // o = newline + insert mode
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        // let mut event = TextInputEvent::None;

        if app.display == self.display.insert_equivalent() {
            match key_event.code {
                KeyCode::Char(to_insert) => {
                    self.enter_char(to_insert);
                    // event = TextInputEvent::OnChange;
                }
                KeyCode::Backspace => {
                    self.delete_char();
                    // event = TextInputEvent::OnChange;
                }
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => app.normal_mode(),
                _ => {}
            }
        }

        if app.display == self.display {
            match key_event.code {
                KeyCode::Char('h') | KeyCode::Left => self.move_cursor_left(),
                KeyCode::Char('l') | KeyCode::Right => self.move_cursor_right(),
                KeyCode::Char('w') => self.cursor_next_word(),
                KeyCode::Char('b') => self.cursor_prev_word(),
                KeyCode::Char('0') => self.cursor_start_line(),
                KeyCode::Char('$') => self.cursor_end_line(),
                KeyCode::Char('i') => {
                    app.insert_mode();
                    self.keys.clear();
                }
                KeyCode::Char('I') => {
                    app.insert_mode();
                    self.cursor_start_line();
                }
                KeyCode::Char('a') => {
                    app.insert_mode();
                    self.move_cursor_right();
                }
                KeyCode::Char('A') => {
                    app.insert_mode();
                    self.cursor_end_line();
                }
                KeyCode::Char('x') => self.delete_char_forward(),
                KeyCode::Char('d') => self.delete_line(),
                KeyCode::Esc => self.keys.clear(),
                _ => {}
            }
        }
    }
}

impl TextInput {
    pub fn new(title: &str) -> Self {
        Self {
            input: vec![String::new()],
            cursor_position: CursorPosition::default(),
            input_type: TextInputType::Text,
            title: String::from(title),
            max_title_len: title.chars().count() as u16,
            placeholder: None,
            title_as_placeholder: false,
            required: false,
            min: None,
            max: None,
            display: Display::default(),
            inline: false,
            form_input: false,
            use_size: false,
            size: TextInputSize::default(),
            keys: KeyManager::default(),
        }
    }

    pub fn default_input(mut self, input: String) -> Self {
        self.input = input.split('\n').map(|s| s.to_string()).collect();
        self.cursor_end_line();
        self
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
        self.cursor_position.reset();
    }

    pub fn datetime_input(mut self) -> Self {
        self.input_type = TextInputType::Date;
        self.placeholder = Some(DateTime::display_now());
        self.min = Some(16);
        self.max = Some(16);
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    pub fn title_as_placeholder(mut self) -> Self {
        self.title_as_placeholder = true;
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
            self.input[self.cursor_position.y].chars().count() >= min
        } else {
            true
        }
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    pub fn display(mut self, display: Display) -> Self {
        self.display = display;
        self
    }

    pub fn required_len(mut self, length: usize) -> Self {
        self.min = Some(length);
        self.max = Some(length);
        self
    }

    pub fn size(mut self, size: (u16, u16)) -> Self {
        self.use_size = true;
        self.size = TextInputSize {
            width: size.0,
            // height: size.1,
        };
        self
    }

    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    pub fn is_empty(&self) -> bool {
        if self.input.is_empty() || self.input_string().chars().count() == 0 {
            return true;
        }
        false
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.x.saturating_sub(1);
        self.cursor_position.x = self.clamp_cursor(cursor_moved_left);
        self.keys.clear();
    }

    fn move_cursor_right(&mut self) {
        if !(self.input[self.cursor_position.y].is_empty() && self.cursor_position.x == 0) {
            let cursor_moved_right = self.cursor_position.x.saturating_add(1);
            self.cursor_position.x = self.clamp_cursor(cursor_moved_right);
        }
        self.keys.clear();
    }

    fn enter_char(&mut self, new_char: char) {
        if let Some(max) = self.max {
            if self.input[self.cursor_position.y].chars().count() == max {
                return;
            }
        }
        self.input[self.cursor_position.y].insert(self.cursor_position.x, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if !self.cursor_position.at_start() {
            let before_char_to_delete = self.input[self.cursor_position.y]
                .chars()
                .take(self.cursor_position.x.saturating_sub(1));
            let after_char_to_delete = self.input[self.cursor_position.y]
                .chars()
                .skip(self.cursor_position.x);
            self.input[self.cursor_position.y] =
                before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn delete_char_forward(&mut self) {
        let before_char_to_delete = self.input[self.cursor_position.y]
            .chars()
            .take(self.cursor_position.x);
        let after_char_to_delete = self.input[self.cursor_position.y]
            .chars()
            .skip(self.cursor_position.x + 1);
        self.input[self.cursor_position.y] =
            before_char_to_delete.chain(after_char_to_delete).collect();
    }

    fn delete_line(&mut self) {
        if self.keys.key_is(KeyCode::Char('d')) {
            self.input[self.cursor_position.y].clear();
            self.cursor_position.x = 0;
            self.keys.clear();
        } else {
            self.keys.add_key(KeyCode::Char('d'));
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input[self.cursor_position.y].chars().count())
    }

    fn cursor_next_word(&mut self) {
        let next_word = self.input[self.cursor_position.y]
            .chars()
            .enumerate()
            .find(|(i, c)| *i > self.cursor_position.x && WORD_SEPARATORS.contains(c))
            .unwrap_or((self.input[self.cursor_position.y].chars().count(), ' '));
        self.cursor_position.x =
            if self.input[self.cursor_position.y].chars().count() == next_word.0 {
                next_word.0
            } else {
                next_word.0 + 1
            };
        self.keys.clear();
    }

    fn cursor_prev_word(&mut self) {
        let line_len = self.input[self.cursor_position.y].chars().count();
        let prev_word = self.input[self.cursor_position.y]
            .chars()
            .rev()
            .enumerate()
            .find(|(i, c)| {
                line_len - *i - 1 < self.cursor_position.x && WORD_SEPARATORS.contains(c)
            })
            .unwrap_or((line_len, ' '));
        self.cursor_position.x = (line_len - prev_word.0).saturating_sub(2);
        self.keys.clear();
    }

    fn cursor_start_line(&mut self) {
        self.cursor_position.x = 0;
        self.keys.clear();
    }

    pub fn cursor_end_line(&mut self) {
        self.cursor_position.x = self.input[self.cursor_position.y].chars().count();
        self.keys.clear();
    }

    fn render_lines<'a>(&self, app: &App, area: Rect, focused: bool) -> Vec<Line<'a>> {
        let colors = &app.config.colors;

        let input = if !self.input.is_empty()
            && self.input[self.cursor_position.y].is_empty()
            && self.cursor_position.at_start()
        {
            if !focused {
                if let Some(placeholder) = &self.placeholder {
                    vec![Line::from(vec![
                        Span::from(if self.inline { " " } else { "" }),
                        Span::from(placeholder.clone()).style(Style::new().fg(colors.secondary_fg)),
                    ])]
                } else {
                    vec![]
                }
            } else {
                vec![Line::from(vec![
                    Span::from(if self.inline { " " } else { "" }),
                    Span::from(" ").style(if app.display == self.display.insert_equivalent() {
                        Style::new().fg(colors.bg).bg(colors.fg)
                    } else {
                        Style::new().fg(colors.bg).bg(colors.secondary_fg)
                    }),
                ])]
            }
        } else {
            // TODO: Implement multiline by spliting \n
            // TODO: Scrollable view (not with the widget) for when there are more lines to
            // display than `height`

            let border_width = 2;
            let cursor_width = 1;
            let side_space_width = if self.inline { 1 } else { 0 };
            let form_width = if self.form_input { 4 } else { 0 };
            let width = if self.use_size {
                self.size.width
            } else {
                area.width - 2
            };
            let line_length = width.saturating_sub(border_width + cursor_width + side_space_width)
                as usize
                + form_width;

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

                    let cursor_on_char = self.cursor_position.is(real_line_x_value, line_index);
                    if focused && cursor_on_char {
                        if app.display == self.display.insert_equivalent() {
                            style = style
                                .fg(colors.input_cursor_insert_fg)
                                .bg(colors.input_cursor_insert_bg)
                        } else {
                            style = style.fg(colors.input_cursor_fg).bg(colors.input_cursor_bg)
                        }
                    }
                    let mut span = vec![Span::from(character.to_string()).style(style)];

                    let is_last_char = real_line_x_value
                        == self.input[self.cursor_position.y]
                            .chars()
                            .count()
                            .saturating_sub(1);
                    let char_is_before_cursor =
                        self.cursor_position.is(real_line_x_value + 1, line_index);

                    // Render the cursor (only if last character)
                    if focused
                        && is_last_char
                        && !self.cursor_position.at_start()
                        && char_is_before_cursor
                    {
                        span.push(Span::from(" ").style(
                            if app.display == self.display.insert_equivalent() {
                                Style::new()
                                    .fg(colors.input_cursor_insert_fg)
                                    .bg(colors.input_cursor_insert_bg)
                            } else {
                                Style::new()
                                    .fg(colors.input_cursor_fg)
                                    .bg(colors.input_cursor_bg)
                            },
                        ));
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
                            if self.inline {
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

    fn render_text(&self, app: &App, area: Rect, focused: bool) -> impl Widget {
        let colors = &app.config.colors;
        let input_lines = self.render_lines(app, area, focused);
        Text::from(input_lines).style(if focused {
            Style::new()
                .fg(colors.input_focus_fg)
                .bg(colors.input_focus_bg)
        } else {
            Style::new().fg(colors.input_fg).bg(colors.input_bg)
        })
    }

    fn render_block(&self, app: &App, area: Rect, focused: bool) -> impl Widget {
        let colors = &app.config.colors;

        let text = self.render_lines(app, area, focused);
        Paragraph::new(text).block(
            Block::new()
                .padding(Padding::horizontal(1))
                .title(format!(" {} ", self.title))
                .title_style(Style::new().fg(if focused {
                    colors.fg
                } else {
                    colors.secondary_fg
                }))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(if focused {
                    if app.display == self.display.insert_equivalent() {
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
