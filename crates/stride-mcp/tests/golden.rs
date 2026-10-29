//! The golden fixtures, through the tools.
//!
//! The fixtures in `backend/tests/golden/` were recorded from the Python app
//! — the reference implementation — and they hold the reference's *answers*:
//! a readiness score with its drivers, a plan week with its verdicts, causes
//! and confidences, an adherence summary.
//!
//! The domain numbers in them belong to `stride-core`, and this crate does
//! not reimplement the domain: it reads the router's answer and hands it to
//! the harness. So the numbers are fed to the router as they were recorded,
//! exactly as the API's own golden test does, and what is compared here is
//! the layer this crate owns:
//!
//! * the tool hands the reference's numbers back **unchanged** — no
//!   rounding, no reshaping, no dropping;
//! * under the field names a harness relies on;
//! * with the cause and the confidence that belong to the verdict;
//! * and with the range the answer came from.
//!
//! If a tool renumbers a readiness driver, rounds a verdict's pace delta or
//! loses the confidence of a cause, this test fails. That is the whole
//! contract of this crate, checked against the reference.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use serde_json::{json, Value};

use stride_api::backend::{
    ActivityRow, PhysiologyRow, PlanBlockRow, PlanPhaseRow, PlanSessionRow, PlanWeekRow,
};
use stride_api::mem::MemBackend;
use stride_mcp::{StrideApi, Tools};

/// The fixture directory, as the repository holds it.
const FIXTURES: &str = "../../backend/tests/golden";

