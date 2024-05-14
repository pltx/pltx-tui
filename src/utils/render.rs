use crossterm::event::{Event, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    Frame,
};

use crate::{config::ColorsConfig, state::State, App};

pub trait InitData {
    /// This function should be called at the same time as `init()`.
    fn init_data(&mut self, app: &mut App) -> rusqlite::Result<()>;
}

pub trait Init {
    /// Any data which should only be fetched once should be done in the
    /// `init()` function, as the `render()` function runs in a loop.
    fn init(app: &mut App) -> Self
    where
        Self: Sized;
}

pub trait RenderScreen {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect);
}

pub trait RenderPage<T> {
    fn render(&mut self, app: &mut App, frame: &mut Frame, area: Rect, state: T);
}

pub trait RenderScrollPopup {
    fn render(&mut self, frame: &mut Frame, app: &mut App);
    fn render_widgets_into_scrollview(&mut self, buf: &mut Buffer, app: &App);
}

pub trait RenderPopup {
    fn render(&mut self, frame: &mut Frame, app: &App);
}

pub trait RenderPopupContained {
    fn render(&mut self, frame: &mut Frame, app: &App, area: Rect);
}

pub trait EventHandler {
    fn event_handler(event: &Event, app: &mut App);
}

pub trait KeyEventHandler<T = ()> {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, event_state: &State) -> T;
}

pub trait ScreenKeybinds {
    /// Returns a list of keybinds to be shown as the bottom title of the screen
    fn screen_keybinds<'a>(&self) -> Vec<(&'a str, &'a str)>;
}

/// Creates the title that shows the available keybinds for a screen
pub fn pane_title_bottom<'a>(
    colors: &ColorsConfig,
    hints: Vec<(&'a str, &'a str)>,
    focused: bool,
) -> Line<'a> {
    let separator = "──";
    let hints_line = hints
        .iter()
        .flat_map(|h| {
            vec![
                Span::from(format!("{} ", separator)).fg(if focused {
                    colors.primary
                } else {
                    colors.border
                }),
                Span::from(h.0).bold().fg(colors.keybind_key),
                Span::from(" ➜ ").fg(colors.secondary),
                Span::from(format!("{} ", h.1)).fg(colors.keybind_fg),
            ]
        })
        .collect::<Vec<Span>>();
    Line::from([hints_line, vec![Span::from(separator).fg(colors.border)]].concat())
}
