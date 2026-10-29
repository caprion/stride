//! The helpers the tools share.
//!
//! Everything here is small and boring on purpose: the tools must not
//! re-implement the router's arithmetic, so they read the numbers the API
//! already computed (the pace, the labels, the zone times) rather than
//! deriving their own.

use serde_json::Value;

use crate::api::StrideApi;
use crate::error::McpError;

/// The fields of a run a list may show.
///
/// This is the disclosure list. Anything the API's activity summary carries
/// that is not here — the Garmin link, the route preview, the external id,
/// the device name, the athlete's own notes — is not shown.
pub const ACTIVITY_ITEM_FIELDS: &[&str] = &[
    "id",
    "name",
    "sport_type",
    "local_date",
    "day_of_week",
    "source",
    "distance_m",
    "moving_time_s",
    "elapsed_time_s",
    "pace_s_per_km",
    "pace_label",
    "elevation_gain_m",
    "avg_hr",
    "max_hr",
    "avg_cadence",
    "calories",
    "perceived_exertion",
    "is_trail",
    "is_commute",
    "running_time_s",
    "walking_time_s",
    "stopped_time_s",
    "running_distance_m",
    "has_geo",
];

/// The fields of a run a detail view may show.
///
/// It is the list above plus the fields that only make sense for one run.
pub const ACTIVITY_DETAIL_FIELDS: &[&str] = &[
    "id",
    "name",
    "sport_type",
    "local_date",
    "day_of_week",
    "start_time",
    "source",
    "distance_m",
    "moving_time_s",
    "moving_time_label",
    "elapsed_time_s",
    "pace_s_per_km",
    "pace_label",
    "elevation_gain_m",
    "avg_hr",
    "max_hr",
    "avg_cadence",
    "max_cadence",
    "calories",
    "perceived_exertion",
    "is_trail",
    "is_commute",
    "running_time_s",
    "walking_time_s",
    "stopped_time_s",
    "running_distance_m",
    "lap_count",
    "has_geo",
];

/// Percent-encode a query-string component.
pub fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Build a query string from the parts that are there.
pub fn query(parts: &[(&str, Option<String>)]) -> String {
    let mut out = String::new();
    for (key, value) in parts {
        let Some(value) = value else { continue };
        if !out.is_empty() {
            out.push('&');
        }
        out.push_str(key);
        out.push('=');
        out.push_str(&encode(value));
    }
    out
}

/// Call a read-only route.
pub fn get(api: &dyn StrideApi, tool: &str, path: &str) -> Result<Value, McpError> {
    api.call("GET", path, None)
        .map_err(|error| McpError::api(tool, &error))
}

/// The `items` of a list answer.
pub fn items(value: &Value) -> Vec<Value> {
    value
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// A number of an object.
pub fn f64_of(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

/// An integer of an object.
pub fn i64_of(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

/// A string of an object.
pub fn str_of<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// A boolean of an object.
pub fn bool_of(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

/// The median of a list of numbers.
///
/// The router's own median, so a group median here is the same number the
/// app shows.
pub fn median(values: &[f64]) -> Option<f64> {
    stride_api::num::median(values)
}

/// The mean of a list of numbers.
///
/// The router's own mean, for the same reason.
pub fn mean(values: &[f64]) -> Option<f64> {
    stride_api::num::mean(values)
}

/// Round to a number of decimals, the way Python does.
///
/// The router's rounding: a tie goes to the even neighbour, on the exact
/// binary value of the double. This crate does not round on its own —
/// a second rounding rule is a second answer.
pub fn round_to(value: f64, digits: usize) -> f64 {
    stride_api::num::round_to(value, digits)
}

/// The pace in seconds per kilometre, from the API's own summary.
pub fn pace_of(summary: &Value) -> Option<f64> {
    f64_of(summary, "pace_s_per_km")
}

/// The pace in seconds per kilometre, computed the way the reference does.
///
/// The reference is `analytics._pace_s_per_km`: the moving time over the
/// distance, and nothing when either is zero. The router owns that
/// arithmetic, so this crate does not do it a second time.
pub fn pace_s_per_km(distance_m: f64, moving_time_s: f64) -> Option<f64> {
    stride_api::num::pace_s_per_km(distance_m, moving_time_s)
}

/// A date as `YYYY-MM-DD`.
pub fn date_text(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// The number of days between two dates.
pub fn days_between(from: chrono::NaiveDate, to: chrono::NaiveDate) -> i64 {
    (to - from).num_days()
}

/// The ISO week key of a date, e.g. `2026-W03`.
pub fn week_key(date: &str) -> String {
    stride_api::num::week_key(date)
}

/// The month key of a date, e.g. `2026-03`.
pub fn month_key(date: &str) -> String {
    stride_api::num::month_key(date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_query_string_escapes_what_it_must() {
        let query = query(&[
            ("q", Some("long run".to_string())),
            ("sport", Some("running".to_string())),
            ("from", None),
        ]);
        assert_eq!(query, "q=long%20run&sport=running");
    }

    #[test]
    fn the_activity_item_does_not_carry_the_garmin_link() {
        let summary = json!({
            "id": 1,
            "name": "Morning run",
            "garmin_url": "https://connect.garmin.com/modern/activity/12345",
            "route_preview": [[1.0, 2.0]],
            "external_id": "12345",
            "description": "felt awful",
            "device": "Forerunner 265",
        });
        let item = crate::output::pick(&summary, ACTIVITY_ITEM_FIELDS);
        assert_eq!(item["id"], json!(1));
        assert!(item.get("garmin_url").is_none());
        assert!(item.get("route_preview").is_none());
        assert!(item.get("external_id").is_none());
        assert!(item.get("description").is_none());
        assert!(item.get("device").is_none());
    }

    #[test]
    fn rounding_matches_the_reference() {
        assert_eq!(round_to(1493.3333, 1), 1493.3);
        // Python's round() is banker's rounding: a tie goes to the even
        // neighbour, on the exact binary value of the double.
        assert_eq!(round_to(2.5, 0), 2.0);
        assert_eq!(round_to(3.5, 0), 4.0);
        assert_eq!(round_to(2.675, 2), 2.67);
    }
}
