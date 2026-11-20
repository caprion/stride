//! The source's own wellness observations.
//!
//! Garmin serves far more wellness than the product shows: the sleep window
//! and the window's confirmation, the unmeasurable time, the naps, the
//! stress durations per band and the stress qualifier, the whole body
//! battery, the seven-day average resting heart rate, the respiration, the
//! day's minimum and maximum heart rate, and the source's own date range.
//!
//! This module maps a Garmin payload to the observations it carries. It is
//! pure: no clock, no store, no I/O. The caller says when the read
//! happened and where the rows go.
//!
//! Three rules hold, and all three are about honesty:
//!
//! * **The day is the response's own.** A payload is filed under the day it
//!   carries, not under the day it was asked for.
//! * **The missingness is kept.** A field the payload does not carry has no
//!   row at all; a field it carries as null has a row whose value is null.
//! * **Nothing is derived.** Every value is one the source reported, in the
//!   source's own unit. The device's own verdicts — the sleep scores, the
//!   HRV status — are judgements, not observations, and are not kept.

use chrono::{DateTime, Utc};
use serde_json::Value;
use stride_store::NewWellnessObservation;

/// Where every observation came from.
pub const SOURCE: &str = "garmin_personal";

/// A reading, as it stands in a payload.
enum Reading {
    /// The payload does not carry the field at all.
    Absent,
    /// The payload carries the field, and its value is null.
    Null,
    /// A number.
    Number(f64),
    /// A value that is not a number.
    Text(String),
}

/// The rows of one read, and the day they belong to.
pub struct Observations {
    /// The day the response carried, when it carried one.
    pub date: Option<String>,
    /// The rows the response carried.
    pub rows: Vec<NewWellnessObservation>,
}

/// The object that holds a payload's fields.
///
/// Garmin wraps some payloads and not others: the sleep comes as
/// `{"dailySleep": {…}}` and the summary as `{"summaryDTO": {…}}`, and both
/// may also arrive bare. The wrapper is looked for, and the payload itself
/// is the fallback.
fn body<'a>(payload: &'a Value, wrapper: &str) -> &'a Value {
    payload.get(wrapper).unwrap_or(payload)
}

/// The reading of one field.
///
/// A field the payload does not carry is absent, and a field it carries as
/// null is null: the two are different facts and they are kept apart.
fn reading(object: &Value, name: &str) -> Reading {
    match object.get(name) {
        None => Reading::Absent,
        Some(Value::Null) => Reading::Null,
        Some(Value::Bool(flag)) => Reading::Text(flag.to_string()),
        Some(Value::Number(number)) => match number.as_f64() {
            Some(value) => Reading::Number(value),
            None => Reading::Absent,
        },
        Some(Value::String(text)) => Reading::Text(text.clone()),
        // An object or an array is not a reading: it is a structure, and the
        // fields inside it are read by their own names.
        Some(_) => Reading::Absent,
    }
}

/// The writer of one read's rows.
struct Writer {
    /// The day the rows belong to.
    date: String,
    /// The API path the rows were read from.
    endpoint: String,
    /// When the read happened.
    retrieved_at: DateTime<Utc>,
    /// The rows so far.
    rows: Vec<NewWellnessObservation>,
}

impl Writer {
    /// A writer for one day and one endpoint.
    fn new(date: &str, endpoint: &str, retrieved_at: DateTime<Utc>) -> Writer {
        Writer {
            date: date.to_string(),
            endpoint: endpoint.to_string(),
            retrieved_at,
            rows: Vec::new(),
        }
    }

    /// Write one row.
    fn push(
        &mut self,
        field: &str,
        value: Option<f64>,
        text: Option<String>,
        unit: Option<&str>,
        ordinal: i64,
    ) {
        self.rows.push(NewWellnessObservation {
            date: self.date.clone(),
            source: SOURCE.to_string(),
            endpoint: self.endpoint.clone(),
            field: field.to_string(),
            ordinal,
            value,
            text_value: text,
            unit: unit.map(str::to_string),
            observed_at: None,
            retrieved_at: self.retrieved_at,
        });
    }

