// Integration tests are their own crate, so clippy's allow-in-tests
// configuration does not cover them: the expectations below are allowed on
// purpose. The library itself raises no panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The fixture store the tools are tested against.
//!
//! It is the API's own in-memory backend, filled with a small, known history:
//! five runs, their splits, their series, their zones, a fortnight of
//! physiology and a plan week. The derived numbers — the fingerprints, the
//! readiness, the comparison — are put in by hand, because they belong to
//! `stride-core` and this crate must not reimplement them.
//!
//! The day is fixed: every test counts from 2026-10-28.

#![allow(dead_code)]

use std::sync::Arc;

use serde_json::{json, Value};
use stride_api::backend::{
    ActivityRow, LapRow, PhysiologyRow, PlanBlockRow, PlanPhaseRow, PlanSessionRow, PlanWeekRow,
    StreamRow, ZoneRow,
};
use stride_api::mem::MemBackend;
use stride_mcp::{StrideApi, Tools};

/// The day every test counts from.
pub const TODAY: &str = "2026-10-28";

/// The tools over the fixture.
pub fn tools() -> Tools {
    let api: Arc<dyn StrideApi> = Arc::new(stride_api::Api::with_today(
        Arc::new(store()),
        chrono::NaiveDate::parse_from_str(TODAY, "%Y-%m-%d").expect("the fixture's day"),
    ));
    Tools::with_today(
        api,
        chrono::NaiveDate::parse_from_str(TODAY, "%Y-%m-%d").expect("the fixture's day"),
    )
}

