//! Reading the harness's arguments.
//!
//! The schemas in `schema.rs` say what a tool accepts; this module says what
//! happens when the harness sends something else. Every failure is a
//! [`McpError::InvalidParams`] that names the field — a model that gets the
//! argument wrong should be told which argument, not that "something went
//! wrong".

use chrono::NaiveDate;
use serde_json::Value;

use crate::error::McpError;

/// The empty argument object, for a call that sends nothing.
static NO_ARGUMENTS: Value = Value::Null;

/// The arguments of one call.
pub struct Args<'a> {
    tool: &'a str,
    value: &'a Value,
}

impl<'a> Args<'a> {
    /// The arguments of a call.
    ///
    /// A call with no arguments is a call with an empty object, and a call
    /// whose arguments are not an object is a bad request.
    pub fn new(tool: &'a str, value: &'a Value) -> Result<Self, McpError> {
        match value {
            Value::Null => Ok(Self {
                tool,
                value: &NO_ARGUMENTS,
            }),
            Value::Object(_) => Ok(Self { tool, value }),
            _ => Err(McpError::invalid(
                tool,
                "arguments",
                "must be a JSON object",
            )),
        }
    }

    /// The tool's name, for the errors.
    pub fn tool(&self) -> &str {
        self.tool
    }

    /// The arguments as they were sent.
    pub fn value(&self) -> &Value {
        self.value
    }

    /// Reject an argument the tool does not know.
    ///
    /// The schemas are strict — `additionalProperties` is false — and this
    /// makes the strictness real, so a typo is an error instead of a
    /// silently ignored argument.
    pub fn reject_unknown(&self, allowed: &[&str]) -> Result<(), McpError> {
        let Some(object) = self.value.as_object() else {
            return Ok(());
        };
        for key in object.keys() {
            if !allowed.contains(&key.as_str()) {
                return Err(McpError::invalid(
                    self.tool,
                    key,
                    format!("unknown argument; this tool accepts {allowed:?}"),
                ));
            }
        }
        Ok(())
    }

    /// A string argument.
    pub fn opt_str(&self, key: &str) -> Result<Option<&str>, McpError> {
        match self.value.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(text)) => Ok(Some(text.as_str())),
            Some(_) => Err(McpError::invalid(self.tool, key, "must be a string")),
        }
    }

    /// A string argument that must be there.
    pub fn req_str(&self, key: &str) -> Result<&str, McpError> {
        self.opt_str(key)?
            .ok_or_else(|| McpError::invalid(self.tool, key, "is required"))
    }

    /// A string argument from a fixed set.
    pub fn enum_str(&self, key: &str, allowed: &[&str]) -> Result<Option<String>, McpError> {
        match self.opt_str(key)? {
            None => Ok(None),
            Some(text) if allowed.contains(&text) => Ok(Some(text.to_string())),
            Some(text) => Err(McpError::invalid(
                self.tool,
                key,
                format!("must be one of {allowed:?}, not {text:?}"),
            )),
        }
    }

    /// An integer argument.
    pub fn opt_i64(&self, key: &str) -> Result<Option<i64>, McpError> {
        match self.value.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(number)) => number
                .as_i64()
                .map(Some)
                .ok_or_else(|| McpError::invalid(self.tool, key, "must be an integer")),
            Some(_) => Err(McpError::invalid(self.tool, key, "must be an integer")),
        }
    }

    /// An integer argument that must be there.
    pub fn req_i64(&self, key: &str) -> Result<i64, McpError> {
        self.opt_i64(key)?
            .ok_or_else(|| McpError::invalid(self.tool, key, "is required"))
    }

    /// An integer argument with a range.
    pub fn bounded_i64(
        &self,
        key: &str,
        default: i64,
        min: i64,
        max: i64,
    ) -> Result<i64, McpError> {
        let value = self.opt_i64(key)?.unwrap_or(default);
        if value < min || value > max {
            return Err(McpError::invalid(
                self.tool,
                key,
                format!("must be between {min} and {max}"),
            ));
        }
        Ok(value)
    }

    /// A number argument.
    pub fn opt_f64(&self, key: &str) -> Result<Option<f64>, McpError> {
        match self.value.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(number)) => number
                .as_f64()
                .map(Some)
                .ok_or_else(|| McpError::invalid(self.tool, key, "must be a number")),
            Some(_) => Err(McpError::invalid(self.tool, key, "must be a number")),
        }
    }

    /// A number argument with a range.
    pub fn bounded_f64(
        &self,
        key: &str,
        default: f64,
        min: f64,
        max: f64,
    ) -> Result<f64, McpError> {
        let value = self.opt_f64(key)?.unwrap_or(default);
        if !value.is_finite() || value < min || value > max {
            return Err(McpError::invalid(
                self.tool,
                key,
                format!("must be a number between {min} and {max}"),
            ));
        }
        Ok(value)
    }

    /// A boolean argument.
    pub fn opt_bool(&self, key: &str) -> Result<Option<bool>, McpError> {
        match self.value.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(flag)) => Ok(Some(*flag)),
            Some(_) => Err(McpError::invalid(self.tool, key, "must be a boolean")),
        }
    }

    /// A list of strings.
    pub fn str_list(&self, key: &str) -> Result<Vec<String>, McpError> {
        match self.value.get(key) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Array(items)) => items
                .iter()
                .map(|item| {
                    item.as_str().map(str::to_string).ok_or_else(|| {
                        McpError::invalid(self.tool, key, "must be a list of strings")
                    })
                })
                .collect(),
            Some(_) => Err(McpError::invalid(
                self.tool,
                key,
                "must be a list of strings",
            )),
        }
    }

    /// A list of strings, each of which must be in a fixed set.
    pub fn enum_list(&self, key: &str, allowed: &[&str]) -> Result<Vec<String>, McpError> {
        let values = self.str_list(key)?;
        for value in &values {
            if !allowed.contains(&value.as_str()) {
                return Err(McpError::invalid(
                    self.tool,
                    key,
                    format!("{value:?} is not one of {allowed:?}"),
                ));
            }
        }
        Ok(values)
    }

    /// A date argument, `YYYY-MM-DD`.
    pub fn date(&self, key: &str) -> Result<Option<NaiveDate>, McpError> {
        let Some(raw) = self.opt_str(key)? else {
            return Ok(None);
        };
        parse_date(self.tool, key, raw).map(Some)
    }
}

/// Parse a `YYYY-MM-DD` date.
pub fn parse_date(tool: &str, field: &str, raw: &str) -> Result<NaiveDate, McpError> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|_| {
        McpError::invalid(
            tool,
            field,
            format!("must be a date of the form YYYY-MM-DD, not {raw:?}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_unknown_argument_is_an_error() {
        let value = json!({ "limit": 10, "colour": "red" });
        let args = Args::new("search_activities", &value).expect("args");
        let error = args.reject_unknown(&["limit"]).expect_err("unknown");
        assert_eq!(error.tool(), "search_activities");
        assert!(error.to_string().contains("colour"));
    }

    #[test]
    fn a_range_is_enforced() {
        let value = json!({ "limit": 500 });
        let args = Args::new("search_activities", &value).expect("args");
        let error = args
            .bounded_i64("limit", 20, 1, 100)
            .expect_err("out of range");
        assert!(error.to_string().contains("between 1 and 100"));
    }

    #[test]
    fn a_date_must_be_iso() {
        let value = json!({ "from": "01/10/2026" });
        let args = Args::new("search_activities", &value).expect("args");
        let error = args.date("from").expect_err("bad date");
        assert!(error.to_string().contains("YYYY-MM-DD"));
    }
}
