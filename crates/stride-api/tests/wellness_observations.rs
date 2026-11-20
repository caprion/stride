//! The source's own wellness observations, and the ingest that writes them.
//!
//! Four things are proved here:
//!
//! * a payload with every field is kept whole, with the day and the unit
//!   the source gave;
//! * a field the payload does not carry has **no row**, and a field it
//!   carries as null has a row whose value is null — the two are different
//!   facts;
//! * a failed read keeps the previous value **and** the day it was read,
//!   and the pull says it failed;
//! * the observations are filed under the day the **response** carried, not
//!   the day that was asked for.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_ingest::{
    ACTIVITIES_PATH, BODY_BATTERY_PATH, HEART_RATE_ZONES_PATH, HRV_DAILY_PATH, SLEEP_DAILY_PATH,
    STRESS_PATH, USER_SUMMARY_PATH,
};
use stride_store::WellnessObservation;

/// A synthetic zone model, with invented numbers.
const ZONE_MODEL: &str = r#"[
  {
    "trainingMethod": "HR_MAX",
    "zone1Floor": 106,
    "zone2Floor": 121,
    "zone3Floor": 135,
    "zone4Floor": 149,
    "zone5Floor": 163,
    "maxHeartRateUsed": 178,
    "sport": "DEFAULT"
  }
]"#;

/// A sleep payload with every field the source serves.
///
/// The numbers are invented: the repository is public, and no athlete's
/// night belongs in it.
const SLEEP: &str = r#"{
  "dailySleep": {
    "calendarDate": "2026-05-11",
    "sleepTimeSeconds": 27900.0,
    "deepSleepSeconds": 6420.0,
    "remSleepSeconds": 5580.0,
    "lightSleepSeconds": 14100.0,
    "awakeSleepSeconds": 1800.0,
    "unmeasurableSleepSeconds": 240.0,
    "napTimeSeconds": 900.0,
    "sleepStartTimestampGMT": "2026-05-10T22:30:00.000Z",
    "sleepEndTimestampGMT": "2026-05-11T06:15:00.000Z",
    "sleepWindowConfirmed": true,
    "sleepWindowConfirmationType": "device_confirmed",
    "averageRespirationValue": 14.2,
    "lowestRespirationValue": 12.0,
    "highestRespirationValue": 16.8,
    "avgSleepStress": 18.0,
    "avgHeartRate": 52.0,
    "awakeCount": 3,
    "dailyNapDTOS": [
      {
        "napTimeSec": 900.0,
        "napStartTimestampGMT": "2026-05-11T13:10:00.000Z",
        "napEndTimestampGMT": "2026-05-11T13:25:00.000Z",
        "napFeedback": "NOT_WANTED",
        "napSource": 1,
        "napStartTimeOffset": 40,
        "napEndTimeOffset": 55
      }
    ]
  }
}"#;

/// The HRV payload the source serves.
const HRV: &str = r#"{
  "hrvSummary": {
    "calendarDate": "2026-05-11",
    "lastNightAvg": 58.0,
    "weeklyAvg": 55.5,
    "lastNight5MinHigh": 96.0,
    "status": "BALANCED",
    "feedbackPhrase": "BALANCED",
    "baseline": {
      "lowUpper": 60,
      "balancedLow": 40,
      "balancedUpper": 65,
      "markerValue": 58
    }
  }
}"#;

/// The daily summary, with the readings that used to be dropped.
const SUMMARY: &str = r#"{
  "summaryDTO": {
    "calendarDate": "2026-05-11",
    "restingHeartRate": 47.0,
    "minHeartRate": 44.0,
    "maxHeartRate": 176.0,
    "lastSevenDaysAvgRestingHeartRate": 48.1,
    "averageStressLevel": 28.4,
    "maxStressLevel": 61.0,
    "stressDuration": 86400,
    "restStressDuration": 43200,
    "activityStressDuration": 7200,
    "uncategorizedStressDuration": 3600,
    "totalStressDuration": 86400,
    "lowStressDuration": 21600,
    "mediumStressDuration": 14400,
    "highStressDuration": 7200,
    "stressQualifier": "BALANCED",
    "bodyBatteryChargedValue": 44,
    "bodyBatteryDrainedValue": 61,
    "bodyBatteryHighestValue": 88,
    "bodyBatteryLowestValue": 22,
    "bodyBatteryMostRecentValue": 71,
    "bodyBatteryDuringSleep": 31,
    "bodyBatteryAtWakeTime": 74,
    "totalSteps": 11240,
    "avgWakingRespirationValue": 13.4,
    "highestRespirationValue": 16.8,
    "lowestRespirationValue": 11.2,
    "latestRespirationValue": 13.1,
    "wellnessStartTimeGmt": "2026-05-11T00:00:00.000Z",
    "wellnessEndTimeGmt": "2026-05-11T23:59:59.000Z",
    "vo2Max": 52.0
  }
}"#;

