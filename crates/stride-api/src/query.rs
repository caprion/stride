//! The query string and the request body.
//!
//! FastAPI validates the query string before the handler runs, and a failure
//! is a 422 with a `detail` array. The router does the same, so a client sees
//! the same status and the same body shape whichever side answers.

use serde_json::{json, Value};

use crate::error::{validation_error_with, ApiError};

/// The parsed query string of a request.
#[derive(Debug, Clone, Default)]
pub struct Query {
    pairs: Vec<(String, String)>,
}

impl Query {
    /// Parse `a=1&b=2`. A leading `?` is tolerated, and so is an empty string.
    pub fn parse(raw: Option<&str>) -> Self {
        let mut pairs = Vec::new();
        let Some(raw) = raw else {
            return Self { pairs };
        };
        let raw = raw.strip_prefix('?').unwrap_or(raw);
        for chunk in raw.split('&') {
            if chunk.is_empty() {
                continue;
            }
            let (key, value) = match chunk.split_once('=') {
                Some((key, value)) => (key, value),
                None => (chunk, ""),
            };
            pairs.push((percent_decode(key), percent_decode(value)));
        }
        Self { pairs }
    }

    /// The first value of a key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    /// Every value of a key.
    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.pairs
            .iter()
            .filter(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    /// A required integer parameter, with FastAPI's bounds check.
    pub fn int(
        &self,
        key: &str,
        default: i64,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Result<i64, ApiError> {
        let Some(raw) = self.get(key) else {
            return Ok(default);
        };
        let parsed: i64 = raw.trim().parse().map_err(|_| {
            validation_error_with(
                vec!["query", key],
                "Input should be a valid integer, unable to parse string as an integer",
                "int_parsing",
                Value::String(raw.to_string()),
                Value::Null,
            )
        })?;
        if let Some(min) = min {
            if parsed < min {
                return Err(validation_error_with(
                    vec!["query", key],
                    &format!("Input should be greater than or equal to {min}"),
                    "greater_than_equal",
                    Value::String(raw.to_string()),
                    json!({ "ge": min }),
                ));
            }
        }
        if let Some(max) = max {
            if parsed > max {
                return Err(validation_error_with(
                    vec!["query", key],
                    &format!("Input should be less than or equal to {max}"),
                    "less_than_equal",
                    Value::String(raw.to_string()),
                    json!({ "le": max }),
                ));
            }
        }
        Ok(parsed)
    }

    /// A required string parameter. A missing one is FastAPI's 422.
    pub fn required(&self, key: &str) -> Result<&str, ApiError> {
        self.get(key).ok_or_else(|| {
            validation_error_with(
                vec!["query", key],
                "Field required",
                "missing",
                Value::Null,
                Value::Null,
            )
        })
    }

    /// An optional string parameter; an empty string counts as absent, the
    /// way FastAPI's `str | None` treats a query string with no value.
    pub fn optional(&self, key: &str) -> Option<&str> {
        self.get(key).filter(|value| !value.is_empty())
    }

    /// A comma-separated list of field names.
    pub fn field_list(&self, key: &str) -> Option<Vec<String>> {
        let raw = self.get(key)?;
        let names: Vec<String> = raw
            .split(',')
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect();
        if names.is_empty() {
            None
        } else {
            Some(names)
        }
    }
}

/// Percent-decode a query component, treating `+` as a space.
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = &raw[index + 1..index + 3];
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The parsed JSON body of a request.
///
/// A body that is not valid JSON is the 422 FastAPI produces for a body it
/// cannot decode.
pub fn parse_body(body: Option<&str>) -> Result<Value, ApiError> {
    let Some(raw) = body else {
        return Ok(Value::Null);
    };
    if raw.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(raw).map_err(|err| {
        validation_error_with(
            vec!["body"],
            &format!("JSON decode error: {err}"),
            "json_invalid",
            Value::String(raw.to_string()),
            Value::Null,
        )
    })
}

/// The body as an object, or an empty object when there is none.
///
/// Several Python routes take `body: dict[str, Any]` and treat a missing body
/// as `{}`; the shape of the response does not depend on the difference.
pub fn body_object(body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = parse_body(body)?;
    match parsed {
        Value::Null => Ok(json!({})),
        Value::Object(_) => Ok(parsed),
        _ => Err(validation_error_with(
            vec!["body"],
            "Input should be a valid dictionary or object to extract fields from",
            "dict_type",
            parsed.clone(),
            Value::Null,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pairs() {
        let query = Query::parse(Some("page=2&sport=running&flag"));
        assert_eq!(query.get("page"), Some("2"));
        assert_eq!(query.get("sport"), Some("running"));
        assert_eq!(query.get("flag"), Some(""));
        assert_eq!(query.get("missing"), None);
    }

    #[test]
    fn decodes_percent_escapes() {
        let query = Query::parse(Some("q=long%20run&s=a+b"));
        assert_eq!(query.get("q"), Some("long run"));
        assert_eq!(query.get("s"), Some("a b"));
    }

    #[test]
    fn bounds_are_checked() {
        let query = Query::parse(Some("page=0"));
        let err = query.int("page", 1, Some(1), None).unwrap_err();
        assert_eq!(err.status, 422);
    }

    #[test]
    fn missing_required_is_422() {
        let query = Query::parse(None);
        let err = query.required("ids").unwrap_err();
        assert_eq!(err.status, 422);
    }
}
