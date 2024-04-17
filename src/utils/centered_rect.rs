use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create a centered rect using a percentage of the available rect.
pub fn centered_rect(width_percentage: u16, height_percentage: u16, r: Rect) -> Rect {
    // Cut the given rectange into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percentage) / 2),
            Constraint::Percentage(height_percentage),
            Constraint::Percentage(100 - height_percentage / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pices
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percentage) / 2),
            Constraint::Percentage(width_percentage),
            Constraint::Percentage((100 - width_percentage) / 2),
        ])
        .split(popup_layout[1])[1] // return the middle chunk
}

/// Create a centered rect with an absolute size.
pub fn centered_rect_absolute(absolute_width: u16, absolute_height: u16, r: Rect) -> Rect {
    let width = (r.width.saturating_sub(absolute_width)) / 2;
    let height = (r.height.saturating_sub(absolute_height)) / 2;

    Rect::new(
        width,
        height,
        absolute_width.min(r.width),
        absolute_height.min(r.height),
    )
}
