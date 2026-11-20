//! The run update: the payload a model may read, the checks, and the
//! stored update.
//!
//! Every fixture is synthetic: the corpus files are the repository's own
//! sanitised ones, and the zone model carries invented floors.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{DateTime, Utc};
use serde_json::json;
use stride_store::{NewActivity, NewHeartRateZoneModel};

/// The floors of the synthetic model below, in beats per minute.
const FLOORS: [f64; 5] = [106.0, 121.0, 135.0, 149.0, 163.0];

/// A synthetic zone model, with invented numbers.
fn model(read_at: DateTime<Utc>) -> NewHeartRateZoneModel {
    NewHeartRateZoneModel {
        sport: "DEFAULT".to_string(),
        training_method: "HR_MAX".to_string(),
        max_heart_rate: Some(178.0),
        zone1_floor: FLOORS[0],
        zone2_floor: FLOORS[1],
        zone3_floor: FLOORS[2],
        zone4_floor: FLOORS[3],
        zone5_floor: FLOORS[4],
        read_at,
    }
}

/// Write a run whose FIT file is the corpus file named.
fn write_fit_run(harness: &common::Harness, name: &str) -> i64 {
    let fit = common::corpus_fit(name);
    harness.write_run(NewActivity {
        source: "garmin_personal".to_string(),
        external_id: format!("fit-{name}"),
        name: "a run with heart rate".to_string(),
        sport_type: "running".to_string(),
        start_time: common::instant(),
        local_date: "2026-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 0.0,
        avg_hr: Some(150.0),
        max_hr: Some(180.0),
        fit_blob: fit,
        created_at: common::instant(),
        updated_at: common::instant(),
    })
}

/// The settings the rig would have written.
fn write_settings(harness: &common::Harness) {
    let settings = stride_store::UpdateSettings {
        enabled: true,
        allow_remote: false,
        writer_model: "the writer".to_string(),
        writer_family: "one family".to_string(),
        writer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        reviewer_model: "the reviewer".to_string(),
        reviewer_family: "another family".to_string(),
        reviewer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        key_env: "STRIDE_MODEL_KEY".to_string(),
        updated_at: common::instant(),
    };
    harness.store.write_update_settings(&settings).unwrap();
}

