use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
    Frame,
};
use tui_scrollview::ScrollView;

use crate::{
    components,
    config::ColorsConfig,
    state::{Mode, Popup, State},
    utils::{Init, KeyEventHandler, RenderScrollPopup},
    App,
};

pub struct Help {
    pub width: u16,
    pub height: u16,
}

impl Init for Help {
    fn init(_: &mut App) -> Help {
        Help {
            width: 70,
            height: 20,
        }
    }
}

impl KeyEventHandler for Help {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) {
        if key_event.code == KeyCode::Char('?') {
            app.state.mode = Mode::Navigation;
            app.state.popup = Popup::None;
        }
    }
}

impl RenderScrollPopup for Help {
    fn render(&mut self, frame: &mut Frame, app: &mut App) {
        let popup = components::Popup::new(app, frame.size())
            .title_top("Help Menu")
            .render(frame);
        // TODO: Fix height being twice as much as it needs to be
        let mut scroll_view = ScrollView::new(Size::new(popup.area.width, self.total_height()));
        self.render_widgets_into_scrollview(scroll_view.buf_mut(), app);
        scroll_view.render(
            popup.sub_area,
            frame.buffer_mut(),
            &mut app.scroll_view_state,
        );
    }

    fn render_widgets_into_scrollview(&mut self, buf: &mut Buffer, app: &App) {
        let area = buf.area;

        let [content, spacing] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(4)])
            .areas(area);

        Block::new().render(spacing, buf);

        let layouts = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(3)
            .constraints(
                self.get_modes()
                    .map(|m| Constraint::Length((self.get_keybinds(&m).len() + 2) as u16)),
            )
            .split(content);

        for (i, mode) in self.get_modes().iter().enumerate() {
            self.keybinds_paragraph(app, mode).render(layouts[i], buf);
        }
    }
}

impl Help {
    /// Used to get the list of keybinds for each mode. `.len()` is also used to
    /// calculate the vertical space needed for it in the layout exactly.
    fn get_keybinds<'a>(&self, mode: &Mode) -> Vec<(&'a str, &'a str)> {
        match mode {
            Mode::Navigation => vec![
                ("?", "Show help menu"),
                ("q", "Quit application"),
                ("h", "Select next horizontal option"),
                ("j", "Select next vertical option"),
                ("k", "Select previous vertical option"),
                ("l", "Select previous horizontal option"),
                ("<enter>", "Open selected option + Next navigation pane"),
                ("<bs>", "Previous navigation pane"),
            ],
            Mode::Insert => vec![("<esc>", "Exit insert mode")],
            Mode::Popup => vec![
                ("q", "Close popup"),
                ("?", "Close help menu (if open)"),
                ("l", "Next page"),
                ("h", "Previous page"),
                ("k", "Scroll up"),
                ("j", "Scroll down"),
                ("J", "Page up"),
                ("K", "Page down"),
                ("g", "Scroll top"),
                ("G", "Scroll bottom"),
            ],
            Mode::PopupInsert => vec![("<esc>", "Exit popup insert mode")],
            Mode::Delete => vec![("y", "Yes (delete)"), ("n", "No (cancel)")],
        }
    }

    fn get_modes(&self) -> [Mode; 5] {
        [
            Mode::Navigation,
            Mode::Insert,
            Mode::Popup,
            Mode::PopupInsert,
            Mode::Delete,
        ]
    }

    fn total_height(&self) -> u16 {
        self.get_modes()
            .iter()
            .map(|m| (self.get_keybinds(m).len() + 4) as u16)
            .sum()
    }

    /// Returns (fg, bg).
    /// TODO: There is another function similar to this
    /// (`App::get_mode_colors()`). Merge them into one and refactor
    /// accordingly.
    fn get_mode_color(&mut self, colors: &ColorsConfig, mode: &Mode) -> (Color, Color) {
        (
            match mode {
                Mode::Navigation => colors.status_bar_navigation_mode_fg,
                Mode::Insert => colors.status_bar_insert_mode_fg,
                Mode::Popup => colors.status_bar_popup_mode_fg,
                Mode::PopupInsert => colors.status_bar_popup_insert_mode_fg,
                Mode::Delete => colors.status_bar_delete_mode_fg,
            },
            match mode {
                Mode::Navigation => colors.status_bar_navigation_mode_bg,
                Mode::Insert => colors.status_bar_insert_mode_bg,
                Mode::Popup => colors.status_bar_popup_mode_bg,
                Mode::PopupInsert => colors.status_bar_popup_insert_mode_bg,
                Mode::Delete => colors.status_bar_delete_mode_bg,
            },
        )
    }

    fn keybinds_paragraph(&mut self, app: &App, mode: &Mode) -> impl Widget {
        let colors = &app.config.colors;
        let text = Text::from(
            [
                vec![Line::from(vec![
                    Span::from(format!(" {} Mode ", app.get_mode_text(*mode))).style(
                        Style::new()
                            .bold()
                            .fg(self.get_mode_color(colors, mode).0)
                            .bg(self.get_mode_color(colors, mode).1),
                    ),
                    Span::from("").style(Style::new().fg(self.get_mode_color(colors, mode).1)),
                ])],
                self.get_keybinds(mode)
                    .iter()
                    .map(|k| {
                        Line::from(vec![
                            Span::from(k.0).bold().fg(colors.keybind_key),
                            Span::from(" ➜ ").fg(colors.secondary),
                            Span::from(k.1).fg(colors.keybind_fg),
                        ])
                    })
                    .collect::<Vec<Line>>(),
            ]
            .concat(),
        );
        Paragraph::new(text)
    }
}
