//! The date/time port.
//!
//! SQLite has no date type. The frozen schema says the datetime columns hold
//! **ISO-8601 UTC** (`docs/contract/schema.sql`: `start_time DATETIME NOT
//! NULL -- ISO-8601 UTC`), so that is what is written:
//!
//! ```text
//! 2026-05-12T06:30:00.000000Z
//! ```
//!
//! Six digits of microseconds, always, and a `Z` — one shape, one parser, no
//! ambiguity about the offset. The reader is more generous than the writer: it
//! also accepts the SQLAlchemy storage format the old implementation used
//! (`2026-05-12 06:30:00.123456`), so a database written by the old desktop
//! can be read.
//!
//! The athlete's **local** date is a separate thing and is stored as the
//! string `YYYY-MM-DD` in its own column — the local date is not a UTC
//! instant, and turning it into one would put the athlete's midnight in the
//! wrong place.

use chrono::{DateTime, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

use crate::error::{Result, StoreError};

/// The text format every datetime is written in.
const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6fZ";

/// A UTC instant, stored as ISO-8601.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbDateTime(pub DateTime<Utc>);

impl DbDateTime {
    /// The text SQLite holds for this instant.
    pub fn to_text(self) -> String {
        self.0.format(FORMAT).to_string()
    }

    /// Parse the text SQLite holds for this instant.
    pub fn parse_text(value: &str) -> std::result::Result<Self, String> {
        parse_datetime_text(value).map(DbDateTime)
    }
}

impl From<DateTime<Utc>> for DbDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        DbDateTime(value)
    }
}

impl From<DbDateTime> for DateTime<Utc> {
    fn from(value: DbDateTime) -> Self {
        value.0
    }
}

impl std::fmt::Display for DbDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

/// Parse a datetime out of the text a database holds.
///
/// ISO-8601 with a `Z` or an offset is the format this crate writes. The
/// SQLAlchemy storage format and the plain `YYYY-MM-DD HH:MM:SS` are accepted
/// too, because the old implementation wrote those and an imported database
/// has to be readable.
pub fn parse_datetime_text(value: &str) -> std::result::Result<DateTime<Utc>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("the value is empty".to_string());
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(parsed.with_timezone(&Utc));
    }

    for format in [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Ok(Utc.from_utc_datetime(&naive));
        }
    }

    Err(format!("{trimmed:?} is not a datetime"))
}

/// Parse a datetime out of the text a database holds.
pub fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    parse_datetime_text(value).map_err(|reason| StoreError::InvalidKey {
        reason: format!("the datetime {reason}"),
    })
}

/// The ISO-8601 form, with the offset written out.
pub fn to_iso8601(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, true)
}

/// The athlete's local date, `YYYY-MM-DD`.
pub fn local_date(value: DateTime<Utc>, offset_seconds: i32) -> String {
    match chrono::FixedOffset::east_opt(offset_seconds) {
        Some(offset) => value.with_timezone(&offset).format("%Y-%m-%d").to_string(),
        // An offset outside -24h..24h is not a real one; UTC is the fallback
        // and the caller gets a date rather than an error.
        None => value.format("%Y-%m-%d").to_string(),
    }
}

impl ToSql for DbDateTime {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Text(self.to_text())))
    }
}

impl FromSql for DbDateTime {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = match value {
            ValueRef::Text(text) => std::str::from_utf8(text)
                .map_err(|error| rusqlite::types::FromSqlError::Other(Box::new(error)))?
                .to_string(),
            _ => return Err(rusqlite::types::FromSqlError::InvalidType),
        };
        DbDateTime::parse_text(&text).map_err(|value| {
            rusqlite::types::FromSqlError::Other(Box::new(StoreError::InvalidKey {
                reason: format!("the datetime {value}"),
            }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_datetime_round_trips() -> Result<()> {
        let instant = Utc.with_ymd_and_hms(2026, 5, 12, 6, 30, 15).unwrap();
        let text = DbDateTime(instant).to_text();
        assert_eq!(text, "2026-05-12T06:30:15.000000Z");
        let parsed = parse_datetime(&text)?;
        assert_eq!(parsed, instant);
        Ok(())
    }

    #[test]
    fn the_sqlalchemy_format_is_readable() -> Result<()> {
        let parsed = parse_datetime("2024-05-01 10:20:30.123456")?;
        assert_eq!(DbDateTime(parsed).to_text(), "2024-05-01T10:20:30.123456Z");
        Ok(())
    }

    #[test]
    fn a_nonsense_value_is_an_error() {
        assert!(DbDateTime::parse_text("not a datetime").is_err());
        assert!(DbDateTime::parse_text("").is_err());
    }
}