#[test]
fn the_payload_carries_the_facts_and_nothing_else() {
    let harness = common::Harness::empty("update-payload");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();
    write_settings(&harness);

    let body = harness.ok_get(&format!("/activities/{id}/update/payload"));

    assert_eq!(body["activity_id"].as_i64(), Some(id));
    assert!(body["unavailable"].is_null());
    let hash = body["payload_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64, "the payload's hash is a SHA-256");

    // The payload is the run's own facts.
    let text = body["text"].as_str().unwrap();
    assert!(
        text.contains("10 km"),
        "the payload has no distance: {text}"
    );
    assert!(
        text.contains("150 bpm"),
        "the payload has no heart rate: {text}"
    );
    assert!(text.contains("Z1"), "the payload has no zones: {text}");

    // And nothing else: no position, no note, no name, no plan.
    assert!(
        !text.contains("latitude") && !text.contains("longitude"),
        "the payload carries a position: {text}"
    );
    let payload = body["payload"].as_object().unwrap();
    for key in payload.keys() {
        assert!(
            !["geo_points", "note", "name", "plan"].contains(&key.as_str()),
            "the payload carries {key}"
        );
    }
}

#[test]
fn a_run_with_no_file_has_no_payload() {
    let harness = common::Harness::empty("update-no-file");
    let id = harness.write_run(common::run("a-run", "2026-05-10", 10_000.0, 3_000.0));

    let body = harness.ok_get(&format!("/activities/{id}/update/payload"));

    assert!(body["payload"].is_null(), "the payload is not there");
    assert!(body["payload_hash"].is_null(), "the hash is not there");
    assert_eq!(
        body["unavailable"].as_str(),
        Some("the run has no file to read")
    );
}

#[test]
fn the_prompt_is_the_core_s_own_instruction() {
    let harness = common::Harness::empty("update-prompt");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();
    write_settings(&harness);

    let body = harness.post(
        &format!("/activities/{id}/update/request"),
        &json!({"role": "writer"}).to_string(),
    );
    let prompt = body["prompt"].as_str().unwrap();
    assert!(prompt.contains("Use only the numbers in the payload"));
    assert!(prompt.contains("no verdict"));
    // The payload travels with it.
    assert!(
        prompt.contains("10 km"),
        "the prompt has no payload: {prompt}"
    );

    // The reviewer's prompt asks for a verdict, and names the three.
    let body = harness.post(
        &format!("/activities/{id}/update/request"),
        &json!({"role": "reviewer", "draft": "The run was 10.0 km."}).to_string(),
    );
    let prompt = body["prompt"].as_str().unwrap();
    assert!(prompt.contains("ship, repair or discard"), "{prompt}");
}

#[test]
fn a_draft_that_fails_the_checks_is_not_kept() {
    let harness = common::Harness::empty("update-refuse");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();
    write_settings(&harness);

    let payload = harness.ok_get(&format!("/activities/{id}/update/payload"));
    let hash = payload["payload_hash"].as_str().unwrap().to_string();

    // The draft says a number the payload does not have.
    let body = harness.post(
        &format!("/activities/{id}/update"),
        &json!({
            "writer": "the writer",
            "reviewer": "the reviewer",
            "verdict": "ship",
            "payload_hash": hash,
            "draft": "The run was 10 km, and it covered 500 m more than usual.",
        })
        .to_string(),
    );

    assert_eq!(body["stored"].as_bool(), Some(false));
    assert_eq!(body["verdict"].as_str(), Some("discard"));
    let findings = body["checks"].as_array().unwrap();
    let grounded = findings
        .iter()
        .find(|check| check["check"] == "groundedness")
        .unwrap();
    let said = grounded["findings"].as_array().unwrap();
    assert_eq!(said.len(), 1, "the number from nowhere was not found");
    assert!(said[0].as_str().unwrap().contains("500"));

    // And nothing was kept.
    let stored = harness.ok_get(&format!("/activities/{id}/update"));
    assert!(stored["update"].is_null(), "the draft was kept");
}

#[test]
fn a_draft_that_passes_is_kept_with_its_checks() {
    let harness = common::Harness::empty("update-keep");
    let id = write_fit_run(&harness, "hr_gps_run.fit");
    harness
        .store
        .write_hr_zone_model_if_new(&model(common::instant()))
        .unwrap();
    write_settings(&harness);

    let payload = harness.ok_get(&format!("/activities/{id}/update/payload"));
    let hash = payload["payload_hash"].as_str().unwrap().to_string();

    let body = harness.post(
        &format!("/activities/{id}/update"),
        &json!({
            "writer": "the writer",
            "reviewer": "the reviewer",
            "verdict": "ship",
            "payload_hash": hash,
            "draft": "The run was 10 km in 50 m, at an average of 150 bpm.",
        })
        .to_string(),
    );
    assert_eq!(body["stored"].as_bool(), Some(true));
    assert_eq!(body["verdict"].as_str(), Some("ship"));

    // The stored update carries its checks and its verdict.
    let stored = harness.ok_get(&format!("/activities/{id}/update"));
    let update = &stored["update"];
    assert_eq!(update["writer"].as_str(), Some("the writer"));
    assert_eq!(update["reviewer"].as_str(), Some("the reviewer"));
    assert_eq!(update["verdict"].as_str(), Some("ship"));
    assert_eq!(update["payload_hash"].as_str(), Some(hash.as_str()));
    let checks = update["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 3, "the three checks are listed");
    for check in checks {
        assert!(check["findings"].as_array().unwrap().is_empty());
    }
}

#[test]
fn the_settings_are_off_until_the_athlete_turns_them_on() {
    let harness = common::Harness::empty("update-settings");

    let body = harness.ok_get("/update/settings");
    assert_eq!(body["enabled"].as_bool(), Some(false));
    assert_eq!(body["allow_remote"].as_bool(), Some(false));
    // The models are named: a season of runs chose them, and the athlete
    // can change every one of them.
    assert_eq!(body["writer_model"].as_str(), Some("deepseek-flash"));
    assert_eq!(
        body["reviewer_model"].as_str(),
        Some("gemini-3.1-flash-lite")
    );
    // And the two are of different families, on purpose.
    assert_ne!(
        body["writer_family"].as_str(),
        body["reviewer_family"].as_str()
    );

    // The reviewer of the writer's own family is refused: a same-family
    // reviewer repeats the writer's misreadings.
    let refused = harness.post(
        "/update/settings",
        &json!({
            "enabled": true,
            "writer_model": "the writer",
            "writer_family": "one family",
            "reviewer_model": "the reviewer",
            "reviewer_family": "one family",
        })
        .to_string(),
    );
    assert_eq!(refused["code"].as_str(), Some("bad_request"));

    // Two families are accepted.
    let body = harness.post(
        "/update/settings",
        &json!({
            "enabled": true,
            "allow_remote": false,
            "writer_model": "the writer",
            "writer_family": "one family",
            "writer_base_url": "http://127.0.0.1:1234/v1",
            "reviewer_model": "the reviewer",
            "reviewer_family": "another family",
            "reviewer_base_url": "http://127.0.0.1:1234/v1",
        })
        .to_string(),
    );
    assert_eq!(body["enabled"].as_bool(), Some(true));
    assert_eq!(body["writer_model"].as_str(), Some("the writer"));
    // The key itself is never in the answer: the settings name the
    // environment variable it is read from.
    assert_eq!(body["key_env"].as_str(), Some("STRIDE_MODEL_KEY"));
}
