mod centered_rect;
mod render;

pub use centered_rect::*;
pub use render::*;

pub fn get_version<'a>() -> &'a str {
    env!("CARGO_PKG_VERSION")
}

pub fn current_timestamp() -> String {
    chrono::offset::Local::now()
        .format("%Y-%m-%d %H:%M")
        .to_string()
}
