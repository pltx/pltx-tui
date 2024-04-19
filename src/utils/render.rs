use crossterm::event::{Event, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    Frame,
};

use crate::{state::State, App};

pub trait InitData {
    /// This function should be called at the same time as `init()`.
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()>;
}

pub trait InitScreen {
    /// Any data which should only be fetched once should be done in the
    /// `init()` function, as the `render()` function runs in a loop.
    fn init(app: &mut App) -> Self
    where
        Self: Sized;
}
pub trait RenderScreen {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect);
}

pub trait RenderPopup {
    fn render(&mut self, frame: &mut Frame, app: &mut App);
    fn render_widgets_into_scrollview(&self, buf: &mut Buffer, app: &App);
}

pub trait EventHandler {
    fn event_handler(event: &Event, app: &mut App);
}

pub trait KeyEventHandler {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State);
}

pub trait ScreenKeybinds {
    /// Returns a list of keybinds to be shown as the bottom title of the screen
    fn screen_keybinds<'a>(&mut self) -> [(&'a str, &'a str); 3];
}

pub trait ScreenKeybindsTitle {
    /// Return the title for the keybinds
    fn screen_keybinds_title(&mut self, app: &mut App) -> Line;
}

/// Creates the title that shows the available keybinds for a screen
pub fn pane_title_bottom<'a>(app: &mut App, hints: [(&'a str, &'a str); 3]) -> Line<'a> {
    let colors = &app.config.colors;
    let separator = "──";
    let hints_line = hints
        .iter()
        .flat_map(|h| {
            vec![
                Span::from(format!("{} ", separator)).fg(colors.border),
                Span::from(h.0).bold().fg(colors.keybind_key),
                Span::from(" ➜ ").fg(colors.secondary),
                Span::from(format!("{} ", h.1)).fg(colors.keybind_fg),
            ]
        })
        .collect::<Vec<Span>>();
    Line::from([hints_line, vec![Span::from(separator).fg(colors.border)]].concat())
}