/// The stress report: the durations per band, day by day.
const STRESS: &str = r#"[
  {
    "calendarDate": "2026-05-11",
    "values": {
      "avgLevel": 28,
      "maxLevel": 61,
      "restProportion": 0.5,
      "activityProportion": 0.08,
      "uncategorizedProportion": 0.04,
      "lowStressProportion": 0.25,
      "mediumStressProportion": 0.17,
      "highStressProportion": 0.08,
      "qualifier": "BALANCED",
      "totalDurationInMillis": 86400000,
      "restDurationInMillis": 43200000,
      "activityDurationInMillis": 7200000,
      "uncategorizedDurationInMillis": 3600000,
      "lowStressDurationInMillis": 21600000,
      "mediumStressDurationInMillis": 14400000,
      "highStressDurationInMillis": 7200000
    }
  }
]"#;

/// The body battery report: the day, its charged and drained, and its curve.
const BODY_BATTERY: &str = r#"[
  {
    "date": "2026-05-11",
    "charged": 30.0,
    "drained": 55.0,
    "startTimestampGMT": "2026-05-11T00:00:00.000Z",
    "endTimestampGMT": "2026-05-11T23:59:59.000Z",
    "bodyBatteryValuesArray": [
      [1778524800000, 71],
      [1778528400000, 69]
    ]
  }
]"#;

/// A harness whose Garmin answers the whole wellness set.
fn harness_with_wellness(prefix: &str) -> common::Harness {
    let harness = common::Harness::empty(prefix);
    harness.router.api().set_garmin_key(common::GARMIN_KEY);
    harness
        .garmin
        .with_json(common::TOKEN_PATH, "\"the-access-value\"");
    harness
        .garmin
        .with_json(ACTIVITIES_PATH, "{\"activityList\": []}");
    harness.garmin.with_json(SLEEP_DAILY_PATH, SLEEP);
    harness.garmin.with_json(HRV_DAILY_PATH, HRV);
    harness.garmin.with_json(USER_SUMMARY_PATH, SUMMARY);
    harness.garmin.with_json(STRESS_PATH, STRESS);
    harness.garmin.with_json(BODY_BATTERY_PATH, BODY_BATTERY);
    harness
}

/// The observation of one field, if it is there.
fn observation<'a>(
    rows: &'a [WellnessObservation],
    field: &str,
) -> Option<&'a WellnessObservation> {
    rows.iter().find(|row| row.field == field)
}

#[test]
fn the_sleep_is_kept_with_its_window_its_confirmation_and_its_naps() {
    let harness = harness_with_wellness("wellness-sleep");
    harness.post("/sync/run", "{\"days\": 1}");

    let rows = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");

    // The window the source reported.
    let start = observation(&rows, "sleepStartTimestampGMT").expect("the window is there");
    assert_eq!(
        start.text_value.as_deref(),
        Some("2026-05-10T22:30:00.000Z")
    );
    assert_eq!(start.endpoint, SLEEP_DAILY_PATH);
    assert_eq!(start.retrieved_at, common::instant());

    // The confirmation.
    let confirmed = observation(&rows, "sleepWindowConfirmed").expect("the confirmation is there");
    assert_eq!(confirmed.text_value.as_deref(), Some("true"));
    let kind = observation(&rows, "sleepWindowConfirmationType").expect("the kind is there");
    assert_eq!(kind.text_value.as_deref(), Some("device_confirmed"));

    // The unmeasurable time and the naps.
    let unmeasurable =
        observation(&rows, "unmeasurableSleepSeconds").expect("the unmeasurable time is there");
    assert_eq!(unmeasurable.value, Some(240.0));
    assert_eq!(unmeasurable.unit.as_deref(), Some("s"));
    let nap = observation(&rows, "dailyNapDTOS.napTimeSec").expect("the nap is there");
    assert_eq!(nap.value, Some(900.0));
    assert_eq!(nap.ordinal, 0);

    // The respiration.
    let respiration =
        observation(&rows, "averageRespirationValue").expect("the respiration is there");
    assert_eq!(respiration.value, Some(14.2));

    // The stages the product already keeps are here too.
    let stages = observation(&rows, "deepSleepSeconds").expect("the stages are there");
    assert_eq!(stages.value, Some(6420.0));
}

