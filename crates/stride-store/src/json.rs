//! The JSON port.
//!
//! The Python schema has `JSON` columns. SQLite has no JSON type either, and
//! SQLAlchemy stores a JSON document as **text** produced by `json.dumps`.
//! We store `serde_json::Value` as text the same way; the difference between
//! Python's `json.dumps` spacing (`[1, 2]`) and ours (`[1,2]`) is whitespace
//! inside a JSON document and carries no meaning — both parse to the same
//! value, and the phone only ever reads these back through a JSON parser.

use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value as SqlValue, ValueRef,
};
use serde_json::Value;

/// A JSON document, stored as text.
#[derive(Debug, Clone, PartialEq)]
pub struct DbJson(pub Value);

impl DbJson {
    /// The text SQLite holds for this document.
    pub fn to_text(&self) -> String {
        self.0.to_string()
    }

    /// Parse a document out of the text SQLite holds.
    pub fn parse_text(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value).map(DbJson)
    }
}

impl From<Value> for DbJson {
    fn from(value: Value) -> Self {
        DbJson(value)
    }
}

impl From<DbJson> for Value {
    fn from(value: DbJson) -> Self {
        value.0
    }
}

impl ToSql for DbJson {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(SqlValue::Text(self.to_text())))
    }
}

impl FromSql for DbJson {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = match value {
            ValueRef::Text(text) => std::str::from_utf8(text)
                .map_err(|err| FromSqlError::Other(Box::new(err)))?
                .to_string(),
            _ => return Err(FromSqlError::InvalidType),
        };
        DbJson::parse_text(&text).map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}
