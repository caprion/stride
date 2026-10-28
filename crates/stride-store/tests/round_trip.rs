//! The round trip: write an activity with its laps, streams and zones, read it
//! back, and check that it is the same activity.

// An integration test is its own crate, so clippy's allow-in-tests
// configuration does not cover it: the test code opts out here.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{TimeZone, Utc};
use serde_json::json;
use stride_store::{
    Activity, ActivityLap, ActivityStream, Athlete, DailyPhysiology, Gear, HrZone, PlanBlock,
    PlanPhase, PlanResult, PlanSession, PlanWeek, Readiness, Store, SyncRun,
};

#[test]
fn an_activity_with_its_laps_streams_and_zones_survives_the_round_trip() {
    let dir = common::temp_dir("round-trip");
    let key = common::key();
    let store = Store::open_and_migrate(&dir.join("stride.db"), &key).expect("could not open");

    // The activity belongs to the athlete, and the foreign key is enforced.
    let athlete_id = store
        .upsert_athlete(&Athlete {
            name: "Sumit".to_string(),
            ..Athlete::default()
        })
        .expect("could not write the athlete");

    let activity = Activity {
        id: None,
        athlete_id: Some(athlete_id),
        source: "garmin_fit".to_string(),
        external_id: "FIT-9911".to_string(),
        name: "Morning run".to_string(),
        sport_type: "running".to_string(),
        description: Some("a good one".to_string()),
        start_time: Utc.with_ymd_and_hms(2026, 3, 11, 5, 30, 15).unwrap(),
        local_date: "2026-03-11".to_string(),
        day_of_week: Some("Wed".to_string()),
        distance_m: 10_493.3,
        moving_time_s: 3_001.0,
        elapsed_time_s: 3_045.5,
        elevation_gain_m: 87.5,
        avg_hr: Some(152.0),
        max_hr: Some(178.0),
        calories: Some(712.0),
        avg_cadence: Some(84.5),
        max_cadence: Some(97.0),
        avg_speed_ms: Some(3.497),
        max_speed_ms: Some(4.612),
        running_time_s: Some(2_930.0),
        walking_time_s: Some(60.0),
        stopped_time_s: Some(11.5),
        running_distance_m: Some(10_120.4),
        perceived_exertion: Some(4),
        device: Some("Forerunner 265".to_string()),
        gear: Some("Shoes".to_string()),
        is_trail: false,
        is_commute: true,
        polyline: Some(json!(["_p~iF~ps|U_ulLnnqC_mqNvxq`@"])),
        geo_points: Some(json!([[28.6139, 77.2090], [28.6141, 77.2094]])),
        raw: Some(json!({"sport": "running", "n": 2})),
        created_at: Utc.with_ymd_and_hms(2026, 3, 11, 9, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 3, 11, 9, 0, 0).unwrap(),
    };

    let activity_id = store
        .upsert_activity(&activity)
        .expect("could not write the activity");
    assert!(activity_id > 0, "the activity got an id");

    let laps = vec![
        ActivityLap {
            activity_id,
            lap_index: 1,
            distance_m: 1_000.0,
            duration_s: 285.5,
            avg_speed_ms: Some(3.5026),
            max_speed_ms: Some(4.612),
            avg_hr: Some(148.0),
            max_hr: Some(163.0),
            avg_cadence: Some(84.0),
            max_cadence: Some(91.0),
            stride_length_m: Some(1.24),
            elevation_gain_m: Some(12.0),
            elevation_loss_m: Some(9.5),
            start_elevation_m: Some(212.0),
            end_elevation_m: Some(214.5),
            running_distance_m: Some(985.0),
            elapsed_s: Some(287.0),
            ..ActivityLap::default()
        },
        ActivityLap {
            activity_id,
            lap_index: 2,
            distance_m: 1_000.0,
            duration_s: 290.0,
            avg_hr: Some(152.0),
            max_hr: Some(166.0),
            running_distance_m: Some(1_000.0),
            elapsed_s: Some(291.5),
            ..ActivityLap::default()
        },
    ];
    store
        .replace_activity_laps(activity_id, &laps)
        .expect("could not write the laps");

    let streams = vec![
        ActivityStream {
            activity_id,
            kind: "hr".to_string(),
            seconds: json!([0, 1, 2]),
            values: json!([131, 134, 138]),
            label: Some("Heart rate".to_string()),
            unit: Some("bpm".to_string()),
            ..ActivityStream::default()
        },
        ActivityStream {
            activity_id,
            kind: "cadence".to_string(),
            seconds: json!([0, 1, 2]),
            values: json!([82, 84, 85]),
            label: None,
            unit: Some("spm".to_string()),
            ..ActivityStream::default()
        },
    ];
    store
        .replace_activity_streams(activity_id, &streams)
        .expect("could not write the streams");

    let zones = vec![
        HrZone {
            activity_id,
            zone_no: 1,
            name: "Zone 1".to_string(),
            low_bpm: Some(90.0),
            high_bpm: Some(119.0),
            time_s: 320.0,
            ..HrZone::default()
        },
        HrZone {
            activity_id,
            zone_no: 2,
            name: "Zone 2".to_string(),
            low_bpm: Some(120.0),
            high_bpm: Some(139.0),
            time_s: 840.0,
            ..HrZone::default()
        },
    ];
    store
        .replace_hr_zones(activity_id, &zones)
        .expect("could not write the zones");

    // ---- read it back -------------------------------------------------

    let read = store
        .activity(activity_id)
        .expect("could not read the activity")
        .expect("the activity is not there");

    assert_eq!(read.id, Some(activity_id));
    assert_eq!(read.source, "garmin_fit");
    assert_eq!(read.external_id, "FIT-9911");
    assert_eq!(read.name, "Morning run");
    assert_eq!(read.start_time, activity.start_time);
    assert_eq!(read.local_date, "2026-03-11");
    assert_eq!(read.distance_m, 10_493.3);
    assert_eq!(read.moving_time_s, 3_001.0);
    assert_eq!(read.elapsed_time_s, 3_045.5);
    assert_eq!(read.avg_hr, Some(152.0));
    assert_eq!(read.max_hr, Some(178.0));
    assert_eq!(read.running_time_s, Some(2_930.0));
    assert_eq!(read.walking_time_s, Some(60.0));
    assert_eq!(read.stopped_time_s, Some(11.5));
    assert_eq!(read.running_distance_m, Some(10_120.4));
    assert_eq!(read.avg_cadence, Some(84.5));
    assert_eq!(read.device.as_deref(), Some("Forerunner 265"));
    assert!(read.is_commute);
    assert!(!read.is_trail);
    assert_eq!(read.polyline, activity.polyline);
    assert_eq!(read.geo_points, activity.geo_points);
    assert_eq!(read.raw, activity.raw);
    assert_eq!(read.created_at, activity.created_at);
    assert_eq!(read.updated_at, activity.updated_at);

    let read_laps = store
        .activity_laps(activity_id)
        .expect("could not read the laps");
    assert_eq!(read_laps.len(), 2);
    assert_eq!(read_laps[0].lap_index, 1);
    assert_eq!(read_laps[0].distance_m, 1_000.0);
    assert_eq!(read_laps[0].duration_s, 285.5);
    assert_eq!(read_laps[0].avg_hr, Some(148.0));
    assert_eq!(read_laps[0].running_distance_m, Some(985.0));
    assert_eq!(read_laps[0].elapsed_s, Some(287.0));
    assert_eq!(read_laps[1].lap_index, 2);
    assert_eq!(read_laps[1].max_hr, Some(166.0));

    let read_streams = store
        .activity_streams(activity_id)
        .expect("could not read the streams");
    assert_eq!(read_streams.len(), 2);
    let hr = read_streams
        .iter()
        .find(|stream| stream.kind == "hr")
        .expect("the heart-rate stream is missing");
    assert_eq!(hr.values, json!([131, 134, 138]));
    assert_eq!(hr.seconds, json!([0, 1, 2]));
    assert_eq!(hr.unit.as_deref(), Some("bpm"));

    let read_zones = store
        .hr_zones(activity_id)
        .expect("could not read the zones");
    assert_eq!(read_zones.len(), 2);
    assert_eq!(read_zones[0].zone_no, 1);
    assert_eq!(read_zones[0].time_s, 320.0);
    assert_eq!(read_zones[1].zone_no, 2);
    assert_eq!(read_zones[1].time_s, 840.0);
}