#[test]
fn the_summary_keeps_the_stress_durations_the_qualifier_and_the_body_battery() {
    let harness = harness_with_wellness("wellness-summary");
    harness.post("/sync/run", "{\"days\": 1}");

    let rows = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");

    // The stress durations per band.
    for (field, value) in [
        ("restStressDuration", 43_200.0),
        ("lowStressDuration", 21_600.0),
        ("mediumStressDuration", 14_400.0),
        ("highStressDuration", 7_200.0),
    ] {
        let row = observation(&rows, field).expect("the duration is there");
        assert_eq!(row.value, Some(value), "{field}");
        assert_eq!(row.unit.as_deref(), Some("s"), "{field}");
    }

    // The stress qualifier.
    let qualifier = observation(&rows, "stressQualifier").expect("the qualifier is there");
    assert_eq!(qualifier.text_value.as_deref(), Some("BALANCED"));

    // The body battery.
    let charged = observation(&rows, "bodyBatteryChargedValue").expect("the battery is there");
    assert_eq!(charged.value, Some(44.0));

    // The seven-day average resting heart rate, and the day's own range.
    let average = observation(&rows, "lastSevenDaysAvgRestingHeartRate")
        .expect("the seven-day average is there");
    assert_eq!(average.value, Some(48.1));
    assert_eq!(average.unit.as_deref(), Some("bpm"));
    let lowest = observation(&rows, "minHeartRate").expect("the minimum is there");
    assert_eq!(lowest.value, Some(44.0));

    // The source's own range of the day.
    let start = observation(&rows, "wellnessStartTimeGmt").expect("the range is there");
    assert_eq!(
        start.text_value.as_deref(),
        Some("2026-05-11T00:00:00.000Z")
    );

    // The device's own judgements are not observations, and are not kept.
    assert!(observation(&rows, "vo2Max").is_none());
    assert!(observation(&rows, "status").is_none());
    assert!(observation(&rows, "sleepScores").is_none());
}

#[test]
fn the_reports_are_read_and_kept_under_the_day_they_carry() {
    let harness = harness_with_wellness("wellness-reports");
    harness.post("/sync/run", "{\"days\": 1}");

    let rows = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");

    // The stress report: the durations per band, in the milliseconds the
    // source states.
    let rest = observation(&rows, "restDurationInMillis").expect("the stress report is there");
    assert_eq!(rest.value, Some(43_200_000.0));
    assert_eq!(rest.unit.as_deref(), Some("ms"));
    assert_eq!(rest.endpoint, STRESS_PATH);

    // The body battery report: the day's own charged and drained, and the
    // curve, sample by sample.
    let charged = observation(&rows, "charged").expect("the report is there");
    assert_eq!(charged.value, Some(30.0));
    assert_eq!(charged.unit.as_deref(), Some("0-100"));
    assert_eq!(charged.endpoint, BODY_BATTERY_PATH);
    let curve: Vec<&WellnessObservation> = rows
        .iter()
        .filter(|row| row.field == "bodyBatteryValuesArray")
        .collect();
    assert_eq!(curve.len(), 2);
    assert_eq!(curve[0].value, Some(71.0));
    assert_eq!(curve[0].ordinal, 0);
    assert!(curve[0].observed_at.is_some());
    assert_eq!(curve[1].ordinal, 1);
}

#[test]
fn a_field_the_source_did_not_send_has_no_row_and_a_null_field_has_an_empty_one() {
    let harness = harness_with_wellness("wellness-missing");
    // The sleep payload carries every field but the unmeasurable time, and
    // it carries the confirmation as null.
    harness.garmin.with_json(
        SLEEP_DAILY_PATH,
        r#"{
          "dailySleep": {
            "calendarDate": "2026-05-11",
            "sleepTimeSeconds": 27900.0,
            "unmeasurableSleepSeconds": null,
            "sleepWindowConfirmed": null
          }
        }"#,
    );
    harness.post("/sync/run", "{\"days\": 1}");

    let rows = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");

    // The field the source sent as null has a row, and the row has no
    // value: it is not the same as a field that was never sent.
    let unmeasurable =
        observation(&rows, "unmeasurableSleepSeconds").expect("the null field has a row");
    assert_eq!(unmeasurable.value, None);
    assert_eq!(unmeasurable.text_value, None);
    let confirmed = observation(&rows, "sleepWindowConfirmed").expect("the null field has a row");
    assert_eq!(confirmed.value, None);
    assert_eq!(confirmed.text_value, None);

    // The field the source did not send at all has no row.
    assert!(observation(&rows, "napTimeSeconds").is_none());
    assert!(observation(&rows, "dailyNapDTOS.napTimeSec").is_none());
    assert!(observation(&rows, "averageRespirationValue").is_none());
}

