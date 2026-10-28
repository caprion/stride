//! The shape of a tool's answer, and the bound on its size.
//!
//! Every tool answers with the same envelope: the `data` it was asked for,
//! the `coverage` that says where the data came from and what was there, and
//! the `limitations` that say what the answer cannot be used for. A harness
//! can rely on those three keys without knowing which tool it called.
//!
//! **Every answer is bounded.** The athlete's history is large and a harness's
//! context is small, so a tool that returns "a year of it" is a tool that
//! will break the session. The bounds here are enforced, not assumed: the
//! tools cap every list they build, and [`bound`] is the backstop that
//! guarantees the serialized answer stays inside [`MAX_OUTPUT_BYTES`].

use serde_json::{Map, Value};

/// The largest answer a tool may return, in bytes.
pub const MAX_OUTPUT_BYTES: usize = 128 * 1024;

/// The largest number of items in one list of a tool's answer.
///
/// It is the same as the largest `limit` a tool accepts, so a caller that
/// asks for a hundred runs gets a hundred runs.
pub const MAX_ITEMS: usize = 100;

/// The largest number of samples in one series.
pub const MAX_SAMPLES: usize = 240;

/// The largest number of segments in one analysis.
pub const MAX_SEGMENTS: usize = 60;

/// The largest number of limitations an answer may carry.
pub const MAX_LIMITATIONS: usize = 20;

/// A tool's answer, before it is rendered.
#[derive(Debug, Clone)]
pub struct Envelope {
    data: Value,
    coverage: Map<String, Value>,
    limitations: Vec<String>,
}

impl Envelope {
    /// An answer with nothing but data.
    pub fn new(data: Value) -> Self {
        Self {
            data,
            coverage: Map::new(),
            limitations: Vec::new(),
        }
    }

    /// Say where the answer came from.
    pub fn coverage(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.coverage.insert(key.to_string(), value.into());
        self
    }

    /// Say what the answer cannot be used for.
    pub fn limitation(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        if !self.limitations.iter().any(|existing| existing == &text) {
            self.limitations.push(text);
        }
        self
    }

    /// Render the answer.
    ///
    /// The result is `{tool, data, coverage, limitations}` and it is inside
    /// the size bound: if it is not, the largest list is trimmed until it is,
    /// and the trimming is disclosed as a limitation.
    pub fn finish(mut self, tool: &str) -> Value {
        if self.limitations.len() > MAX_LIMITATIONS {
            self.limitations.truncate(MAX_LIMITATIONS);
        }
        let mut value = serde_json::json!({
            "tool": tool,
            "data": self.data,
            "coverage": Value::Object(self.coverage),
            "limitations": self.limitations,
        });
        let (bounded, trimmed) = bound(value, MAX_OUTPUT_BYTES);
        value = bounded;
        if trimmed {
            if let Some(object) = value.as_object_mut() {
                let limitations = object
                    .entry("limitations")
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Some(list) = limitations.as_array_mut() {
                    list.push(Value::String(
                        "the answer was trimmed to fit the harness context budget".to_string(),
                    ));
                }
            }
        }
        value
    }
}

/// Keep only the named fields of an object.
///
/// This is the disclosure mechanism: a tool says which fields of the API's
/// answer it passes on, and everything else — the Garmin link, the route
/// preview, the device name, the athlete's own notes — is left behind.
pub fn pick(value: &Value, fields: &[&str]) -> Value {
    let Some(object) = value.as_object() else {
        return Value::Null;
    };
    let mut out = Map::new();
    for field in fields {
        if let Some(item) = object.get(*field) {
            out.insert((*field).to_string(), item.clone());
        }
    }
    Value::Object(out)
}

/// Take the first `max` items of a list.
pub fn take(items: Vec<Value>, max: usize) -> Vec<Value> {
    items.into_iter().take(max).collect()
}

/// The size of a value, in bytes, as it would be serialized.
pub fn size(value: &Value) -> usize {
    serde_json::to_string(value)
        .map(|text| text.len())
        .unwrap_or(usize::MAX)
}

