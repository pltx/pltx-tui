use std::fmt;

use chrono::{DateTime as ChronoDateTime, Duration, Local, Utc};

/// Custom struct around [`Chrono`](chrono) for managing datetime within the
/// application. Provides convenience methods to reduce the need for repetitive
/// code.
#[derive(Clone)]
pub struct DateTime {
    pub datetime: ChronoDateTime<Utc>,
}

pub struct DurationSince {
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

impl fmt::Display for DurationSince {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hour_prefix = if self.hours < 10 { "0" } else { "" };
        let min_prefix = if self.minutes < 10 { "0" } else { "" };
        let sec_prefix = if self.seconds < 10 { "0" } else { "" };

        if self.hours > 0 {
            write!(
                f,
                "{}{}:{}{}:{}{}",
                hour_prefix, self.hours, min_prefix, self.minutes, sec_prefix, self.seconds
            )
        } else if self.minutes != 0 {
            write!(
                f,
                "00:{}{}:{}{}",
                min_prefix, self.minutes, sec_prefix, self.seconds
            )
        } else {
            write!(f, "00:00:{}{}", sec_prefix, self.seconds)
        }
    }
}

impl From<ChronoDateTime<Utc>> for DateTime {
    fn from(datetime: ChronoDateTime<Utc>) -> Self {
        Self { datetime }
    }
}

impl From<&str> for DateTime {
    fn from(value: &str) -> Self {
        let datetime = ChronoDateTime::parse_from_rfc3339(value)
            .expect("failed to parse datetime")
            .to_utc();
        Self::from(datetime)
    }
}

impl From<String> for DateTime {
    fn from(value: String) -> Self {
        let datetime = ChronoDateTime::parse_from_rfc3339(&value)
            .expect("failed to parse datetime")
            .to_utc();
        Self::from(datetime)
    }
}

impl DateTime {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            datetime: Utc::now(),
        }
    }

    /// Parse a datetime in the format "%Y-%m-%d %H:%M" and return in rfc3999.
    /// format "%Y-%m-%dT%H:%M:%S+%z" as a [`Option`](Option).
    /// ```
    /// # use pltx_utils::DateTime;
    /// let datetime = DateTime::from_input(String::from("2000-01-01 00:00"));
    /// assert_eq!(datetime, Some(String::from("2000-01-01T00:00:00+00:00")));
    /// ```
    pub fn from_input(input: String) -> Option<String> {
        let parsed_string_format = format!("{}:00+00:00", input.replacen(' ', "T", 1));
        let input_result = match ChronoDateTime::parse_from_rfc3339(&parsed_string_format) {
            Ok(datetime) => Ok(Self::from(datetime.to_utc())),
            Err(err) => Err(err),
        };

        match input_result {
            Ok(result) => Some(result.datetime.to_rfc3339()),
            Err(_) => None,
        }
    }

    /// Get the current local datetime.
    pub fn display_now() -> String {
        Local::now().format("%Y-%m-%d %H:%M").to_string()
    }

    /// Get the current local datetime with seconds.
    pub fn display_now_with_seconds() -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Convert and display the datetime in local format.
    pub fn display(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    }

    /// Convert and display the datetime in local format with seconds.
    pub fn display_with_seconds(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    /// Convert and display the datetime to just the date in local format.
    pub fn display_date(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%Y-%m-%d")
            .to_string()
    }

    /// Convert and display the datetime to just the time in local format.
    pub fn display_time(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%H:%M:%S")
            .to_string()
    }

    /// Convert and display the timedate to just the time with seconds in local
    /// format.
    pub fn display_time_with_seconds(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%H:%M:%S")
            .to_string()
    }

    /// Checks if the current datetime is past a datetime.
    /// ```
    /// # use pltx_utils::DateTime;
    /// let datetime = DateTime::from("2000-01-01T00:00:00+00:00");
    /// assert!(datetime.is_past());
    /// ```
    pub fn is_past(&self) -> bool {
        Utc::now().timestamp() > self.datetime.timestamp()
    }

    /// Checks if the current datetime is past a datetime minus a specified
    /// number of days, which means the number of days *before* the
    /// datetime.
    /// ```
    /// # use pltx_utils::DateTime;
    /// let datetime = DateTime::from("2000-01-01T00:00:00+00:00");
    /// assert!(datetime.is_past_days(100));
    /// ```
    pub fn is_past_days(&self, days: i32) -> bool {
        Utc::now().timestamp() > (self.datetime - Duration::days(days as i64)).timestamp()
    }

    /// Calculates the duration since a past date.
    /// ```
    /// # use pltx_utils::DateTime;
    /// let later_datetime= DateTime::from("2024-05-25T23:56:30+00:00");
    /// let past_datetime  = DateTime::from("2024-05-25T20:59:54+00:00");
    /// let duration = later_datetime.duration_since(&past_datetime);
    /// assert_eq!(duration.hours, 2);
    /// assert_eq!(duration.minutes, 56);
    /// assert_eq!(duration.seconds, 36);
    /// assert_eq!(duration.to_string(), String::from("02:56:36"));
    /// ```
    pub fn duration_since(&self, past: &DateTime) -> DurationSince {
        let diff = self.datetime.time() - past.datetime.time();

        let hours_overlap = diff.num_days() * 24;
        let minutes_overlap = (diff.num_days() * 24 * 60) + (diff.num_hours() * 60);
        let seconds_overlap = (diff.num_days() * 24 * 60 * 60) + (diff.num_minutes() * 60);

        DurationSince {
            hours: diff.num_hours() - hours_overlap,
            minutes: diff.num_minutes() - minutes_overlap,
            seconds: diff.num_seconds() - seconds_overlap,
        }
    }
}

// Idiomatic methods for working with the database.
impl DateTime {
    /// Get the current datetime in rfc3999 format.
    pub fn now() -> String {
        Utc::now().to_rfc3339()
    }

    /// Create an instance from an database DATETIME field that doesn't have
    /// DEFAULT or NOT NULL.
    pub fn from_db_option(db_option: Option<String>) -> Option<Self> {
        db_option.map(Self::from)
    }

    /// Create an instance from a NOT NULL database field.
    pub fn from_db(db_datetime: Option<String>) -> Self {
        let db_datetime = db_datetime
            .expect("no datetime was provided when this function expects Some(db_datetime)");
        Self::from(
            ChronoDateTime::parse_from_rfc3339(&db_datetime)
                .expect("failed to parse db datetime")
                .to_utc(),
        )
    }

    /// Converts the current datetime to rfc3999 format for storing in the
    /// database. ```
    /// # use pltx_utils::DateTime;
    /// # use chrono::DateTime as ChronoDateTime;
    /// let datetime = DateTime::from("2000-01-01T00:00:00+00:00");
    /// assert!(ChronoDateTime::parse_from_rfc3339(&datetime.datetime.
    /// to_rfc3339()).is_ok()); ```
    pub fn into_db(&self) -> String {
        self.datetime.to_rfc3339()
    }
}