#[test]
fn a_failed_read_keeps_the_previous_value_and_the_day_it_was_read() {
    let harness = harness_with_wellness("wellness-failure");
    harness.post("/sync/run", "{\"days\": 1}");

    let before = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");
    let charged = observation(&before, "bodyBatteryChargedValue").expect("the battery is there");
    assert_eq!(charged.value, Some(44.0));
    assert_eq!(charged.retrieved_at, common::instant());

    // The next pull cannot reach the source: the summary is refused.
    harness
        .garmin
        .with_status(USER_SUMMARY_PATH, 503, "the source is away");
    let body = harness.post("/sync/run", "{\"days\": 1}");
    assert!(
        body["failure"].is_object(),
        "the failed read is not a failure: {body}"
    );

    // The value that was read before is still there, with the day it was
    // read: a failed read is not a missing reading.
    let after = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");
    let charged = observation(&after, "bodyBatteryChargedValue").expect("the battery is there");
    assert_eq!(charged.value, Some(44.0));
    assert_eq!(charged.retrieved_at, common::instant());
}

#[test]
fn a_failed_report_read_keeps_the_previous_value_and_says_the_pull_failed() {
    let harness = harness_with_wellness("wellness-report-failure");
    harness.post("/sync/run", "{\"days\": 1}");

    let before = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");
    let charged = observation(&before, "charged").expect("the report is there");
    assert_eq!(charged.value, Some(30.0));
    assert_eq!(charged.retrieved_at, common::instant());

    // The body battery report cannot be reached this time.
    harness
        .garmin
        .with_status(BODY_BATTERY_PATH, 503, "the source is away");
    let body = harness.post("/sync/run", "{\"days\": 1}");
    assert!(
        body["failure"].is_object(),
        "the failed read is not a failure: {body}"
    );

    // The value that was read before is still there, with the day it was
    // read: a failed read is not a missing reading.
    let after = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");
    let charged = observation(&after, "charged").expect("the report is there");
    assert_eq!(charged.value, Some(30.0));
    assert_eq!(charged.retrieved_at, common::instant());
}

#[test]
fn a_failed_report_read_does_not_cost_the_zone_model() {
    let harness = harness_with_wellness("wellness-report-failure-zone");
    harness.garmin.with_json(HEART_RATE_ZONES_PATH, ZONE_MODEL);
    // The body battery report cannot be reached.
    harness
        .garmin
        .with_status(BODY_BATTERY_PATH, 503, "the source is away");

    harness.post("/sync/run", "{\"days\": 1}");

    // The zone model is the athlete's own configuration, and it is read
    // before the reports: a report that cannot be reached does not cost it.
    let model = harness
        .store
        .current_hr_zone_model()
        .expect("the zone model could not be read");
    assert!(model.is_some(), "the zone model was not stored");
}

#[test]
fn the_observations_are_filed_under_the_day_the_response_carried() {
    let harness = harness_with_wellness("wellness-response-date");
    // The source answers with the day before the one that was asked for.
    harness.garmin.with_json(
        USER_SUMMARY_PATH,
        r#"{"summaryDTO": {"calendarDate": "2026-05-09", "restingHeartRate": 47.0}}"#,
    );
    harness.post("/sync/run", "{\"days\": 1}");

    // The summary's readings are under the day the response carried.
    let rows = harness
        .store
        .wellness_observations("2026-05-09")
        .expect("the observations could not be read");
    let resting = observation(&rows, "restingHeartRate").expect("the reading is there");
    assert_eq!(resting.value, Some(47.0));

    // And the day's own row is under that day too.
    let row = harness
        .store
        .wellness_daily("2026-05-09")
        .expect("the row could not be read");
    assert!(row.is_some());
}

#[test]
fn the_sleep_read_is_the_days_own_and_not_the_days_that_was_asked_for() {
    let harness = harness_with_wellness("wellness-response-date-sleep");
    // The sleep answers with a day of its own, and the summary with
    // another.
    harness.garmin.with_json(
        SLEEP_DAILY_PATH,
        r#"{"dailySleep": {"calendarDate": "2026-05-10", "sleepTimeSeconds": 27900.0}}"#,
    );
    harness.garmin.with_json(
        USER_SUMMARY_PATH,
        r#"{"summaryDTO": {"calendarDate": "2026-05-11", "restingHeartRate": 47.0}}"#,
    );
    harness.post("/sync/run", "{\"days\": 1}");

    let sleep_rows = harness
        .store
        .wellness_observations("2026-05-10")
        .expect("the observations could not be read");
    assert!(observation(&sleep_rows, "sleepTimeSeconds").is_some());

    let summary_rows = harness
        .store
        .wellness_observations("2026-05-11")
        .expect("the observations could not be read");
    assert!(observation(&summary_rows, "restingHeartRate").is_some());
}
