use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{block::Title, Block, Cell, Clear, Row, Table},
    Frame,
};

use crate::{
    components,
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
            width: 60,
            height: 15,
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
    fn render(self, frame: &mut Frame, app: &App) {
        let colors = &app.config.colors;
        let popup = components::Popup::new(app);

        let area = centered_rect_absolute(self.width, self.height, frame.size());
        frame.render_widget(Clear, area);
        frame.render_widget(popup.block, area);

        let popup_layout = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(3)
            .constraints([Constraint::Min(1)])
            .split(area);

        let rows = [
            Row::new(vec![Cell::new("?").bold(), Cell::new("Show the help menu")]),
            Row::new(vec!["q", "Quit the application"]),
            Row::new(vec!["h", "Focus on the previous pane"]),
            Row::new(vec!["j", "Navigate down to the next option"]),
            Row::new(vec!["k", "Navigate up to the previous option"]),
            Row::new(vec!["l", "Focus on the next pane"]),
        ];
        let widths = [Constraint::Length(10), Constraint::Min(1)];
        let table = Table::new(rows, widths)
            .column_spacing(1)
            .style(Style::new().fg(colors.secondary))
            .header(
                Row::new(vec!["Keybind", "Description"]).style(Style::new().bold().fg(colors.fg)),
            )
            .block(
                Block::new()
                    .title(Title::from("Navigation Mode"))
                    .title_style(Style::new().bold().fg(colors.status_bar_navigation_mode_bg)),
            )
            .highlight_style(Style::new().reversed());

        frame.render_widget(table, popup_layout[0])
    }
}