#[test]
fn writing_an_activity_twice_updates_it_in_place() {
    let dir = common::temp_dir("idempotent");
    let key = common::key();
    let store = Store::open_and_migrate(&dir.join("stride.db"), &key).expect("could not open");

    let mut activity = Activity {
        source: "garmin".to_string(),
        external_id: "12345".to_string(),
        name: "First name".to_string(),
        distance_m: 5_000.0,
        ..Activity::default()
    };

    let first = store.upsert_activity(&activity).expect("could not write");
    activity.name = "Second name".to_string();
    activity.distance_m = 5_100.0;
    activity.updated_at = Utc.with_ymd_and_hms(2026, 3, 12, 8, 0, 0).unwrap();
    let second = store
        .upsert_activity(&activity)
        .expect("could not write again");

    assert_eq!(first, second, "the same activity kept its id");
    let count = store
        .count_activities(&Default::default())
        .expect("could not count");
    assert_eq!(count, 1, "the re-sync did not duplicate the activity");

    let read = store
        .activity(first)
        .expect("could not read")
        .expect("gone");
    assert_eq!(read.name, "Second name");
    assert_eq!(read.distance_m, 5_100.0);
}

#[test]
fn the_other_tables_survive_the_round_trip_too() {
    let dir = common::temp_dir("other-tables");
    let key = common::key();
    let store = Store::open_and_migrate(&dir.join("stride.db"), &key).expect("could not open");

    let athlete_id = store
        .upsert_athlete(&Athlete {
            name: "Sumit".to_string(),
            email: Some("sumit@example.com".to_string()),
            ..Athlete::default()
        })
        .expect("could not write the athlete");
    let athlete = store
        .athlete(athlete_id)
        .expect("could not read")
        .expect("gone");
    assert_eq!(athlete.name, "Sumit");
    assert_eq!(athlete.email.as_deref(), Some("sumit@example.com"));
    assert_eq!(athlete.timezone, "Asia/Kolkata");

    let physiology = DailyPhysiology {
        date: "2026-03-11".to_string(),
        sleep_s: Some(27_360.0),
        sleep_score: Some(81.0),
        hrv_ms: Some(62.5),
        hrv_status: Some("balanced".to_string()),
        resting_hr: Some(48.0),
        stress_avg: Some(21.5),
        body_battery_min: Some(41.0),
        body_battery_max: Some(88.0),
        steps: Some(11_204.0),
        training_readiness: Some(76.0),
        ..DailyPhysiology::default()
    };
    store
        .upsert_physiology(&physiology)
        .expect("could not write the physiology");
    let read = store
        .physiology_day("2026-03-11")
        .expect("could not read")
        .expect("gone");
    assert_eq!(read.sleep_s, Some(27_360.0));
    assert_eq!(read.hrv_ms, Some(62.5));
    assert_eq!(read.hrv_status.as_deref(), Some("balanced"));
    assert_eq!(read.steps, Some(11_204.0));
    assert_eq!(read.training_readiness, Some(76.0));

    let readiness = Readiness {
        date: "2026-03-11".to_string(),
        score: 78.0,
        grade: "good".to_string(),
        drivers: Some(json!({"hrv": 1.0, "sleep": 0.8})),
        recommendation: Some("Go".to_string()),
        rationale: Some("because".to_string()),
        ..Readiness::default()
    };
    store.upsert_readiness(&readiness).expect("could not write");
    let read = store
        .readiness("2026-03-11")
        .expect("could not read")
        .expect("gone");
    assert_eq!(read.score, 78.0);
    assert_eq!(read.grade, "good");
    assert_eq!(read.drivers, Some(json!({"hrv": 1.0, "sleep": 0.8})));

    let gear_id = store
        .upsert_gear(&Gear {
            uuid: "GEAR-1".to_string(),
            name: "Pegasus 41".to_string(),
            brand: Some("Nike".to_string()),
            distance_m: 120_400.0,
            activity_count: 23,
            first_used: Some("2025-11-02".to_string()),
            last_used: Some("2026-03-11".to_string()),
            ..Gear::default()
        })
        .expect("could not write the gear");
    let gear = store
        .gear_by_uuid("GEAR-1")
        .expect("could not read")
        .expect("gone");
    assert!(gear_id > 0);
    assert_eq!(gear.name, "Pegasus 41");
    assert_eq!(gear.distance_m, 120_400.0);
    assert_eq!(gear.activity_count, 23);
    assert_eq!(gear.retire_at_m, 800_000.0);

    let block_id = store
        .upsert_plan_block(&PlanBlock {
            name: "HM Sub-2:00".to_string(),
            start_date: "2026-03-01".to_string(),
            end_date: Some("2026-04-12".to_string()),
            goal: Some("Sub two".to_string()),
            ..PlanBlock::default()
        })
        .expect("could not write the block");
    let phase_id = store
        .upsert_plan_phase(&PlanPhase {
            block_id,
            name: "Base".to_string(),
            start_date: Some("2026-03-01".to_string()),
            target_weekly_km_min: Some(30.0),
            target_weekly_km_max: Some(45.0),
            target_sessions: Some(4),
            pace_easy_low: Some(330.0),
            pace_easy_high: Some(360.0),
            ..PlanPhase::default()
        })
        .expect("could not write the phase");
    let week_id = store
        .upsert_plan_week(&PlanWeek {
            phase_id,
            week_number: 1,
            start_date: Some("2026-03-02".to_string()),
            end_date: Some("2026-03-08".to_string()),
            target_km: Some(35.0),
            key_workout: Some("8 km tempo".to_string()),
            ..PlanWeek::default()
        })
        .expect("could not write the week");
    let session_id = store
        .upsert_plan_session(&PlanSession {
            week_id: Some(week_id),
            block_id: Some(block_id),
            date: "2026-03-04".to_string(),
            session_type: "tempo".to_string(),
            title: Some("Tempo".to_string()),
            target_distance_m: Some(12_000.0),
            pace_low_s_km: Some(250.0),
            pace_high_s_km: Some(260.0),
            ..PlanSession::default()
        })
        .expect("could not write the session");

    let session = store
        .plan_session(session_id)
        .expect("could not read")
        .expect("gone");
    assert_eq!(session.session_type, "tempo");
    assert_eq!(session.target_distance_m, Some(12_000.0));
    assert_eq!(session.week_id, Some(week_id));

    let blocks = store.plan_blocks().expect("could not read the blocks");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].name, "HM Sub-2:00");
    assert_eq!(store.plan_phases(block_id).expect("phases").len(), 1);
    assert_eq!(store.plan_weeks(phase_id).expect("weeks").len(), 1);
    assert_eq!(
        store
            .plan_sessions("2026-03-01", "2026-03-31")
            .expect("sessions")
            .len(),
        1
    );

    let result_id = store
        .upsert_plan_result(&PlanResult {
            plan_session_id: session_id,
            verdict: "hit".to_string(),
            distance_delta_m: Some(120.0),
            pace_band_hit: Some(true),
            distance_hit: Some(true),
            adherence_score: Some(1.0),
            ..PlanResult::default()
        })
        .expect("could not write the result");
    assert!(result_id > 0);
    let results = store.plan_results(session_id).expect("could not read");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].verdict, "hit");
    assert_eq!(results[0].pace_band_hit, Some(true));

    let sync_id = store
        .upsert_sync_run(&SyncRun {
            source: "garmin_personal".to_string(),
            status: "ok".to_string(),
            days: 30,
            activities_found: 12,
            activities_new: 3,
            activities_updated: 9,
            message: Some("done".to_string()),
            ..SyncRun::default()
        })
        .expect("could not write the sync run");
    assert!(sync_id > 0);
    let runs = store.sync_runs(10).expect("could not read the sync log");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "ok");
    assert_eq!(runs[0].activities_found, 12);
}
