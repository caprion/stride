//! The source's own facts about a run, read off the activity search payload.
//!
//! The activity search response carries more than the app keeps: the
//! watch's own zone minutes (`hrTimeInZone_1..5`, over the **elapsed**
//! time), the aerobic and anaerobic training effect and their messages,
//! the athlete's own recorded effort, the moderate and vigorous minutes,
//! the respiration and the steps.
//!
//! **They ride on the payload the sync already fetches.** No extra HTTP
//! request is made, and the pull's request budget does not grow: the
//! listing loop already holds the search entry it is reading, and the
//! observations are written from it.
//!
//! Three rules hold, and all three are about honesty:
//!
//! * **The list is closed.** The app keeps the fields below and no others.
//!   Garmin's own estimates — the VO₂ max, the training stress score, the
//!   activity training load, the lactate threshold, the intensity factor,
//!   the stress and the body battery — are judgements, and the contract
//!   excludes them: a field that is not on the list has no row, whatever
//!   the payload carried.
//! * **The missingness is kept.** A field the payload does not carry has
//!   no row at all; a field it carries as null has a row whose value is
//!   null. Nothing is ever turned into a zero, and nothing is invented to
//!   fill a gap — never a zero for "no time in that zone".
//! * **Nothing is derived.** Every value is one the source reported, in the
//!   source's own unit.

use chrono::{DateTime, Utc};
use serde_json::Value;
use stride_store::NewActivityObservation;

/// Where every observation came from.
pub const SOURCE: &str = "garmin_personal";

/// The numeric fields the activity search payload carries, and the unit the
/// source states for each.
///
/// A field the payload does not carry has no row. A field it carries as
/// null has a row whose value is null.
const NUMERIC: &[(&str, Option<&str>)] = &[
    // The watch's own zone minutes, over the elapsed time.
    ("hrTimeInZone_1", Some("s")),
    ("hrTimeInZone_2", Some("s")),
    ("hrTimeInZone_3", Some("s")),
    ("hrTimeInZone_4", Some("s")),
    ("hrTimeInZone_5", Some("s")),
    // The training effect and the athlete's own recorded effort.
    ("aerobicTrainingEffect", None),
    ("anaerobicTrainingEffect", None),
    ("trainingEffect", None),
    ("directWorkoutRpe", None),
    ("directWorkoutFeel", None),
    // The minutes the source calls moderate and vigorous, its respiration
    // and its steps.
    ("moderateIntensityMinutes", Some("min")),
    ("vigorousIntensityMinutes", Some("min")),
    ("avgRespirationRate", Some("brpm")),
    ("steps", Some("count")),
];

/// The text fields the activity search payload carries.
const TEXT: &[&str] = &[
    "aerobicTrainingEffectMessage",
    "anaerobicTrainingEffectMessage",
];

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
        // An object or an array is not a reading: it is a structure, and
        // the fields inside it are read by their own names.
        Some(_) => Reading::Absent,
    }
}

/// The observations one search entry carries.
///
/// The entry is the search payload's own item for the run, the run's id is
/// the row the observations belong to, and `observed_at` is the instant the
/// response carried them — the run's own start.
///
/// Returns no rows for a payload that carries none of the fields.
pub fn observations(
    entry: &Value,
    activity_id: i64,
    observed_at: DateTime<Utc>,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> Vec<NewActivityObservation> {
    let mut rows = Vec::new();

    for (field, unit) in NUMERIC {
        match reading(entry, field) {
            Reading::Absent => {}
            Reading::Null => rows.push(row(
                activity_id,
                field,
                None,
                None,
                *unit,
                observed_at,
                endpoint,
                retrieved_at,
            )),
            Reading::Number(value) => rows.push(row(
                activity_id,
                field,
                Some(value),
                None,
                *unit,
                observed_at,
                endpoint,
                retrieved_at,
            )),
            Reading::Text(text) => rows.push(row(
                activity_id,
                field,
                None,
                Some(text),
                *unit,
                observed_at,
                endpoint,
                retrieved_at,
            )),
        }
    }

    for field in TEXT {
        match reading(entry, field) {
            Reading::Absent => {}
            Reading::Null => rows.push(row(
                activity_id,
                field,
                None,
                None,
                None,
                observed_at,
                endpoint,
                retrieved_at,
            )),
            Reading::Number(value) => rows.push(row(
                activity_id,
                field,
                Some(value),
                None,
                None,
                observed_at,
                endpoint,
                retrieved_at,
            )),
            Reading::Text(text) => rows.push(row(
                activity_id,
                field,
                None,
                Some(text),
                None,
                observed_at,
                endpoint,
                retrieved_at,
            )),
        }
    }

    rows
}

/// One row of `activity_observations`.
#[allow(clippy::too_many_arguments)]
fn row(
    activity_id: i64,
    field: &str,
    value: Option<f64>,
    text_value: Option<String>,
    unit: Option<&str>,
    observed_at: DateTime<Utc>,
    endpoint: &str,
    retrieved_at: DateTime<Utc>,
) -> NewActivityObservation {
    NewActivityObservation {
        activity_id,
        source: SOURCE.to_string(),
        endpoint: endpoint.to_string(),
        field: field.to_string(),
        ordinal: 0,
        value,
        text_value,
        unit: unit.map(str::to_string),
        observed_at: Some(observed_at),
        retrieved_at,
    }
}