/// The fixture itself.
pub fn store() -> MemBackend {
    let store = MemBackend::new();

    // ------------------------------------------------------------ the runs
    store.add_activity(ActivityRow {
        id: 1,
        source: "garmin_personal".into(),
        external_id: "14823947261".into(),
        name: "Morning run".into(),
        sport_type: "running".into(),
        description: Some("Felt strong after the rest week".into()),
        start_time: Some("2026-10-27T06:30:00+00:00".into()),
        local_date: "2026-10-27".into(),
        day_of_week: Some("Tuesday".into()),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_120.0,
        elevation_gain_m: 52.0,
        avg_hr: Some(150.0),
        max_hr: Some(174.0),
        calories: Some(720.0),
        avg_cadence: Some(85.0),
        max_cadence: Some(92.0),
        avg_speed_ms: Some(3.33),
        max_speed_ms: Some(4.4),
        device: Some("Forerunner 265".into()),
        gear: Some("Nova 3".into()),
        is_trail: false,
        is_commute: false,
        running_time_s: Some(2_940.0),
        walking_time_s: Some(45.0),
        stopped_time_s: Some(15.0),
        running_distance_m: Some(9_920.0),
        geo_points: Some(vec![
            [12.4964, 41.9028],
            [12.4971, 41.9040],
            [12.4982, 41.9055],
            [12.4994, 41.9071],
        ]),
        lap_count: 4,
        ..Default::default()
    });
    store.add_activity(ActivityRow {
        id: 2,
        source: "garmin_personal".into(),
        external_id: "14823000000".into(),
        name: "Evening easy".into(),
        sport_type: "running".into(),
        start_time: Some("2026-10-24T17:45:00+00:00".into()),
        local_date: "2026-10-24".into(),
        day_of_week: Some("Saturday".into()),
        distance_m: 8_000.0,
        moving_time_s: 2_700.0,
        elapsed_time_s: 2_760.0,
        elevation_gain_m: 30.0,
        avg_hr: Some(142.0),
        max_hr: Some(160.0),
        avg_cadence: Some(83.0),
        running_time_s: Some(2_640.0),
        walking_time_s: Some(40.0),
        stopped_time_s: Some(20.0),
        running_distance_m: Some(7_960.0),
        lap_count: 3,
        ..Default::default()
    });
    store.add_activity(ActivityRow {
        id: 3,
        source: "garmin_fit".into(),
        external_id: "14790000000".into(),
        name: "Long run".into(),
        sport_type: "trail_running".into(),
        start_time: Some("2026-10-20T07:10:00+00:00".into()),
        local_date: "2026-10-20".into(),
        day_of_week: Some("Tuesday".into()),
        distance_m: 21_000.0,
        moving_time_s: 6_300.0,
        elapsed_time_s: 6_480.0,
        elevation_gain_m: 412.0,
        avg_hr: Some(148.0),
        max_hr: Some(169.0),
        avg_cadence: Some(82.0),
        is_trail: true,
        running_time_s: Some(6_100.0),
        walking_time_s: Some(160.0),
        stopped_time_s: Some(40.0),
        running_distance_m: Some(20_800.0),
        lap_count: 5,
        ..Default::default()
    });
    store.add_activity(ActivityRow {
        id: 4,
        source: "garmin_personal".into(),
        external_id: "14780000000".into(),
        name: "Recovery jog".into(),
        sport_type: "running".into(),
        start_time: Some("2026-10-15T08:00:00+00:00".into()),
        local_date: "2026-10-15".into(),
        day_of_week: Some("Thursday".into()),
        distance_m: 5_000.0,
        moving_time_s: 1_800.0,
        elapsed_time_s: 1_830.0,
        elevation_gain_m: 12.0,
        avg_hr: Some(135.0),
        max_hr: Some(150.0),
        avg_cadence: Some(81.0),
        lap_count: 2,
        ..Default::default()
    });
    // A run that is not a run: the search has to be able to say "all".
    store.add_activity(ActivityRow {
        id: 6,
        source: "garmin_personal".into(),
        external_id: "14800000000".into(),
        name: "Evening ride".into(),
        sport_type: "cycling".into(),
        start_time: Some("2026-10-25T17:30:00+00:00".into()),
        local_date: "2026-10-25".into(),
        day_of_week: Some("Sunday".into()),
        distance_m: 30_000.0,
        moving_time_s: 3_600.0,
        elapsed_time_s: 3_720.0,
        elevation_gain_m: 180.0,
        avg_hr: Some(138.0),
        max_hr: Some(165.0),
        avg_cadence: Some(90.0),
        lap_count: 3,
        ..Default::default()
    });
    store.add_activity(ActivityRow {
        id: 5,
        source: "garmin_personal".into(),
        external_id: "14660000000".into(),
        name: "Autumn race".into(),
        sport_type: "running".into(),
        start_time: Some("2026-09-27T08:05:00+00:00".into()),
        local_date: "2026-09-27".into(),
        day_of_week: Some("Sunday".into()),
        distance_m: 10_000.0,
        moving_time_s: 2_850.0,
        elapsed_time_s: 2_910.0,
        elevation_gain_m: 80.0,
        avg_hr: Some(165.0),
        max_hr: Some(181.0),
        avg_cadence: Some(87.0),
        lap_count: 4,
        ..Default::default()
    });

    // --------------------------------------------------------- the splits
    for (activity, laps) in [
        (1, vec![2_500.0, 2_500.0, 2_500.0, 2_500.0]),
        (2, vec![2_666.7, 2_666.7, 2_666.6]),
        (3, vec![4_200.0, 4_200.0, 4_200.0, 4_200.0, 4_200.0]),
        (4, vec![2_500.0, 2_500.0]),
        (5, vec![2_500.0, 2_500.0, 2_500.0, 2_500.0]),
    ] {
        for (index, distance) in laps.iter().enumerate() {
            let grade = if activity == 1 && index == 1 {
                60.0
            } else {
                10.0
            };
            // The first run climbs on its second split and fades on its last,
            // so the analysis has something to say.
            let climb = if activity == 1 && index == 1 {
                120.0
            } else {
                10.0
            };
            let speed = if activity == 1 {
                [3.33, 3.30, 3.28, 3.10][index.min(3)]
            } else {
                3.33
            };
            store.add_lap(LapRow {
                activity_id: activity,
                lap_index: index as i64 + 1,
                distance_m: *distance,
                duration_s: distance * 3.0,
                avg_speed_ms: Some(speed),
                max_speed_ms: Some(4.4),
                avg_hr: Some(148.0 + index as f64 * 3.0),
                max_hr: Some(170.0 + index as f64),
                avg_cadence: Some(85.0),
                max_cadence: Some(91.0),
                stride_length_m: Some(1.18),
                elevation_gain_m: Some(grade),
                elevation_loss_m: Some(grade / 2.0),
                start_elevation_m: Some(100.0 + index as f64 * 10.0),
                end_elevation_m: Some(100.0 + index as f64 * 10.0 + climb),
                running_distance_m: Some(*distance - 10.0),
                elapsed_s: Some(distance * 3.1),
            });
        }
    }

    // ---------------------------------------------------------- the series
    let kinds = [
        ("hr", "bpm"),
        ("cadence", "spm"),
        ("pace", "s/km"),
        ("distance", "m"),
        ("elevation", "m"),
    ];
    for activity in [1_i64, 2, 3, 4, 5] {
        for (kind, unit) in kinds {
            let count = 300usize;
            let seconds: Vec<f64> = (0..count).map(|index| index as f64).collect();
            let values: Vec<Option<f64>> = (0..count)
                .map(|index| match kind {
                    "hr" => Some(140.0 + (index % 30) as f64),
                    "cadence" => Some(84.0 + (index % 6) as f64),
                    "pace" => Some(300.0 + (index % 20) as f64),
                    "distance" => Some(index as f64 * 33.3),
                    _ => Some(100.0 + (index % 15) as f64),
                })
                .collect();
            store.add_stream(StreamRow {
                activity_id: activity,
                kind: kind.to_string(),
                seconds,
                values,
                label: None,
                unit: Some(unit.to_string()),
            });
        }
    }

    // ----------------------------------------------------------- the zones
    for activity in [1_i64, 2, 3, 4, 5] {
        for (zone, (name, low, high, time)) in [
            (1, ("Recovery", 0.0, 114.0, 600.0)),
            (2, ("Aerobic", 114.0, 137.0, 1_200.0)),
            (3, ("Tempo", 137.0, 152.0, 900.0)),
            (4, ("Threshold", 152.0, 167.0, 300.0)),
            (5, ("Anaerobic", 167.0, 200.0, 60.0)),
        ] {
            store.add_zone(ZoneRow {
                activity_id: activity,
                zone_no: zone,
                name: name.to_string(),
                low_bpm: Some(low),
                high_bpm: Some(high),
                time_s: time,
            });
        }
    }

    // ------------------------------------------------------ the physiology
    for (date, sleep, hrv, resting, status) in [
        ("2026-10-15", 25_200.0, 54.0, 48.0, "BALANCED"),
        ("2026-10-16", 27_000.0, 61.0, 47.0, "BALANCED"),
        ("2026-10-19", 24_300.0, 38.0, 54.0, "LOW"),
        ("2026-10-22", 28_080.0, 66.0, 46.0, "BALANCED"),
        ("2026-10-24", 26_400.0, 58.0, 47.0, "BALANCED"),
        ("2026-10-27", 25_800.0, 52.0, 49.0, "BALANCED"),
        ("2026-10-28", 27_600.0, 63.0, 46.0, "BALANCED"),
    ] {
        store.add_physiology(PhysiologyRow {
            date: date.to_string(),
            sleep_s: Some(sleep),
            sleep_score: Some(78.0),
            deep_sleep_s: Some(sleep * 0.18),
            rem_s: Some(sleep * 0.22),
            light_sleep_s: Some(sleep * 0.55),
            awake_s: Some(1_800.0),
            hrv_ms: Some(hrv),
            hrv_status: Some(status.to_string()),
            resting_hr: Some(resting),
            max_hr: Some(174.0),
            stress_avg: Some(28.0),
            stress_max: Some(72.0),
            body_battery_min: Some(40.0),
            body_battery_avg: Some(70.0),
            body_battery_max: Some(92.0),
            steps: Some(9_400.0),
            training_readiness: Some(74.0),
            acute_load: Some(310.0),
            chronic_load: Some(280.0),
            vo2max: Some(52.0),
            endurance_score: Some(6_400.0),
            running_tolerance: Some(120.0),
            source: Some("garmin_personal".to_string()),
            updated_at: Some(format!("{date}T08:00:00+00:00")),
        });
    }

    // ---------------------------------------------------------- the plan
    store.add_block(PlanBlockRow {
        id: 1,
        name: "Autumn base".into(),
        start_date: "2026-10-01".into(),
        end_date: Some("2026-11-30".into()),
        goal: Some("A comfortable 42 km week".into()),
        status: "active".into(),
        source: "manual".into(),
        notes: None,
    });
    store.add_phase(PlanPhaseRow {
        id: 1,
        block_id: 1,
        name: "Base".into(),
        start_date: Some("2026-10-01".into()),
        end_date: Some("2026-11-30".into()),
        target_weekly_km_min: Some(35.0),
        target_weekly_km_max: Some(45.0),
        target_sessions: Some(4),
        pace_easy_low: Some(330.0),
        pace_easy_high: Some(360.0),
        pace_tempo_low: Some(270.0),
        pace_tempo_high: Some(290.0),
        pace_interval_low: Some(240.0),
        pace_interval_high: Some(260.0),
        pace_long_low: Some(320.0),
        pace_long_high: Some(350.0),
        hr_zone_target: Some(2),
        notes: None,
    });
    store.add_week(PlanWeekRow {
        id: 1,
        phase_id: 1,
        week_number: 4,
        start_date: Some("2026-10-26".into()),
        end_date: Some("2026-11-01".into()),
        target_km: Some(42.0),
        key_workout: Some("Thursday tempo".into()),
        notes: None,
    });
    store.add_session(PlanSessionRow {
        id: 1,
        week_id: Some(1),
        block_id: Some(1),
        date: "2026-10-27".into(),
        session_type: "tempo".into(),
        title: Some("Tempo 3 x 10".into()),
        details: Some("Three times ten minutes at the tempo pace".into()),
        target_distance_m: Some(10_000.0),
        pace_low_s_km: Some(270.0),
        pace_high_s_km: Some(290.0),
        hr_zone_target: Some(3),
        status: "planned".into(),
        source: "manual".into(),
        notes: None,
    });
    store.add_session(PlanSessionRow {
        id: 2,
        week_id: Some(1),
        block_id: Some(1),
        date: "2026-10-29".into(),
        session_type: "easy".into(),
        title: Some("Easy 6 km".into()),
        details: None,
        target_distance_m: Some(6_000.0),
        pace_low_s_km: Some(330.0),
        pace_high_s_km: Some(360.0),
        hr_zone_target: Some(2),
        status: "planned".into(),
        source: "manual".into(),
        notes: None,
    });

    // -------------------------------------------------- the derived numbers
    let fingerprint = |id: i64, date: &str, cadence: f64, decoupling: f64| {
        json!({
            "activity_id": id,
            "local_date": date,
            "cadence_avg": cadence,
            "cadence_cv": 1.8,
            "cadence_drift_pct": 0.9,
            "hr_avg": 150.0,
            "hr_max": 174.0,
            "hr_drift_pct": 4.1,
            "efficiency_first": 2.41,
            "efficiency_last": 2.28,
            "decoupling_pct": decoupling,
            "pace_cv": 3.2,
            "negative_split_pct": 0.0,
            "stride_length_cm": 118.0,
            "form_score": 74.0,
            "running_time_s": 2_940.0,
            "walking_time_s": 45.0,
            "stopped_time_s": 15.0,
            "hr_drift_bpm": 6.0,
            "pace_drift_pct": 2.4,
            "zone_distribution": { "1": 600.0, "2": 1_200.0, "3": 900.0, "4": 300.0, "5": 60.0 },
        })
    };
    store.set_derived("fingerprint:1", fingerprint(1, "2026-10-27", 85.0, 4.1));
    store.set_derived("fingerprint:5", fingerprint(5, "2026-09-27", 87.0, 2.2));
    let fingerprints = json!([
        fingerprint(1, "2026-10-27", 85.0, 4.1),
        fingerprint(2, "2026-10-24", 83.0, 2.9),
        fingerprint(3, "2026-10-20", 82.0, 6.2),
        fingerprint(4, "2026-10-15", 81.0, 1.4),
    ]);
    for key in [
        "fingerprints:2026-07-07:2026-10-28",
        "fingerprints:2026-10-15:2026-10-28",
        "fingerprints:2026-10-01:2026-10-28",
        "fingerprints:2026-09-28:2026-10-25",
    ] {
        store.set_derived(key, fingerprints.clone());
    }

    let readiness = |date: &str, score: f64, grade: &str| {
        json!({
            "date": date,
            "score": score,
            "grade": grade,
            "drivers": [
                { "key": "sleep", "label": "Sleep", "value": 27_600.0, "unit": "s",
                  "baseline": 26_000.0, "delta": 1_600.0, "contribution": 12.0,
                  "detail": "A little more sleep than usual." },
                { "key": "hrv", "label": "HRV", "value": 63.0, "unit": "ms",
                  "baseline": 58.0, "delta": 5.0, "contribution": 7.0,
                  "detail": "Above the baseline." },
                { "key": "resting_hr", "label": "Resting HR", "value": 46.0, "unit": "bpm",
                  "baseline": 48.0, "delta": -2.0, "contribution": 4.0,
                  "detail": "Lower than usual." },
            ],
            "recommendation": "Go for the planned session.",
            "rationale": "Sleep and HRV are both above the baseline.",
        })
    };
    let readiness_range = json!([
        readiness("2026-10-15", 68.0, "good"),
        readiness("2026-10-16", 71.0, "good"),
        readiness("2026-10-19", 44.0, "low"),
        readiness("2026-10-22", 79.0, "peak"),
        readiness("2026-10-24", 73.0, "good"),
        readiness("2026-10-27", 70.0, "good"),
        readiness("2026-10-28", 82.0, "peak"),
    ]);
    for key in [
        "readiness_range:2026-10-15:2026-10-28",
        "readiness_range:2026-10-01:2026-10-28",
        "readiness_range:2026-10-22:2026-10-28",
    ] {
        store.set_derived(key, readiness_range.clone());
    }
    store.set_derived(
        "readiness:2026-10-28",
        readiness("2026-10-28", 82.0, "peak"),
    );
    store.set_derived(
        "baselines:2026-10-28",
        json!({ "sleep_s": 26_000.0, "hrv_ms": 58.0, "resting_hr": 48.0 }),
    );

    store.set_derived(
        "athlete_zones",
        json!([
            { "zone_no": 1, "name": "Recovery", "low_bpm": 0.0, "high_bpm": 114.0 },
            { "zone_no": 2, "name": "Aerobic", "low_bpm": 114.0, "high_bpm": 137.0 },
            { "zone_no": 3, "name": "Tempo", "low_bpm": 137.0, "high_bpm": 152.0 },
            { "zone_no": 4, "name": "Threshold", "low_bpm": 152.0, "high_bpm": 167.0 },
            { "zone_no": 5, "name": "Anaerobic", "low_bpm": 167.0, "high_bpm": 200.0 },
        ]),
    );

    // The plan's own numbers.
    let session = |id: i64,
                   date: &str,
                   kind: &str,
                   title: &str,
                   target: f64,
                   verdict: &str,
                   cause: Option<&str>,
                   activity: Option<Value>| {
        json!({
            "session": {
                "id": id,
                "week_id": 1,
                "block_id": 1,
                "date": date,
                "session_type": kind,
                "title": title,
                "details": null,
                "target_distance_m": target,
                "pace_low_s_km": 270.0,
                "pace_high_s_km": 290.0,
                "hr_zone_target": 3,
                "status": "planned",
                "source": "manual",
                "notes": null,
            },
            "activity": activity,
            "result": {
                "verdict": verdict,
                "distance_hit": verdict == "hit",
                "pace_band_hit": null,
                "distance_delta_m": 0.0,
                "pace_delta_s_km": null,
                "adherence_score": 100.0,
                "cause": cause,
                "cause_confidence": cause.map(|_| 0.6),
            },
        })
    };
    let morning_run = json!({
        "id": 1,
        "source": "garmin_personal",
        "name": "Morning run",
        "sport_type": "running",
        "local_date": "2026-10-27",
        "distance_m": 10_000.0,
        "moving_time_s": 3_000.0,
        "avg_hr": 150.0,
        "elevation_gain_m": 52.0,
    });
    let sessions = json!([
        session(
            1,
            "2026-10-27",
            "tempo",
            "Tempo 3 x 10",
            10_000.0,
            "hit",
            None,
            Some(morning_run),
        ),
        session(
            2,
            "2026-10-29",
            "easy",
            "Easy 6 km",
            6_000.0,
            "missed",
            Some("The session was missed."),
            None,
        ),
    ]);
    store.set_derived("plan_sessions:2026-10-26:2026-11-01", sessions);
    // The other weeks of the period hold no planned session.
    for key in [
        "plan_sessions:2026-09-28:2026-10-04",
        "plan_sessions:2026-10-05:2026-10-11",
        "plan_sessions:2026-10-12:2026-10-18",
        "plan_sessions:2026-10-19:2026-10-25",
    ] {
        store.set_derived(key, json!([]));
    }
    let adherence = json!({
        "overall_pct": 50.0,
        "planned_km": 16.0,
        "actual_km": 10.0,
        "planned_sessions": 2,
        "completed_sessions": 1,
        "hit_sessions": 1,
        "missed_sessions": 1,
        "extra_sessions": 0,
        "by_verdict": { "hit": 1, "missed_pace": 0, "short": 0, "missed": 1, "substituted": 0, "bonus": 0 },
        "rows": [],
    });
    for key in [
        "plan_adherence:2026-10-01:2026-10-28",
        "plan_adherence:2026-10-22:2026-10-28",
        "plan_adherence:2026-10-01:2026-10-27",
    ] {
        store.set_derived(key, adherence.clone());
    }
    let results = json!([
        {
            "id": 1,
            "plan_session_id": 1,
            "activity_id": 1,
            "verdict": "hit",
            "distance_delta_m": 0.0,
            "pace_delta_s_km": null,
            "pace_band_hit": null,
            "distance_hit": true,
            "adherence_score": 100.0,
            "cause": Value::Null,
            "cause_confidence": Value::Null,
            "note": null,
        },
        {
            "id": 2,
            "plan_session_id": 2,
            "activity_id": Value::Null,
            "verdict": "missed",
            "distance_delta_m": Value::Null,
            "pace_delta_s_km": Value::Null,
            "pace_band_hit": Value::Null,
            "distance_hit": false,
            "adherence_score": 0.0,
            "cause": "The session was missed.",
            "cause_confidence": 0.6,
            "note": null,
        },
    ]);
    for key in [
        "plan_results:2026-10-01:2026-10-28",
        "plan_results:2026-10-01:2026-10-27",
    ] {
        store.set_derived(key, results.clone());
    }

    // The comparison.
    let profile = |distance: f64| {
        let mut points: Vec<Value> = Vec::new();
        for index in 0..120 {
            let fraction = index as f64 / 119.0;
            points.push(json!({
                "distance_m": distance * fraction,
                "pct": fraction * 100.0,
                "hr": 145.0 + fraction * 12.0,
                "cadence": 85.0,
                "pace_s_per_km": 300.0 - fraction * 5.0,
                "elevation": 100.0 + fraction * 40.0,
            }));
        }
        points
    };
    let comparison = json!({
        "runs": [
            {
                "id": 1,
                "name": "Morning run",
                "local_date": "2026-10-27",
                "day_of_week": "Tuesday",
                "sport_type": "running",
                "source": "garmin_personal",
                "distance_m": 10_000.0,
                "moving_time_s": 3_000.0,
                "moving_time_label": "50:00",
                "elevation_gain_m": 52.0,
                "avg_hr": 150.0,
                "max_hr": 174.0,
                "calories": 720.0,
                "pace_s_per_km": 300.0,
                "has_geo": true,
                "values": {
                    "distance_m": 10_000.0,
                    "moving_time_s": 3_000.0,
                    "pace_s_per_km": 300.0,
                    "elevation_gain_m": 52.0,
                    "avg_hr": 150.0,
                    "hr_drift_bpm": 6.0,
                    "pace_drift_pct": 2.4,
                    "decoupling_pct": 4.1,
                    "cadence_avg": 85.0,
                    "cadence_cv": 1.8,
                    "efficiency_first": 2.41,
                    "form_score": 74.0,
                    "running_time_s": 2_940.0,
                    "walking_time_s": 45.0,
                    "stopped_time_s": 15.0,
                    "avg_speed_ms": 3.33,
                },
                "profile": profile(10_000.0),
            },
            {
                "id": 5,
                "name": "Autumn race",
                "local_date": "2026-09-27",
                "day_of_week": "Sunday",
                "sport_type": "running",
                "source": "garmin_personal",
                "distance_m": 10_000.0,
                "moving_time_s": 2_850.0,
                "moving_time_label": "47:30",
                "elevation_gain_m": 80.0,
                "avg_hr": 165.0,
                "max_hr": 181.0,
                "calories": 760.0,
                "pace_s_per_km": 285.0,
                "has_geo": false,
                "values": {
                    "distance_m": 10_000.0,
                    "moving_time_s": 2_850.0,
                    "pace_s_per_km": 285.0,
                    "elevation_gain_m": 80.0,
                    "avg_hr": 165.0,
                    "hr_drift_bpm": 9.0,
                    "pace_drift_pct": 5.1,
                    "decoupling_pct": 6.4,
                    "cadence_avg": 87.0,
                    "cadence_cv": 2.2,
                    "efficiency_first": 2.20,
                    "form_score": 68.0,
                    "running_time_s": 2_800.0,
                    "walking_time_s": 30.0,
                    "stopped_time_s": 20.0,
                    "avg_speed_ms": 3.51,
                },
                "profile": profile(10_000.0),
            },
        ],
        "missing": [],
    });
    store.set_derived("compare:1,5", comparison);
    store.set_derived(
        "compare_candidates",
        json!([
            { "id": 1, "name": "Morning run", "local_date": "2026-10-27", "distance_m": 10_000.0 },
            { "id": 5, "name": "Autumn race", "local_date": "2026-09-27", "distance_m": 10_000.0 },
        ]),
    );

    store
}
