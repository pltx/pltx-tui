use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Size},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
    Frame,
};
use tui_scrollview::ScrollView;

use crate::{
    components,
    state::{Mode, Popup, State},
    utils::{centered_rect_absolute, KeyEventHandler, RenderPopup},
    App,
};

pub struct Help {
    pub width: u16,
    pub height: u16,
}

impl Help {
    pub fn init() -> Help {
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

impl RenderPopup for Help {
    fn render(&mut self, frame: &mut Frame, app: &mut App) {
        let popup = components::Popup::new(app);

        let area = centered_rect_absolute(self.width, self.height, frame.size());
        frame.render_widget(Clear, area);
        frame.render_widget(popup.block, area);

        let [sub_area] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints([Constraint::Fill(1)])
            .areas(area);

        // TODO: Fix height being twice as much as it needs to be
        let mut scroll_view = ScrollView::new(Size::new(area.width, self.total_height()));
        self.render_widgets_into_scrollview(scroll_view.buf_mut(), app);
        scroll_view.render(sub_area, frame.buffer_mut(), &mut app.scroll_view_state);
    }

    fn render_widgets_into_scrollview(&self, buf: &mut Buffer, app: &App) {
        let area = buf.area;

        let [content, spacing] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(4)])
            .areas(area);

        Block::new().render(spacing, buf);

        let [navigation_layout, popup_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(3)
            .constraints([
                Constraint::Length((self.get_keybinds(Mode::Navigation).len() + 2) as u16),
                Constraint::Length((self.get_keybinds(Mode::Popup).len() + 2) as u16),
            ])
            .areas(content);

        self.keybinds_paragraph(app, Mode::Navigation)
            .render(navigation_layout, buf);
        self.keybinds_paragraph(app, Mode::Popup)
            .render(popup_layout, buf);
    }
}

impl Help {
    /// Used to get the list of keybinds for each mode. `.len()` is also used to
    /// calculate the vertical space needed for it in the layout exactly.
    fn get_keybinds<'a>(&self, mode: Mode) -> Vec<(&'a str, &'a str)> {
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
        }
    }

    fn total_height(&self) -> u16 {
        ((self.get_keybinds(Mode::Navigation).len() + 4) as u16)
            + ((self.get_keybinds(Mode::Navigation).len() + 4) as u16)
    }

    fn keybinds_paragraph(&self, app: &App, mode: Mode) -> impl Widget {
        let colors = &app.config.colors;
        let text = Text::from(
            [
                vec![Line::from(vec![
                    Span::from(format!(" {} Mode ", app.get_mode_text(mode.clone()))).style(
                        Style::new()
                            .bold()
                            .fg(match mode {
                                Mode::Navigation => colors.status_bar_navigation_mode_fg,
                                Mode::Popup => colors.status_bar_popup_mode_fg,
                            })
                            .bg(match mode {
                                Mode::Navigation => colors.status_bar_navigation_mode_bg,
                                Mode::Popup => colors.status_bar_popup_mode_bg,
                            }),
                    ),
                    Span::from("").style(Style::new().fg(match mode {
                        Mode::Navigation => colors.status_bar_navigation_mode_bg,
                        Mode::Popup => colors.status_bar_popup_mode_bg,
                    })),
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
