use ratatui::layout::Rect;

pub fn centered_rect(
    (width, percentage_based_width): (u16, bool),
    (height, percentage_based_height): (u16, bool),
    rect: Rect,
) -> Rect {
    let absolute_width = if percentage_based_width {
        ((width as f32 * 0.01) * rect.width as f32).floor() as u16
    } else {
        width
    };
    let absolute_height = if percentage_based_height {
        ((height as f32 * 0.01) * rect.height as f32).floor() as u16
    } else {
        height
    };
    let side_width = (rect.width.saturating_sub(absolute_width)) / 2;
    let side_height = (rect.height.saturating_sub(absolute_height)) / 2;

    Rect::new(
        if percentage_based_width {
            side_width + rect.x
        } else {
            side_width
        },
        if percentage_based_height {
            side_height + rect.y
        } else {
            side_height
        },
        absolute_width,
        absolute_height,
    )
}
