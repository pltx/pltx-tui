use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    widgets::{block::Title, Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::{
    utils::{centered_rect, RenderPopup},
    App,
};

pub struct Help {
    pub width: u16,
    pub height: u16,
}

impl Help {
    pub fn init() -> Help {
        Help {
            width: 40,
            height: 30,
        }
    }
}

impl RenderPopup for Help {
    fn render(self, frame: &mut Frame, app: &App) {
        let colors = &app.config.colors;

        let popup_block = Block::default()
            .title(Title::from(" Help Menu ").alignment(Alignment::Center))
            .title_style(Style::new().fg(colors.fg))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(colors.popup_border))
            .bg(colors.popup_bg);

        let popup_area = centered_rect(self.width, self.height, frame.size());
        frame.render_widget(popup_block, popup_area);

        let popup_layout = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(3)
            .constraints([Constraint::Min(1)])
            .split(popup_area);

        let rows = [
            Row::new(vec![Cell::new("?").bold(), Cell::new("Show the help menu")]),
            Row::new(vec!["q", "Quit the application"]),
            Row::new(vec!["h", "Focus on the previous window"]),
            Row::new(vec!["j", "Navigate down to the next option"]),
            Row::new(vec!["k", "Navigate up to the previous option"]),
            Row::new(vec!["l", "Focus on the next window"]),
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
