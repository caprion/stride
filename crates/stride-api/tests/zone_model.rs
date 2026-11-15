//! The zone model the sync reads from Garmin.
//!
//! The model is the athlete's own configuration, read once per pull from
//! `/biometric-service/heartRateZones/v2`. A read that fails must not look
//! like a change: the previous model, and its date, stay in place. Absence
//! is not zero and not "no model".

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_ingest::{
    ACTIVITIES_PATH, HEART_RATE_ZONES_PATH, HRV_DAILY_PATH, SLEEP_DAILY_PATH, USER_SUMMARY_PATH,
};
use stride_store::NewHeartRateZoneModel;

/// A synthetic zone model, with invented numbers.
///
/// The repository is public: no athlete's real floors belong in a test.
const ZONE_MODEL: &str = r#"[
  {
    "trainingMethod": "HR_MAX",
    "restingHeartRateUsed": 52,
    "lactateThresholdHeartRateUsed": 0,
    "zone1Floor": 106,
    "zone2Floor": 121,
    "zone3Floor": 135,
    "zone4Floor": 149,
    "zone5Floor": 163,
    "maxHeartRateUsed": 178,
    "restingHrAutoUpdateUsed": true,
    "sport": "DEFAULT"
  }
]"#;

/// Answer the wellness endpoints with a no-data success, so the sync finishes
/// cleanly and reaches the zone read.
fn answer_clean_sync(harness: &common::Harness) {
    harness.router.api().set_garmin_key(common::GARMIN_KEY);
    harness
        .garmin
        .with_json(common::TOKEN_PATH, "\"the-access-value\"");
    harness
        .garmin
        .with_json(ACTIVITIES_PATH, "{\"activityList\": []}");
    for path in [SLEEP_DAILY_PATH, HRV_DAILY_PATH, USER_SUMMARY_PATH] {
        harness.garmin.with_status(path, 204, "");
    }
}

#[test]
fn the_sync_reads_and_stores_the_zone_model() {
    let harness = common::Harness::empty("zone-model-sync");
    answer_clean_sync(&harness);
    harness.garmin.with_json(HEART_RATE_ZONES_PATH, ZONE_MODEL);

    let body = harness.post("/sync/run", "{\"days\": 1}");
    assert_eq!(body["activities"].as_u64(), Some(0));

    let model = harness
        .store
        .current_hr_zone_model()
        .unwrap()
        .unwrap_or_else(|| panic!("the zone model was not stored"));
    assert_eq!(model.sport, "DEFAULT");
    assert_eq!(model.training_method, "HR_MAX");
    assert_eq!(model.max_heart_rate, Some(178.0));
    assert_eq!(model.zone1_floor, 106.0);
    assert_eq!(model.zone2_floor, 121.0);
    assert_eq!(model.zone3_floor, 135.0);
    assert_eq!(model.zone4_floor, 149.0);
    assert_eq!(model.zone5_floor, 163.0);
    assert_eq!(model.read_at, common::instant());
}

#[test]
fn a_failed_zone_read_keeps_the_previous_model_and_its_date() {
    let harness = common::Harness::empty("zone-model-failure");
    let read_at = common::instant() - chrono::Duration::days(3);
    harness
        .store
        .write_hr_zone_model_if_new(&NewHeartRateZoneModel {
            sport: "DEFAULT".to_string(),
            training_method: "HR_MAX".to_string(),
            max_heart_rate: Some(178.0),
            zone1_floor: 106.0,
            zone2_floor: 121.0,
            zone3_floor: 135.0,
            zone4_floor: 149.0,
            zone5_floor: 163.0,
            read_at,
        })
        .unwrap();

    // The zone endpoint is not answered: the fake returns 404, and the
    // previous model must stay in place.
    answer_clean_sync(&harness);
    harness.post("/sync/run", "{\"days\": 1}");

    let model = harness
        .store
        .current_hr_zone_model()
        .unwrap()
        .unwrap_or_else(|| panic!("the zone model was dropped"));
    assert_eq!(harness.store.count_hr_zone_models().unwrap(), 1);
    assert_eq!(model.read_at, read_at);
    assert_eq!(model.zone1_floor, 106.0);
}

#[test]
fn an_absent_zone_payload_is_not_a_model() {
    let harness = common::Harness::empty("zone-model-absent");
    answer_clean_sync(&harness);
    // Garmin answers, but with no model: no entry, no floors.
    harness.garmin.with_json(HEART_RATE_ZONES_PATH, "[]");

    harness.post("/sync/run", "{\"days\": 1}");

    assert_eq!(harness.store.count_hr_zone_models().unwrap(), 0);
    assert!(harness.store.current_hr_zone_model().unwrap().is_none());
}
