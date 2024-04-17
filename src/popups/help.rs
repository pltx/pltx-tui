use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Size},
    style::{Style, Stylize},
    widgets::{block::Title, Block, Cell, Clear, Row, StatefulWidget, Table, Widget},
    Frame,
};
use tui_scrollview::ScrollView;

use crate::{
    components,
    config::ColorsConfig,
    state::{Mode, Popup, State},
    utils::{centered_rect_absolute, PopupKeyEventHandler, RenderPopup},
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

impl PopupKeyEventHandler for Help {
    fn key_event_handler(app: &mut App, key_event: KeyEvent, _: &State) {
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
        let colors = &app.config.colors;

        let [content, spacing] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(4)])
            .areas(area);

        Block::new().render(spacing, buf);

        let [navigation_layout, popup_layout] = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(3)
            .constraints([
                Constraint::Length((self.get_keybinds(Mode::Navigation).len() + 3) as u16),
                Constraint::Length((self.get_keybinds(Mode::Popup).len() + 3) as u16),
            ])
            .areas(content);

        self.navigation_table(colors).render(navigation_layout, buf);
        self.popup_table(colors).render(popup_layout, buf);
    }
}

impl Help {
    /// Used to get the list of keybinds for each mode. `.len()` is also used to
    /// calculate the vertical space needed for it in the layout exactly.
    fn get_keybinds<'a>(&self, mode: Mode) -> Vec<(&'a str, &'a str)> {
        match mode {
            Mode::Navigation => vec![
                ("?", "Show the help menu"),
                ("q", "Quit the application"),
                ("h", "Focus on the previous pane"),
                ("j", "Navigate down to the next option"),
                ("k", "Navigate up to the previous option"),
                ("l", "Focus on the next pane"),
            ],
            Mode::Popup => vec![
                ("q", "Close the popup"),
                ("?", "Close the help menu (if open)"),
                ("h", "Navigate to the previous page"),
                ("j", "Scroll down a line"),
                ("k", "Scroll up a line"),
                ("J", "Scroll down a page"),
                ("K", "Scroll up a page"),
                ("g", "scroll to the top of the page"),
                ("G", "Scroll to the bottom of the page"),
                ("l", "Navigate to the next page"),
            ],
        }
    }

    fn total_height(&self) -> u16 {
        ((self.get_keybinds(Mode::Navigation).len() + 4) as u16)
            + ((self.get_keybinds(Mode::Navigation).len() + 4) as u16)
    }

    fn navigation_table(&self, colors: &ColorsConfig) -> impl Widget {
        let navigation_rows = self
            .get_keybinds(Mode::Navigation)
            .iter()
            .map(|k| Row::new(vec![Cell::new(k.0).bold(), Cell::new(k.1)]))
            .collect::<Vec<Row>>();
        let navigation_widths = [Constraint::Length(12), Constraint::Min(1)];
        Table::new(navigation_rows, navigation_widths)
            .column_spacing(1)
            .style(Style::new().fg(colors.secondary))
            .header(Row::new(vec!["Keybind", "Description"]).style(Style::new().fg(colors.fg)))
            .block(
                Block::new()
                    .title(Title::from("Navigation Mode"))
                    .title_style(Style::new().bold().fg(colors.status_bar_navigation_mode_bg)),
            )
            .highlight_style(Style::new().reversed())
    }

    fn popup_table(&self, colors: &ColorsConfig) -> impl Widget {
        let popup_rows = self
            .get_keybinds(Mode::Popup)
            .iter()
            .map(|k| Row::new(vec![Cell::new(k.0).bold(), Cell::new(k.1)]))
            .collect::<Vec<Row>>();
        let popup_widths = [Constraint::Length(12), Constraint::Min(1)];
        Table::new(popup_rows, popup_widths)
            .column_spacing(1)
            .style(Style::new().fg(colors.secondary))
            .header(Row::new(vec!["Keybind", "Description"]).style(Style::new().fg(colors.fg)))
            .block(
                Block::new()
                    .title(Title::from("Popup Mode"))
                    .title_style(Style::new().bold().fg(colors.status_bar_popup_mode_bg)),
            )
            .highlight_style(Style::new().reversed())
    }
}
