mod centered_rect;
mod render;

pub use centered_rect::*;
pub use render::*;

use crate::state::Mode;

pub fn get_version<'a>() -> &'a str {
    env!("CARGO_PKG_VERSION")
}

pub fn current_timestamp() -> String {
    chrono::offset::Local::now()
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

pub fn normal_to_insert(mode: Mode) -> Mode {
    match mode {
        Mode::Popup => Mode::PopupInsert,
        Mode::Command => Mode::CommandInsert,
        _ => Mode::Insert,
    }
}
