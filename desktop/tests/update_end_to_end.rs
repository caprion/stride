//! The rig, the router and the core, together: one run's update, written.
//!
//! The other test proves the rig's calling with a fake router. This one
//! uses the **real** router over a real store, with a fake model in place
//! of the two models — so the payload, the prompt, the three checks and
//! the stored update are the ones the product actually writes.
//!
//! Every fixture is synthetic: the corpus file is the repository's own
//! sanitised one.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use stride_api::{Api, Config, SystemClock};
use stride_desktop::api::{ApiRouter, Host, Request, Router};
use stride_desktop::error::Result;
use stride_desktop::key::KeyStore;
use stride_desktop::model::ChatModel;
use stride_desktop::update::{self, ModelSettings};
use stride_store::scratch::ScratchDir;
use stride_store::{DatabaseKey, NewActivity, Store};

/// A model that answers with these answers, in order.
struct ScriptedModel {
    answers: std::sync::Mutex<Vec<String>>,
    calls: std::sync::Mutex<Vec<String>>,
}

impl ScriptedModel {
    fn new(answers: Vec<&str>) -> ScriptedModel {
        ScriptedModel {
            answers: std::sync::Mutex::new(answers.into_iter().map(|a| a.to_string()).collect()),
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl ChatModel for ScriptedModel {
    fn complete(&self, _base_url: &str, model: &str, _prompt: &str) -> Result<String> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(model.to_string());
        }
        let mut answers = match self.answers.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if answers.is_empty() {
            return Ok(String::new());
        }
        Ok(answers.remove(0))
    }
}

/// The host the rig offers the router.
struct TestHost {
    store: Arc<Store>,
    keys: Arc<KeyStore>,
}

impl Host for TestHost {
    fn store(&self) -> Arc<Store> {
        Arc::clone(&self.store)
    }

    fn keys(&self) -> Arc<KeyStore> {
        Arc::clone(&self.keys)
    }
}

/// A run with the corpus's own file.
fn run_with_file() -> NewActivity {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join("fit")
        .join("hr_gps_run.fit");
    let blob = std::fs::read(&path).unwrap_or_else(|error| panic!("the corpus file: {error}"));
    NewActivity {
        source: "garmin_personal".to_string(),
        external_id: "a-run-with-a-file".to_string(),
        name: "a run".to_string(),
        sport_type: "running".to_string(),
        start_time: chrono::Utc::now(),
        local_date: "2026-05-10".to_string(),
        distance_m: 10_000.0,
        moving_time_s: 3_000.0,
        elapsed_time_s: 3_060.0,
        elevation_gain_m: 0.0,
        avg_hr: Some(150.0),
        max_hr: Some(180.0),
        fit_blob: blob,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// The settings the rig would have: on, local, and two families.
fn settings() -> stride_store::UpdateSettings {
    stride_store::UpdateSettings {
        enabled: true,
        allow_remote: false,
        writer_model: "the writer".to_string(),
        writer_family: "one family".to_string(),
        writer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        reviewer_model: "the reviewer".to_string(),
        reviewer_family: "another family".to_string(),
        reviewer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        key_env: "STRIDE_MODEL_KEY".to_string(),
        updated_at: chrono::Utc::now(),
    }
}

#[test]
fn one_run_s_update_is_written_and_kept() {
    let dir: ScratchDir =
        ScratchDir::new("update-end-to-end").unwrap_or_else(|_| panic!("scratch"));
    let key = DatabaseKey::from_text("the database key of a test").unwrap();
    let store = Arc::new(
        Store::open_and_migrate(&dir.file("stride.db"), &key).unwrap_or_else(|_| panic!("store")),
    );
    let id = store
        .upsert_activity(&run_with_file())
        .unwrap_or_else(|_| panic!("the run"));
    store.write_update_settings(&settings()).unwrap();

    // The real router, over the rig's own host.
    let _api = Api::new(store.clone(), Config::default(), Arc::new(SystemClock)).unwrap();
    let router = ApiRouter::over(&TestHost {
        store: store.clone(),
        keys: Arc::new(KeyStore::empty()),
    })
    .unwrap_or_else(|_| panic!("the router"));

    let model = ScriptedModel::new(vec![
        "The run was 10 km in 50 m, at an average of 150 bpm.",
        "ship",
    ]);

    let written = update::write_update(
        &router as &dyn Router,
        &model,
        &ModelSettings::from_answer(&settings_answer(&router)),
        id,
    )
    .unwrap();

    assert_eq!(
        written.draft.as_deref(),
        Some("The run was 10 km in 50 m, at an average of 150 bpm.")
    );
    assert_eq!(written.verdict, "ship");

    // The update is stored, with its checks, its verdict and the hash of
    // the payload it was written from.
    let stored = store.run_update(id).unwrap().expect("the update");
    assert_eq!(stored.writer, "the writer");
    assert_eq!(stored.reviewer, "the reviewer");
    assert_eq!(stored.verdict, "ship");
    assert_eq!(stored.payload_hash.len(), 64);
    assert!(stored.checks.contains("groundedness"));

    // And the router answers with it.
    let answer = router.call(&Request {
        method: "GET",
        path: &format!("/api/v1/activities/{id}/update"),
        query: "",
        body: &[],
    });
    assert_eq!(answer.status, 200);
    let body: serde_json::Value = serde_json::from_slice(&answer.body).unwrap();
    assert_eq!(body["update"]["writer"].as_str(), Some("the writer"));
    assert_eq!(
        body["update"]["checks"].as_array().map(|c| c.len()),
        Some(3)
    );

    // The payload carries the run's own facts and nothing else.
    let answer = router.call(&Request {
        method: "GET",
        path: &format!("/api/v1/activities/{id}/update/payload"),
        query: "",
        body: &[],
    });
    let body: serde_json::Value = serde_json::from_slice(&answer.body).unwrap();
    let text = body["text"].as_str().unwrap_or_default();
    assert!(
        text.contains("10 km"),
        "the payload has no distance: {text}"
    );
    assert!(!text.contains("latitude"), "the payload carries a position");

    drop(dir);
}

/// The settings, as the router answers them.
fn settings_answer(router: &ApiRouter) -> serde_json::Value {
    let answer = router.call(&Request {
        method: "GET",
        path: "/api/v1/update/settings",
        query: "",
        body: &[],
    });
    assert_eq!(answer.status, 200);
    serde_json::from_slice(&answer.body).unwrap_or_default()
}
