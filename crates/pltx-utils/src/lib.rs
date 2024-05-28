//! Provides general purpose utilities for the application.
//! This crate should not depend on any of the other workspace crates.

use ratatui::layout::Rect;

mod datetime;
pub mod dirs;
pub mod symbols;
mod widget;

pub use datetime::DateTime;
pub use widget::*;

/// Center a rect by all sides within an area
pub fn centered_rect(
    (width, percentage_based_width): (u16, bool),
    (height, percentage_based_height): (u16, bool),
    area: Rect,
) -> Rect {
    let absolute_width = if percentage_based_width {
        ((width as f32 * 0.01) * area.width as f32).floor() as u16
    } else {
        width
    };
    let absolute_height = if percentage_based_height {
        ((height as f32 * 0.01) * area.height as f32).floor() as u16
    } else {
        height
    };
    let side_width = (area.width.saturating_sub(absolute_width)) / 2;
    let side_height = (area.height.saturating_sub(absolute_height)) / 2;

    Rect::new(
        side_width + area.x,
        side_height + area.y,
        absolute_width,
        absolute_height,
    )
}
