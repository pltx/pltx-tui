use chrono::{DateTime as ChronoDateTime, Datelike, Duration, Local, ParseError, Timelike, Utc};

#[derive(Clone)]
pub struct DateTime {
    pub datetime: ChronoDateTime<Utc>,
}

pub struct DurationSince {
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl DateTime {
    fn from(datetime: ChronoDateTime<Utc>) -> Self {
        Self { datetime }
    }

    pub fn from_db_option(datetime: Option<String>) -> Option<Self> {
        datetime
            .map(|dt| ChronoDateTime::parse_from_rfc3339(&dt).unwrap().to_utc())
            .map(Self::from)
    }

    pub fn from_db(db_datetime: Option<String>) -> Self {
        Self::from(
            db_datetime
                .map(|datetime| {
                    ChronoDateTime::parse_from_rfc3339(&datetime)
                        .unwrap()
                        .to_utc()
                })
                .unwrap(),
        )
    }

    /// Parse a datetime in the format "%Y-%m-%d %H:%M" and return in rfc3999
    /// format "%Y-%m-%dT%H:%M:%S+%z" as a [`Result`](Result).
    /// ```
    /// use pltx_utils::parse_user_datetime;
    ///
    /// let parsed_datetime = parse_user_datetime(String::from("2000-01-01 00:00"));
    /// assert_eq!(parsed_datetime.unwrap(), String::from("2000-01-01T00:00:00+00:00"));
    /// ```
    pub fn from_input_result(input: String) -> Result<Self, ParseError> {
        let parsed_string_format = format!("{}:00+00:00", input.replacen(' ', "T", 1));
        match ChronoDateTime::parse_from_rfc3339(&parsed_string_format) {
            Ok(datetime) => Ok(Self::from(datetime.to_utc())),
            Err(err) => Err(err),
        }
    }

    /// Parse a datetime in the format "%Y-%m-%d %H:%M" and return in rfc3999
    /// format "%Y-%m-%dT%H:%M:%S+%z" as a [`Option`](Option).
    /// ```
    /// use pltx_utils::parse_user_datetime_option;
    ///
    /// let parsed_datetime = parse_user_datetime_option(String::from("2000-01-01 00:00"));
    /// assert_eq!(parsed_datetime.unwrap(), String::from("2000-01-01T00:00:00+00:00"));
    /// ```
    pub fn input_to_db(input: String) -> Option<String> {
        match Self::from_input_result(input) {
            Ok(result) => Some(result.datetime.to_rfc3339()),
            Err(_) => None,
        }
    }

    /// Get the current datetime. Used for data.
    pub fn now() -> String {
        Utc::now().to_rfc3339()
    }

    /// Get the current local datetime. Used to display datetime and
    /// datetime inputs. Not used as data.
    pub fn display_now() -> String {
        Local::now().format("%Y-%m-%d %H:%M").to_string()
    }

    /// Get the current local datetime with seconds. Used to display datetime
    /// and datetime inputs. Not used as data.
    pub fn display_now_sec() -> String {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Convert and display a UTC timestamp to local format. Used to display
    /// datetime and datetime inputs. Not used as data.
    pub fn display(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    }

    /// Convert and display a UTC timestamp to local format with seconds. Used
    /// to display datetime and datetime inputs. Not used as data.
    pub fn display_with_seconds(&self) -> String {
        ChronoDateTime::<Local>::from(self.datetime)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    /// Check if the current datetime is past a datetime.
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use pltx_utils::after_datetime;
    ///
    /// let datetime = ChronoDateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap().to_utc();
    /// assert!(after_datetime(datetime));
    /// ```
    pub fn is_past(&self) -> bool {
        Utc::now().timestamp() > self.datetime.timestamp()
    }

    pub fn is_past_days(&self, days: i64) -> bool {
        Utc::now().timestamp() > (self.datetime - Duration::days(days)).timestamp()
    }

    pub fn since(&self, past: &DateTime) -> DurationSince {
        DurationSince {
            days: self.datetime.day().saturating_sub(past.datetime.day()),
            hours: self.datetime.hour().saturating_sub(past.datetime.hour()),
            minutes: self
                .datetime
                .minute()
                .saturating_sub(past.datetime.minute()),
            seconds: self
                .datetime
                .second()
                .saturating_sub(past.datetime.second()),
        }
    }
}
