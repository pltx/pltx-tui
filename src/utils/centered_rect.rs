use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn centered_rect(width_percentage: u16, height_percentage: u16, rect: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percentage) / 2),
            Constraint::Percentage(height_percentage),
            Constraint::Percentage((100 - height_percentage) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percentage) / 2),
            Constraint::Percentage(width_percentage),
            Constraint::Percentage((100 - width_percentage) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn centered_rect_absolute(absolute_width: u16, absolute_height: u16, rect: Rect) -> Rect {
    let width = (rect.width.saturating_sub(absolute_width)) / 2;
    let height = (rect.height.saturating_sub(absolute_height)) / 2;

    Rect::new(
        width,
        height,
        absolute_width.min(rect.width),
        absolute_height.min(rect.height),
    )
}
