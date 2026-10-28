//! The input schemas of the tools.
//!
//! The schemas are strict on purpose: `additionalProperties` is false, every
//! enumeration is closed, every number has a range. A harness that reads the
//! schema knows what a tool accepts, and a model that sends something else
//! gets an error that names the field instead of a silently ignored argument.

use serde_json::{json, Map, Value};

/// An object schema with a fixed set of properties.
pub fn object(properties: Value, required: &[&str]) -> Value {
    let mut map = Map::new();
    map.insert("type".into(), json!("object"));
    map.insert("properties".into(), properties);
    if !required.is_empty() {
        map.insert("required".into(), json!(required));
    }
    map.insert("additionalProperties".into(), json!(false));
    Value::Object(map)
}

/// A string.
pub fn string(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

/// A date, `YYYY-MM-DD`.
pub fn date(description: &str) -> Value {
    json!({
        "type": "string",
        "format": "date",
        "pattern": "^\\d{4}-\\d{2}-\\d{2}$",
        "description": description,
    })
}

/// An integer, optionally with a range.
pub fn integer(description: &str, minimum: Option<i64>, maximum: Option<i64>) -> Value {
    let mut map = Map::new();
    map.insert("type".into(), json!("integer"));
    map.insert("description".into(), json!(description));
    if let Some(minimum) = minimum {
        map.insert("minimum".into(), json!(minimum));
    }
    if let Some(maximum) = maximum {
        map.insert("maximum".into(), json!(maximum));
    }
    Value::Object(map)
}

/// A number, optionally with a range.
pub fn number(description: &str, minimum: Option<f64>, maximum: Option<f64>) -> Value {
    let mut map = Map::new();
    map.insert("type".into(), json!("number"));
    map.insert("description".into(), json!(description));
    if let Some(minimum) = minimum {
        map.insert("minimum".into(), json!(minimum));
    }
    if let Some(maximum) = maximum {
        map.insert("maximum".into(), json!(maximum));
    }
    Value::Object(map)
}

/// A boolean.
pub fn boolean(description: &str) -> Value {
    json!({ "type": "boolean", "description": description })
}

/// A list of something.
pub fn array(items: Value, description: &str) -> Value {
    json!({ "type": "array", "items": items, "description": description })
}

/// A string from a fixed set.
pub fn enumeration(values: &[&str], description: &str) -> Value {
    json!({ "type": "string", "enum": values, "description": description })
}

/// A list of strings from a fixed set.
pub fn enumeration_array(values: &[&str], description: &str) -> Value {
    json!({
        "type": "array",
        "items": { "type": "string", "enum": values },
        "description": description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_schema_is_strict() {
        let schema = object(
            json!({ "limit": integer("how many", Some(1), Some(100)) }),
            &["limit"],
        );
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"], json!(["limit"]));
        assert_eq!(schema["properties"]["limit"]["minimum"], json!(1));
        assert_eq!(schema["properties"]["limit"]["maximum"], json!(100));
    }
}
