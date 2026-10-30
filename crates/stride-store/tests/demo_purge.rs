//! The synthetic demo runs cannot come back — not on the phone, not on the
//! desktop.
//!
//! The demo dataset was removed from the product. What was left behind was
//! the rows an older installation still holds: the synthetic runs, their
//! splits, their streams, their zones and their fingerprints. They were
//! seeded at the athlete's **own** coordinates and their own city, and they
//! look exactly like the athlete's own runs.
//!
//! The Python reference removes them at every start-up
//! (`backend/app/services/persistence.py::purge_synthetic_activities`), and
//! `backend/tests/test_demo_is_gone.py` is the tripwire. The Rust core had no
//! such thing: the only mentions of `demo` were the filters, and those filters
//! are inert because the flag that drove them is always true.
//!
//! So this test is the Rust tripwire: a database that still holds the
//! synthetic rows must come out of `migrate()` without them.

// An integration test is its own crate, so clippy's allow-in-tests
// configuration does not cover it: the test code opts out here.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{TimeZone, Utc};
use stride_store::{Activity, ActivityLap, HrZone, Store};

/// The synthetic run an older installation left behind.
fn synthetic(store: &Store) -> i64 {
    let id = store
        .upsert_activity(&Activity {
            id: None,
            athlete_id: None,
            source: "demo".to_string(),
            external_id: "demo-1".to_string(),
            name: "Sample run".to_string(),
            sport_type: "running".to_string(),
            description: None,
            start_time: Utc.with_ymd_and_hms(2026, 1, 1, 6, 0, 0).unwrap(),
            local_date: "2026-01-01".to_string(),
            day_of_week: Some("Thu".to_string()),
            distance_m: 5_000.0,
            moving_time_s: 1_500.0,
            elapsed_time_s: 1_520.0,
            elevation_gain_m: 10.0,
            avg_hr: Some(140.0),
            max_hr: Some(160.0),
            calories: Some(300.0),
            avg_cadence: Some(80.0),
            max_cadence: Some(90.0),
            avg_speed_ms: Some(3.3),
            max_speed_ms: Some(4.0),
            running_time_s: Some(1_500.0),
            walking_time_s: Some(0.0),
            stopped_time_s: Some(0.0),
            running_distance_m: Some(5_000.0),
            perceived_exertion: None,
            device: Some("Demo".to_string()),
            gear: None,
            is_trail: false,
            is_commute: false,
            polyline: None,
            geo_points: None,
            raw: None,
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 6, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 6, 0, 0).unwrap(),
        })
        .expect("could not write the synthetic activity");
    store
        .replace_activity_laps(
            id,
            &[ActivityLap {
                id: None,
                activity_id: id,
                lap_index: 1,
                distance_m: 5_000.0,
                duration_s: 1_500.0,
                avg_speed_ms: Some(3.3),
                max_speed_ms: Some(4.0),
                avg_hr: Some(140.0),
                max_hr: Some(160.0),
                avg_cadence: Some(80.0),
                max_cadence: Some(90.0),
                stride_length_m: None,
                elevation_gain_m: Some(10.0),
                elevation_loss_m: Some(5.0),
                start_elevation_m: None,
                end_elevation_m: None,
                running_distance_m: Some(5_000.0),
                elapsed_s: Some(1_500.0),
            }],
        )
        .expect("could not write the synthetic lap");
    store
        .replace_hr_zones(
            id,
            &[HrZone {
                id: None,
                activity_id: id,
                zone_no: 1,
                name: "Recovery".to_string(),
                low_bpm: Some(0.0),
                high_bpm: Some(114.0),
                time_s: 1_500.0,
            }],
        )
        .expect("could not write the synthetic zone");
    id
}

/// A real run, which must survive the purge.
fn real(store: &Store) -> i64 {
    store
        .upsert_activity(&Activity {
            id: None,
            athlete_id: None,
            source: "garmin_fit".to_string(),
            external_id: "FIT-1".to_string(),
            name: "Morning run".to_string(),
            sport_type: "running".to_string(),
            description: None,
            start_time: Utc.with_ymd_and_hms(2026, 3, 11, 5, 30, 0).unwrap(),
            local_date: "2026-03-11".to_string(),
            day_of_week: Some("Wed".to_string()),
            distance_m: 10_000.0,
            moving_time_s: 3_000.0,
            elapsed_time_s: 3_100.0,
            elevation_gain_m: 80.0,
            avg_hr: Some(150.0),
            max_hr: Some(175.0),
            calories: Some(700.0),
            avg_cadence: Some(84.0),
            max_cadence: Some(95.0),
            avg_speed_ms: Some(3.3),
            max_speed_ms: Some(4.4),
            running_time_s: Some(2_900.0),
            walking_time_s: Some(80.0),
            stopped_time_s: Some(20.0),
            running_distance_m: Some(9_800.0),
            perceived_exertion: None,
            device: Some("Forerunner 265".to_string()),
            gear: None,
            is_trail: false,
            is_commute: false,
            polyline: None,
            geo_points: None,
            raw: None,
            created_at: Utc.with_ymd_and_hms(2026, 3, 11, 5, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 3, 11, 5, 30, 0).unwrap(),
        })
        .expect("could not write the activity")
}

#[test]
fn the_synthetic_runs_of_an_older_install_are_removed() {
    let dir = common::temp_dir("demo-purge");
    let key = common::key();
    let path = dir.join("stride.db");

    // An installation that was seeded before the demo was removed.
    {
        let store = Store::open_and_migrate(&path, &key).expect("could not open");
        let demo_id = synthetic(&store);
        real(&store);
        assert_eq!(
            store.activities("0000-01-01", "9999-12-31").unwrap().len(),
            2
        );
        assert!(store.activity(demo_id).unwrap().is_some());
    }

    // The next start-up is a migration.
    {
        let store = Store::open_and_migrate(&path, &key).expect("could not reopen");
        let rows = store.activities("0000-01-01", "9999-12-31").unwrap();
        assert_eq!(
            rows.len(),
            1,
            "the synthetic run is still in the feed: {:?}",
            rows.iter().map(|row| row.name.clone()).collect::<Vec<_>>()
        );
        assert!(
            rows.iter().all(|row| row.source != "demo"),
            "a synthetic run survived the migration"
        );
        // The children go with the parent: a lap without its activity is a
        // row nothing reads and nothing can delete.
        assert!(
            store
                .activity_laps(rows[0].id.expect("the row has an id"))
                .unwrap()
                .is_empty(),
            "the synthetic laps are still there"
        );
    }
}

#[test]
fn the_purge_is_idempotent() {
    let dir = common::temp_dir("demo-purge-twice");
    let key = common::key();
    let path = dir.join("stride.db");
    {
        let store = Store::open_and_migrate(&path, &key).expect("could not open");
        synthetic(&store);
    }
    for _ in 0..3 {
        let store = Store::open_and_migrate(&path, &key).expect("could not reopen");
        assert_eq!(
            store.activities("0000-01-01", "9999-12-31").unwrap().len(),
            0
        );
    }
}

#[test]
fn a_real_run_is_never_touched() {
    let dir = common::temp_dir("demo-purge-real");
    let key = common::key();
    let path = dir.join("stride.db");
    let id = {
        let store = Store::open_and_migrate(&path, &key).expect("could not open");
        synthetic(&store);
        real(&store)
    };
    let store = Store::open_and_migrate(&path, &key).expect("could not reopen");
    let row = store.activity(id).unwrap().expect("the real run is gone");
    assert_eq!(row.source, "garmin_fit");
    assert_eq!(row.name, "Morning run");
}
