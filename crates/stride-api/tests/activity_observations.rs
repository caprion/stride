//! The source's own facts about a run, read off the activity search payload.
//!
//! Three rules are proved here, and all three are about honesty:
//!
//! * a field the payload does not carry has **no row at all**;
//! * a field the payload carries as null has a row whose value is null —
//!   never a zero for "no time in that zone";
//! * **only the fields on the list are kept.** Garmin's own estimates — the
//!   VO₂ max, the training stress score, the activity training load, the
//!   lactate threshold, the intensity factor, the stress and the body
//!   battery — are judgements, and the contract excludes them: a payload
//!   that carries one has no row for it.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::{DateTime, Utc};
use serde_json::json;
use stride_api::activity_observations::{observations, SOURCE};

const ENDPOINT: &str = "/activitylist-service/activities/search/activities";

fn instant() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-05-12T06:30:15Z")
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// The rows one search entry carries.
fn rows(entry: &serde_json::Value) -> Vec<stride_store::NewActivityObservation> {
    observations(entry, 7, instant(), ENDPOINT, instant())
}

/// The value of one field, if a row carries it.
fn value_of(rows: &[stride_store::NewActivityObservation], field: &str) -> Option<Option<f64>> {
    rows.iter()
        .find(|row| row.field == field)
        .map(|row| row.value)
}

#[test]
fn the_watchs_own_zone_minutes_are_kept_with_their_unit() {
    let entry = json!({
        "activityId": 1,
        "hrTimeInZone_1": 0,
        "hrTimeInZone_2": 1_100,
        "hrTimeInZone_3": 359,
        "hrTimeInZone_4": 120,
        "hrTimeInZone_5": 0,
    });

    let rows = rows(&entry);
    assert_eq!(rows.len(), 5);
    assert_eq!(value_of(&rows, "hrTimeInZone_1"), Some(Some(0.0)));
    assert_eq!(value_of(&rows, "hrTimeInZone_3"), Some(Some(359.0)));
    assert_eq!(value_of(&rows, "hrTimeInZone_5"), Some(Some(0.0)));

    for row in &rows {
        assert_eq!(row.unit.as_deref(), Some("s"));
        assert_eq!(row.source, SOURCE);
        assert_eq!(row.endpoint, ENDPOINT);
        assert_eq!(row.activity_id, 7);
        assert_eq!(row.ordinal, 0);
        assert_eq!(row.observed_at, Some(instant()));
        assert_eq!(row.retrieved_at, instant());
    }
}

#[test]
fn a_field_the_payload_does_not_carry_has_no_row_at_all() {
    let entry = json!({
        "activityId": 1,
        "hrTimeInZone_3": 359,
    });

    let rows = rows(&entry);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field, "hrTimeInZone_3");
    assert!(value_of(&rows, "hrTimeInZone_4").is_none());
    assert!(value_of(&rows, "steps").is_none());
}

#[test]
fn a_field_the_payload_carries_as_null_has_a_row_with_no_value() {
    let entry = json!({
        "activityId": 1,
        "hrTimeInZone_3": null,
        "steps": 4_210,
    });

    let rows = rows(&entry);
    assert_eq!(rows.len(), 2);
    assert_eq!(value_of(&rows, "hrTimeInZone_3"), Some(None));
    assert_eq!(value_of(&rows, "steps"), Some(Some(4_210.0)));
}

#[test]
fn the_training_effect_and_the_athletes_own_effort_are_kept() {
    let entry = json!({
        "activityId": 1,
        "aerobicTrainingEffect": 3.2,
        "anaerobicTrainingEffect": 1.1,
        "aerobicTrainingEffectMessage": "maintaining",
        "directWorkoutRpe": 4,
        "directWorkoutFeel": "comfortable",
        "moderateIntensityMinutes": 22,
        "vigorousIntensityMinutes": 7,
        "avgRespirationRate": 14.5,
        "steps": 4_210,
    });

    let rows = rows(&entry);
    assert_eq!(value_of(&rows, "aerobicTrainingEffect"), Some(Some(3.2)));
    assert_eq!(value_of(&rows, "anaerobicTrainingEffect"), Some(Some(1.1)));
    assert_eq!(value_of(&rows, "directWorkoutRpe"), Some(Some(4.0)));
    assert_eq!(
        value_of(&rows, "moderateIntensityMinutes"),
        Some(Some(22.0))
    );
    assert_eq!(value_of(&rows, "vigorousIntensityMinutes"), Some(Some(7.0)));
    assert_eq!(value_of(&rows, "avgRespirationRate"), Some(Some(14.5)));
    assert_eq!(value_of(&rows, "steps"), Some(Some(4_210.0)));

    // The messages are text, and they are kept as text.
    let message = rows
        .iter()
        .find(|row| row.field == "aerobicTrainingEffectMessage")
        .expect("the message is there");
    assert_eq!(message.value, None);
    assert_eq!(message.text_value.as_deref(), Some("maintaining"));
}

#[test]
fn garmins_own_estimates_are_not_kept() {
    // Every judgement the contract excludes, in one payload.
    let entry = json!({
        "activityId": 1,
        "vO2MaxValue": 52.0,
        "trainingStressScore": 245.0,
        "activityTrainingLoad": 320.0,
        "lactateThresholdBpm": 168.0,
        "lactateThresholdSpeed": 3.6,
        "intensityFactor": 0.82,
        "avgStress": 30.0,
        "differenceBodyBattery": -18.0,
        "hrTimeInZone_3": 359,
    });

    let rows = rows(&entry);
    // Only the one observation the app keeps.
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field, "hrTimeInZone_3");
    for judgement in [
        "vO2MaxValue",
        "trainingStressScore",
        "activityTrainingLoad",
        "lactateThresholdBpm",
        "lactateThresholdSpeed",
        "intensityFactor",
        "avgStress",
        "differenceBodyBattery",
    ] {
        assert!(
            value_of(&rows, judgement).is_none(),
            "the judgement {judgement} was kept"
        );
    }
}

#[test]
fn a_payload_that_carries_nothing_keeps_nothing() {
    let entry = json!({
        "activityId": 1,
        "activityName": "a run",
        "distance": 10_000.0,
    });

    assert!(rows(&entry).is_empty());
}