/// One of the reference's fixtures.
fn fixture(relative: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("the fixture {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("the fixture {relative}: {err}"))
}

/// The tools over a store, counting from a day.
fn tools_over(store: MemBackend, day: &str) -> Tools {
    let date = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").expect("the day");
    let api: Arc<dyn StrideApi> = Arc::new(stride_api::Api::with_today(Arc::new(store), date));
    Tools::with_today(api, date)
}

/// Compare two numbers the way the contract says: 1e-9 relative.
fn close(left: f64, right: f64) -> bool {
    let scale = left.abs().max(right.abs()).max(1.0);
    (left - right).abs() <= 1e-9 * scale
}

/// Compare two JSON values.
///
/// Numbers are compared as numbers — the fixture writes `27900` and the
/// answer writes `27900.0`, and they are the same measurement. Everything
/// else is compared exactly.
#[track_caller]
fn same(got: &Value, wanted: &Value, what: &str) {
    match (got.as_f64(), wanted.as_f64()) {
        (Some(got), Some(wanted)) => assert!(close(got, wanted), "{what}: {got} against {wanted}"),
        _ => assert_eq!(got, wanted, "{what}"),
    }
}

/// The contribution of a driver.
fn contribution(driver: &Value) -> f64 {
    driver["contribution"].as_f64().unwrap_or(0.0)
}

/// The readiness cases, one by one.
#[test]
fn the_readiness_golden_cases_are_handed_back_unchanged() {
    let scenarios = fixture("scenarios/readiness.json");
    let golden = fixture("golden/readiness.json");

    let cases = scenarios["scenarios"].as_array().expect("the scenarios");
    assert!(!cases.is_empty(), "the fixture holds no case");

    for scenario in cases {
        let id = scenario["id"].as_str().expect("the case id");
        let day = scenario["day"].as_str().expect("the case day");
        let expected = golden["scenarios"]
            .as_array()
            .expect("the golden cases")
            .iter()
            .find(|case| case["id"].as_str() == Some(id))
            .unwrap_or_else(|| panic!("the golden case {id}"))["readiness"]
            .clone();

        let store = MemBackend::new();
        for row in scenario["physiology"].as_array().expect("the rows") {
            store.add_physiology(
                serde_json::from_value::<PhysiologyRow>(row.clone()).expect("a row"),
            );
        }
        // The recorded answer, fed to the router as it was recorded.
        store.set_derived(
            &format!("readiness_range:{day}:{day}"),
            json!([expected.clone()]),
        );
        store.set_derived(
            &format!("baselines:{day}"),
            golden["scenarios"]
                .as_array()
                .expect("the golden cases")
                .iter()
                .find(|case| case["id"].as_str() == Some(id))
                .expect("the golden case")["baselines"]
                .clone(),
        );

        let answer = tools_over(store, day)
            .call("physiology_context", &json!({ "from": day, "to": day }))
            .unwrap_or_else(|err| panic!("the tool for {id}: {err}"));

        let got = &answer["data"]["days"][0];
        assert_eq!(
            got["readiness"]["score"], expected["score"],
            "the readiness score of {id}"
        );
        assert_eq!(
            got["readiness"]["grade"], expected["grade"],
            "the readiness grade of {id}"
        );

        // The drivers: the three that matter most, in the reference's order
        // once they are sorted by their contribution.
        let mut wanted = expected["drivers"].as_array().expect("the drivers").clone();
        wanted.sort_by(|left, right| {
            contribution(right)
                .partial_cmp(&contribution(left))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        wanted.truncate(3);
        let mine = got["readiness"]["drivers"].as_array().expect("the drivers");
        assert_eq!(mine.len(), wanted.len(), "the drivers of {id}");
        for (mine, wanted) in mine.iter().zip(wanted.iter()) {
            assert_eq!(mine["key"], wanted["key"], "the driver key of {id}");
            assert_eq!(mine["label"], wanted["label"], "the driver label of {id}");
            assert_eq!(mine["unit"], wanted["unit"], "the driver unit of {id}");
            assert_eq!(
                mine["baseline"], wanted["baseline"],
                "the driver baseline of {id}"
            );
            assert_eq!(mine["delta"], wanted["delta"], "the driver delta of {id}");
            assert_eq!(
                mine["contribution"], wanted["contribution"],
                "the driver contribution of {id}"
            );
            let value = mine["value"].as_f64().expect("the driver value");
            let reference = wanted["value"].as_f64().expect("the driver value");
            assert!(
                close(value, reference),
                "the driver value of {id}: {value} against {reference}"
            );
        }

        // A day with no measurement at all: the score is null and the grade
        // is unknown. Nothing is invented to fill the gap.
        let row = scenario["physiology"]
            .as_array()
            .expect("the rows")
            .iter()
            .find(|row| row["date"].as_str() == Some(day))
            .cloned();
        let Some(row) = row else {
            assert!(
                got["readiness"]["score"].is_null(),
                "the score of {id} is null, not zero"
            );
            assert_eq!(
                got["readiness"]["grade"], expected["grade"],
                "the grade of {id}"
            );
            assert!(
                got["readiness"]["drivers"]
                    .as_array()
                    .expect("the drivers")
                    .is_empty(),
                "the drivers of {id}"
            );
            continue;
        };

        // The measurements of the day itself, and the baseline the reference
        // recorded for it.
        same(
            &got["sleep"]["sleep_s"],
            &row["sleep_s"],
            &format!("the sleep of {id}"),
        );
        same(
            &got["hrv"]["value"],
            &row["hrv_ms"],
            &format!("the HRV of {id}"),
        );
        assert_eq!(
            got["hrv"]["baseline"],
            golden["scenarios"]
                .as_array()
                .expect("the golden cases")
                .iter()
                .find(|case| case["id"].as_str() == Some(id))
                .expect("the golden case")["baselines"]["hrv_ms"],
            "the HRV baseline of {id}"
        );
    }
}

/// The plan case: the verdicts, the causes and the adherence.
#[test]
fn the_plan_golden_case_is_handed_back_unchanged() {
    let scenario = fixture("scenarios/plan.json");
    let golden = fixture("golden/plan.json");

    let store = MemBackend::new();
    store.add_block(PlanBlockRow {
        id: 1,
        name: scenario["block"]["name"].as_str().unwrap_or("").to_string(),
        start_date: scenario["block"]["start_date"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        end_date: scenario["block"]["end_date"].as_str().map(str::to_string),
        status: scenario["block"]["status"]
            .as_str()
            .unwrap_or("active")
            .to_string(),
        ..Default::default()
    });
    store.add_phase(PlanPhaseRow {
        id: 1,
        block_id: 1,
        name: scenario["phase"]["name"].as_str().unwrap_or("").to_string(),
        start_date: scenario["phase"]["start_date"].as_str().map(str::to_string),
        end_date: scenario["phase"]["end_date"].as_str().map(str::to_string),
        ..Default::default()
    });
    store.add_week(PlanWeekRow {
        id: 1,
        phase_id: 1,
        week_number: scenario["week"]["week_number"].as_i64().unwrap_or(1),
        start_date: scenario["week"]["start_date"].as_str().map(str::to_string),
        end_date: scenario["week"]["end_date"].as_str().map(str::to_string),
        target_km: scenario["week"]["target_km"].as_f64(),
        ..Default::default()
    });

    let sessions = golden["sessions"].as_array().expect("the sessions");
    for item in sessions {
        let session = &item["session"];
        store.add_session(PlanSessionRow {
            id: session["id"].as_i64().unwrap_or(0),
            week_id: Some(1),
            block_id: Some(1),
            date: session["date"].as_str().unwrap_or("").to_string(),
            session_type: session["session_type"].as_str().unwrap_or("").to_string(),
            title: session["title"].as_str().map(str::to_string),
            details: session["details"].as_str().map(str::to_string),
            target_distance_m: session["target_distance_m"].as_f64(),
            pace_low_s_km: session["pace_low_s_km"].as_f64(),
            pace_high_s_km: session["pace_high_s_km"].as_f64(),
            ..Default::default()
        });
        if let Some(activity) = item["activity"].as_object() {
            let distance = activity["distance_m"].as_f64().unwrap_or(0.0);
            let pace = activity["pace_s_km"].as_f64().unwrap_or(0.0);
            store.add_activity(ActivityRow {
                id: activity["id"].as_i64().unwrap_or(0),
                name: activity["name"].as_str().unwrap_or("").to_string(),
                sport_type: activity["sport_type"]
                    .as_str()
                    .unwrap_or("running")
                    .to_string(),
                local_date: activity["date"].as_str().unwrap_or("").to_string(),
                distance_m: distance,
                moving_time_s: pace * distance / 1000.0,
                ..Default::default()
            });
        }
    }

    // The recorded sessions, fed to the router as they were recorded: the
    // week of the fixture for the week view, the whole range for the
    // results and the adherence.
    let week_start = scenario["week"]["start_date"].as_str().unwrap_or("");
    let week_end = scenario["week"]["end_date"].as_str().unwrap_or("");
    let week_sessions: Vec<Value> = sessions
        .iter()
        .filter(|item| {
            let date = item["session"]["date"].as_str().unwrap_or("");
            date >= week_start && date <= week_end
        })
        .map(|item| {
            json!({
                "session": item["session"],
                "activity": item["activity"],
                "result": {
                    "verdict": item["verdict"],
                    "distance_hit": item["distance_hit"],
                    "pace_band_hit": item["pace_band_hit"],
                    "distance_delta_m": item["distance_delta_m"],
                    "pace_delta_s_km": item["pace_delta_s_km"],
                    "adherence_score": item["adherence_score"],
                    "cause": item["cause"],
                    "cause_confidence": item["cause_confidence"],
                },
            })
        })
        .collect();
    store.set_derived(
        &format!("plan_sessions:{week_start}:{week_end}"),
        Value::Array(week_sessions),
    );

    let range_start = golden["range"][0].as_str().unwrap_or("");
    let range_end = golden["range"][1].as_str().unwrap_or("");
    let results: Vec<Value> = sessions
        .iter()
        .map(|item| {
            json!({
                "id": item["session"]["id"],
                "plan_session_id": item["session"]["id"],
                "activity_id": item["activity"]["id"],
                "verdict": item["verdict"],
                "distance_hit": item["distance_hit"],
                "pace_band_hit": item["pace_band_hit"],
                "distance_delta_m": item["distance_delta_m"],
                "pace_delta_s_km": item["pace_delta_s_km"],
                "adherence_score": item["adherence_score"],
                "cause": item["cause"],
                "cause_confidence": item["cause_confidence"],
            })
        })
        .collect();
    store.set_derived(
        &format!("plan_results:{range_start}:{range_end}"),
        Value::Array(results),
    );
    store.set_derived(
        &format!("plan_adherence:{range_start}:{range_end}"),
        golden["adherence"].clone(),
    );

    let tools = tools_over(store, range_end);

    // ------------------------------------------------------------ the week
    let answer = tools
        .call(
            "plan_context",
            &json!({ "view": "week", "date": week_start }),
        )
        .unwrap_or_else(|err| panic!("the week: {err}"));
    let mine = answer["data"]["week"]["sessions"]
        .as_array()
        .expect("the sessions")
        .clone();
    assert_eq!(
        mine.len(),
        week_sessions_count(sessions, week_start, week_end)
    );

    for item in &mine {
        let id = item["id"].as_i64().expect("the session id");
        let wanted = sessions
            .iter()
            .find(|candidate| candidate["session"]["id"].as_i64() == Some(id))
            .unwrap_or_else(|| panic!("the golden session {id}"));
        assert_eq!(item["verdict"], wanted["verdict"], "the verdict of {id}");
        assert_eq!(item["cause"], wanted["cause"], "the cause of {id}");
        assert_eq!(
            item["cause_confidence"], wanted["cause_confidence"],
            "the cause confidence of {id}"
        );
        assert_eq!(
            item["distance_delta_m"], wanted["distance_delta_m"],
            "the distance delta of {id}"
        );
        let pace = item["pace_delta_s_per_km"].as_f64();
        let reference = wanted["pace_delta_s_km"].as_f64();
        match (pace, reference) {
            (Some(pace), Some(reference)) => assert!(
                close(pace, reference),
                "the pace delta of {id}: {pace} against {reference}"
            ),
            (None, None) => {}
            _ => panic!("the pace delta of {id} is not the reference's"),
        }
    }

    // ---------------------------------------------------------- the results
    let answer = tools
        .call(
            "plan_context",
            &json!({ "view": "results", "from": range_start, "to": range_end }),
        )
        .unwrap_or_else(|err| panic!("the results: {err}"));
    let mine = answer["data"]["results"].as_array().expect("the results");
    assert_eq!(mine.len(), sessions.len());
    for item in mine {
        let id = item["plan_session_id"].as_i64().expect("the session id");
        let wanted = sessions
            .iter()
            .find(|candidate| candidate["session"]["id"].as_i64() == Some(id))
            .unwrap_or_else(|| panic!("the golden session {id}"));
        assert_eq!(item["verdict"], wanted["verdict"], "the verdict of {id}");
        assert_eq!(item["cause"], wanted["cause"], "the cause of {id}");
        assert_eq!(
            item["cause_confidence"], wanted["cause_confidence"],
            "the cause confidence of {id}"
        );
    }

    // --------------------------------------------------------- the adherence
    let answer = tools
        .call(
            "plan_context",
            &json!({ "view": "results", "from": range_start, "to": range_end }),
        )
        .unwrap_or_else(|err| panic!("the results: {err}"));
    let adherence = &answer["data"]["adherence"];
    let wanted = &golden["adherence"];
    same(
        &adherence["overall_pct"],
        &wanted["overall_pct"],
        "the adherence",
    );
    same(
        &adherence["planned_km"],
        &wanted["planned_km"],
        "the planned kilometres",
    );
    same(
        &adherence["actual_km"],
        &wanted["actual_km"],
        "the actual kilometres",
    );
    assert_eq!(adherence["planned_sessions"], wanted["planned_sessions"]);
    assert_eq!(
        adherence["completed_sessions"],
        wanted["completed_sessions"]
    );
    assert_eq!(adherence["hit_sessions"], wanted["hit_sessions"]);
    assert_eq!(adherence["missed_sessions"], wanted["missed_sessions"]);
    assert_eq!(adherence["extra_sessions"], wanted["extra_sessions"]);
}

/// How many of the golden sessions fall in a week.
fn week_sessions_count(sessions: &[Value], from: &str, to: &str) -> usize {
    sessions
        .iter()
        .filter(|item| {
            let date = item["session"]["date"].as_str().unwrap_or("");
            date >= from && date <= to
        })
        .count()
}
