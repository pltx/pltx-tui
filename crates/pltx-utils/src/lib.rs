mod centered_rect;
mod render;
mod widgets;

pub use centered_rect::*;
use chrono::{DateTime, Local, ParseError, Utc};
use pltx_app::state::Mode;
pub use render::*;
pub use widgets::*;

pub fn get_version<'a>() -> &'a str {
    env!("CARGO_PKG_VERSION")
}

/// Get current timestamp to be used for data.
pub fn current_timestamp() -> String {
    Utc::now().to_rfc3339().to_string()
}

/// Get the current local timestamp. Not to be used as data.
pub fn display_current_timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

/// Convert and display a UTC timestamp to local format. Not to be used as data.
pub fn display_timestamp(datetime: DateTime<Utc>) -> String {
    DateTime::<Local>::from(datetime)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

/// Check if the current datetime is after a datetime.
/// ```
/// use chrono::{DateTime, Utc};
/// use pltx_utils::after_datetime;
///
/// let datetime = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap().to_utc();
/// assert!(after_datetime(datetime));
/// ```
pub fn after_datetime(datetime: DateTime<Utc>) -> bool {
    Utc::now().timestamp() > datetime.timestamp()
}

/// Parse a datetime in the format "%Y-%m-%d %H:%M" and return in rfc3999 format
/// "%Y-%m-%dT%H:%M:%S+%z" as a [`Result`](Result).
/// ```
/// use pltx_utils::parse_user_datetime;
///
/// let parsed_datetime = parse_user_datetime(String::from("2000-01-01 00:00"));
/// assert_eq!(parsed_datetime.unwrap(), String::from("2000-01-01T00:00:00+00:00"));
/// ```
pub fn parse_user_datetime(user_datetime: String) -> Result<String, ParseError> {
    let parsed_string_format = format!("{}:00+00:00", user_datetime.replacen(' ', "T", 1));
    match DateTime::parse_from_rfc3339(&parsed_string_format) {
        Ok(_) => Ok(parsed_string_format),
        Err(err) => Err(err),
    }
}

/// Parse a datetime in the format "%Y-%m-%d %H:%M" and return in rfc3999 format
/// "%Y-%m-%dT%H:%M:%S+%z" as a [`Option`](Option).
/// ```
/// use pltx_utils::parse_user_datetime_option;
///
/// let parsed_datetime = parse_user_datetime_option(String::from("2000-01-01 00:00"));
/// assert_eq!(parsed_datetime.unwrap(), String::from("2000-01-01T00:00:00+00:00"));
/// ```
pub fn parse_user_datetime_option(user_datetime: String) -> Option<String> {
    if user_datetime.chars().count() != 16 {
        None
    } else {
        match parse_user_datetime(user_datetime) {
            Ok(parsed_string_format) => Some(parsed_string_format),
            Err(_) => None,
        }
    }
}

pub fn db_datetime(db_datetime: Option<String>) -> Option<String> {
    db_datetime.map(|datetime| {
        DateTime::parse_from_rfc3339(&datetime)
            .unwrap()
            .format("%Y-%m-%d %H:%M")
            .to_string()
    })
}

pub fn normal_to_insert(mode: Mode) -> Mode {
    match mode {
        Mode::Popup => Mode::PopupInsert,
        Mode::Command => Mode::CommandInsert,
        _ => Mode::Insert,
    }
}