/// Bring a value inside a size bound.
///
/// The largest list in the answer is halved until the answer fits. Halving
/// keeps the shape of the answer intact — the keys are still there, the lists
/// are just shorter — and the caller is told that it happened.
pub fn bound(value: Value, cap: usize) -> (Value, bool) {
    let mut value = value;
    let mut trimmed = false;
    for _ in 0..256 {
        if size(&value) <= cap {
            break;
        }
        let mut best: Option<(Vec<Segment>, usize)> = None;
        let mut path = Vec::new();
        longest_list(&value, &mut path, &mut best);
        let Some((path, length)) = best else {
            break;
        };
        if length == 0 {
            break;
        }
        truncate(&mut value, &path, length / 2);
        trimmed = true;
    }
    (value, trimmed)
}

/// One step of a path into a value.
#[derive(Debug, Clone)]
enum Segment {
    /// A key of an object.
    Key(String),
    /// An index of an array.
    Index(usize),
}

/// Find the longest list in a value, and where it is.
fn longest_list(value: &Value, path: &mut Vec<Segment>, best: &mut Option<(Vec<Segment>, usize)>) {
    match value {
        Value::Array(items) => {
            if items.len() > best.as_ref().map_or(0, |(_, length)| *length) {
                *best = Some((path.clone(), items.len()));
            }
            for (index, item) in items.iter().enumerate() {
                path.push(Segment::Index(index));
                longest_list(item, path, best);
                path.pop();
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                path.push(Segment::Key(key.clone()));
                longest_list(item, path, best);
                path.pop();
            }
        }
        _ => {}
    }
}

/// Cut a list down to `keep` items.
fn truncate(value: &mut Value, path: &[Segment], keep: usize) {
    let Some((first, rest)) = path.split_first() else {
        // The path ends here: this is the list itself.
        if let Value::Array(items) = value {
            items.truncate(keep);
        }
        return;
    };
    match first {
        Segment::Key(key) => {
            if let Some(child) = value.get_mut(key) {
                truncate(child, rest, keep);
            }
        }
        Segment::Index(index) => {
            if let Some(child) = value.get_mut(*index) {
                truncate(child, rest, keep);
            }
        }
    }
}

/// The indices of a series that are kept when it is too long.
///
/// The first and the last sample are always kept: a series that starts and
/// ends in the wrong place is worse than a shorter series.
pub fn sample_indices(length: usize, max: usize) -> Vec<usize> {
    if length == 0 {
        return Vec::new();
    }
    if length <= max {
        return (0..length).collect();
    }
    let step = length.div_ceil(max);
    let mut indices: Vec<usize> = (0..length).step_by(step).take(max).collect();
    let last = length - 1;
    if indices.last().copied() != Some(last) {
        indices.push(last);
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pick_keeps_only_the_named_fields() {
        let value = json!({ "id": 1, "name": "run", "garmin_url": "https://x" });
        let picked = pick(&value, &["id", "name"]);
        assert_eq!(picked, json!({ "id": 1, "name": "run" }));
    }

    #[test]
    fn bound_trims_the_longest_list() {
        let items: Vec<Value> = (0..500)
            .map(|index| json!({ "index": index, "pad": "xxxx" }))
            .collect();
        let value = json!({ "items": items });
        let (bounded, trimmed) = bound(value, 4096);
        assert!(trimmed);
        assert!(size(&bounded) <= 4096);
        assert!(bounded["items"].as_array().expect("array").len() < 500);
    }

    #[test]
    fn the_first_and_the_last_sample_survive() {
        let indices = sample_indices(1000, 240);
        assert_eq!(indices.first().copied(), Some(0));
        assert_eq!(indices.last().copied(), Some(999));
        assert!(indices.len() <= 241);
    }

    #[test]
    fn a_short_series_is_untouched() {
        assert_eq!(sample_indices(10, 240), (0..10).collect::<Vec<usize>>());
    }
}