    /// Take one field of the source's own name.
    ///
    /// A field the payload does not carry is left alone: it has no row.
    fn take(&mut self, object: &Value, name: &str, unit: Option<&str>) {
        match reading(object, name) {
            Reading::Absent => {}
            Reading::Null => self.push(name, None, None, unit, 0),
            Reading::Number(value) => self.push(name, Some(value), None, unit, 0),
            Reading::Text(text) => self.push(name, None, Some(text), unit, 0),
        }
    }

    /// Take one field of a repeated item — a nap, a sampled value.
    fn take_item(
        &mut self,
        object: &Value,
        prefix: &str,
        name: &str,
        ordinal: i64,
        unit: Option<&str>,
    ) {
        let field = format!("{prefix}.{name}");
        match reading(object, name) {
            Reading::Absent => {}
            Reading::Null => self.push(&field, None, None, unit, ordinal),
            Reading::Number(value) => self.push(&field, Some(value), None, unit, ordinal),
            Reading::Text(text) => self.push(&field, None, Some(text), unit, ordinal),
        }
    }
}

/// The day an object carries, when it carries one.
///
/// The daily endpoints name the day they are about; the range endpoints
/// name the day of every item they carry. A payload that names no day falls
/// back to the day that was asked for.
fn date_in(object: &Value, names: &[&str]) -> Option<String> {
    for name in names {
        if let Some(text) = object.get(*name).and_then(Value::as_str) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// The day a payload carries, when it carries one.
fn date_of(payload: &Value, wrapper: &str, names: &[&str]) -> Option<String> {
    date_in(body(payload, wrapper), names)
}

/// The sleep of one night, as the source reported it.
///
/// The stages and the durations are already kept by the product's own
/// projection; what was dropped is kept here — the window and its
/// confirmation, the unmeasurable time, the naps, the respiration.
pub fn sleep(
    payload: &Value,
    fallback_date: &str,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> Observations {
    let object = body(payload, "dailySleep");
    let date = date_of(payload, "dailySleep", &["calendarDate"])
        .unwrap_or_else(|| fallback_date.to_string());
    let mut writer = Writer::new(&date, endpoint, retrieved_at);

    for name in [
        "sleepTimeSeconds",
        "deepSleepSeconds",
        "remSleepSeconds",
        "lightSleepSeconds",
        "awakeSleepSeconds",
        "unmeasurableSleepSeconds",
        "napTimeSeconds",
    ] {
        writer.take(object, name, Some("s"));
    }
    writer.take(object, "avgHeartRate", Some("bpm"));
    writer.take(object, "avgSleepStress", None);
    writer.take(object, "awakeCount", None);
    for name in [
        "averageRespirationValue",
        "lowestRespirationValue",
        "highestRespirationValue",
    ] {
        writer.take(object, name, None);
    }
    // The window: when the sleep started and ended, and the device's own
    // word for how that window was confirmed.
    for name in [
        "sleepStartTimestampGMT",
        "sleepEndTimestampGMT",
        "sleepStartTimestampLocal",
        "sleepEndTimestampLocal",
    ] {
        writer.take(object, name, None);
    }
    writer.take(object, "sleepWindowConfirmed", None);
    writer.take(object, "sleepWindowConfirmationType", None);

    // The naps, one row per field of each nap.
    if let Some(naps) = object.get("dailyNapDTOS").and_then(Value::as_array) {
        for (index, nap) in naps.iter().enumerate() {
            let ordinal = index as i64;
            writer.take_item(nap, "dailyNapDTOS", "napTimeSec", ordinal, Some("s"));
            writer.take_item(nap, "dailyNapDTOS", "napStartTimestampGMT", ordinal, None);
            writer.take_item(nap, "dailyNapDTOS", "napEndTimestampGMT", ordinal, None);
            writer.take_item(nap, "dailyNapDTOS", "napFeedback", ordinal, None);
            writer.take_item(nap, "dailyNapDTOS", "napSource", ordinal, None);
            writer.take_item(nap, "dailyNapDTOS", "napStartTimeOffset", ordinal, None);
            writer.take_item(nap, "dailyNapDTOS", "napEndTimeOffset", ordinal, None);
        }
    }

    Observations {
        date: Some(date),
        rows: writer.rows,
    }
}

/// The heart-rate variability of one night, as the source reported it.
///
/// The status and the feedback phrase are the device's verdict on the
/// athlete, and they are not kept.
pub fn hrv(
    payload: &Value,
    fallback_date: &str,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> Observations {
    let object = body(payload, "hrvSummary");
    let date = date_of(payload, "hrvSummary", &["calendarDate"])
        .unwrap_or_else(|| fallback_date.to_string());
    let mut writer = Writer::new(&date, endpoint, retrieved_at);

    for name in ["lastNightAvg", "weeklyAvg", "lastNight5MinHigh"] {
        writer.take(object, name, Some("ms"));
    }
    // The baseline: the band the source measured the night's reading
    // against. The numbers are thresholds, not a score of the athlete.
    if let Some(baseline) = object.get("baseline") {
        for name in ["lowUpper", "balancedLow", "balancedUpper", "markerValue"] {
            writer.take_item(baseline, "baseline", name, 0, Some("ms"));
        }
    }

    Observations {
        date: Some(date),
        rows: writer.rows,
    }
}

/// The summary of one day, as the source reported it.
///
/// This is where the dropped readings live: the stress durations per band
/// and the stress qualifier, the body battery, the seven-day average resting
/// heart rate, the day's minimum and maximum heart rate, the respiration,
/// the steps and the source's own range of the day.
pub fn summary(
    payload: &Value,
    fallback_date: &str,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> Observations {
    let object = body(payload, "summaryDTO");
    let date = date_of(payload, "summaryDTO", &["calendarDate"])
        .unwrap_or_else(|| fallback_date.to_string());
    let mut writer = Writer::new(&date, endpoint, retrieved_at);

    for name in [
        "restingHeartRate",
        "minHeartRate",
        "maxHeartRate",
        "lastSevenDaysAvgRestingHeartRate",
    ] {
        writer.take(object, name, Some("bpm"));
    }
    writer.take(object, "averageStressLevel", None);
    writer.take(object, "maxStressLevel", None);
    // The stress durations per band, in the seconds the source states.
    for name in [
        "stressDuration",
        "restStressDuration",
        "activityStressDuration",
        "uncategorizedStressDuration",
        "totalStressDuration",
        "lowStressDuration",
        "mediumStressDuration",
        "highStressDuration",
    ] {
        writer.take(object, name, Some("s"));
    }
    writer.take(object, "stressQualifier", None);
    // The body battery: the device's own charge estimate. It is kept as
    // the source reported it and is never combined into anything.
    for name in [
        "bodyBatteryChargedValue",
        "bodyBatteryDrainedValue",
        "bodyBatteryHighestValue",
        "bodyBatteryLowestValue",
        "bodyBatteryMostRecentValue",
        "bodyBatteryDuringSleep",
        "bodyBatteryAtWakeTime",
    ] {
        writer.take(object, name, None);
    }
    writer.take(object, "totalSteps", Some("count"));
    for name in [
        "avgWakingRespirationValue",
        "highestRespirationValue",
        "lowestRespirationValue",
        "latestRespirationValue",
    ] {
        writer.take(object, name, None);
    }
    // The source's own range of the day.
    writer.take(object, "wellnessStartTimeGmt", None);
    writer.take(object, "wellnessEndTimeGmt", None);

    Observations {
        date: Some(date),
        rows: writer.rows,
    }
}

/// The stress report, as the source reported it.
///
/// The report is a range: every item carries its own day, and every item is
/// filed under the day it carries.
pub fn stress(
    payload: &Value,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> Vec<NewWellnessObservation> {
    let mut rows = Vec::new();
    for item in items(payload, "stressValuesArray") {
        let date = match date_in(item, &["calendarDate"]) {
            Some(date) => date,
            None => continue,
        };
        let values = item.get("values").unwrap_or(item);
        let mut writer = Writer::new(&date, endpoint, retrieved_at);
        writer.take(values, "avgLevel", None);
        writer.take(values, "maxLevel", None);
        for name in [
            "restProportion",
            "activityProportion",
            "uncategorizedProportion",
            "lowStressProportion",
            "mediumStressProportion",
            "highStressProportion",
        ] {
            writer.take(values, name, None);
        }
        // The durations per band, in the milliseconds the source states.
        for name in [
            "totalDurationInMillis",
            "restDurationInMillis",
            "activityDurationInMillis",
            "uncategorizedDurationInMillis",
            "lowStressDurationInMillis",
            "mediumStressDurationInMillis",
            "highStressDurationInMillis",
        ] {
            writer.take(values, name, Some("ms"));
        }
        writer.take(values, "qualifier", None);
        rows.extend(writer.rows);
    }
    rows
}

/// The body battery report, as the source reported it.
///
/// The report is a range: every item carries its own day. The curve is
/// kept as it came — one row per sample, with the instant the sample was
/// taken.
pub fn body_battery(
    payload: &Value,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> Vec<NewWellnessObservation> {
    let mut rows = Vec::new();
    for item in items(payload, "bodyBatteryReports") {
        let date = match date_in(item, &["date", "calendarDate"]) {
            Some(date) => date,
            None => continue,
        };
        let mut writer = Writer::new(&date, endpoint, retrieved_at);
        // The amount the battery was charged and drained, on the scale the
        // source states.
        writer.take(item, "charged", Some("0-100"));
        writer.take(item, "drained", Some("0-100"));
        writer.take(item, "startTimestampGMT", None);
        writer.take(item, "endTimestampGMT", None);

        // The curve: one row per sample, with the instant it was taken.
        if let Some(samples) = item.get("bodyBatteryValuesArray").and_then(Value::as_array) {
            for (index, sample) in samples.iter().enumerate() {
                let level = match sample {
                    Value::Array(pair) => pair.get(1).and_then(Value::as_f64),
                    Value::Object(_) => sample.get("value").and_then(Value::as_f64),
                    _ => None,
                };
                let Some(level) = level else { continue };
                let observed_at = match sample {
                    Value::Array(pair) => pair.first().and_then(Value::as_i64).map(|millis| {
                        chrono::DateTime::from_timestamp_millis(millis).unwrap_or(retrieved_at)
                    }),
                    _ => None,
                };
                writer.rows.push(NewWellnessObservation {
                    date: date.clone(),
                    source: SOURCE.to_string(),
                    endpoint: endpoint.to_string(),
                    field: "bodyBatteryValuesArray".to_string(),
                    ordinal: index as i64,
                    value: Some(level),
                    text_value: None,
                    unit: Some("0-100".to_string()),
                    observed_at,
                    retrieved_at,
                });
            }
        }
        rows.extend(writer.rows);
    }
    rows
}

/// The items of a range payload.
///
/// The reports answer with a list; some of them answer with the list
/// wrapped in a name. Both are read.
fn items<'a>(payload: &'a Value, wrapper: &str) -> Vec<&'a Value> {
    let list = payload
        .as_array()
        .or_else(|| payload.get(wrapper).and_then(Value::as_array));
    match list {
        Some(items) => items.iter().collect(),
        None => Vec::new(),
    }
}
