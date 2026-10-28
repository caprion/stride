//! The date/time port.
//!
//! The Python schema has `DateTime(timezone=True)` columns. SQLite has no
//! date type, and SQLAlchemy's SQLite dialect stores them as **text** in this
//! exact format:
//!
//! ```text
//! YYYY-MM-DD HH:MM:SS.ffffff
//! ```
//!
//! Six digits of microseconds, always — `2024-05-01 10:20:30.123456`. A
//! timezone-aware Python `datetime` is stored the same way: the offset is
//! dropped and the wall-clock UTC value is written. That is what the reference
//! implementation does, so this is what we do; a database written by either
//! implementation opens in the other.
//!
//! In Rust the type is `chrono::DateTime<Utc>`. `DbDateTime` is the thin
//! wrapper that carries it across the SQL boundary.

use chrono::{DateTime, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

use crate::error::{Result, StoreError};

/// A UTC instant, stored in the Python/SQLAlchemy text format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbDateTime(pub DateTime<Utc>);

impl DbDateTime {
    /// The text SQLite holds for this instant.
    pub fn to_text(self) -> String {
        self.0.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
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

/// Parse a datetime out of the text SQLAlchemy (or we) wrote.
fn parse_datetime_text(value: &str) -> std::result::Result<DateTime<Utc>, String> {
    let trimmed = value.trim();

    // ISO-8601 with an offset or a `Z`: `2024-05-01T10:20:30.123456Z`.
    if trimmed.contains('T') || trimmed.ends_with('Z') || trimmed.ends_with('z') {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
            return Ok(parsed.with_timezone(&Utc));
        }
    }

    // The SQLAlchemy storage format: `2024-05-01 10:20:30.123456`.
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

    Err(value.to_string())
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
                .map_err(|err| FromSqlError::Other(Box::new(err)))?
                .to_string(),
            _ => return Err(FromSqlError::InvalidType),
        };
        DbDateTime::parse_text(&text).map_err(|value| {
            FromSqlError::Other(Box::new(StoreError::DateTime {
                column: "datetime".to_string(),
                value,
            }))
        })
    }
}

/// Format an instant the way the Python implementation stores it.
pub fn format_datetime(value: DateTime<Utc>) -> String {
    DbDateTime(value).to_text()
}

/// Parse an instant out of the Python storage format.
pub fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    parse_datetime_text(value).map_err(|reason| StoreError::DateTime {
        column: "datetime".to_string(),
        value: reason,
    })
}

/// ISO-8601 with a `Z` suffix, the way the API serialises a timestamp.
pub fn to_iso8601(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}
