//! The rig's calling: the writer, the reviewer, and the repair.
//!
//! The rig asks a model to write, asks a **second** model to review the
//! draft against the payload, and keeps the draft only when the reviewer
//! ships it and the checks pass. These tests use a fake model and a fake
//! router: no socket is opened and no model is called.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Mutex;

use serde_json::{json, Value};
use stride_desktop::api::{Request, Response, Router};
use stride_desktop::error::Result;
use stride_desktop::model::ChatModel;
use stride_desktop::update::{self, ModelSettings};

/// A model that answers what the test tells it to, and counts the calls.
struct FakeModel {
    answers: Mutex<Vec<String>>,
    calls: Mutex<Vec<String>>,
    refuse: bool,
}

impl FakeModel {
    /// A model that answers with these answers, in order.
    fn new(answers: Vec<&str>) -> FakeModel {
        FakeModel {
            answers: Mutex::new(answers.into_iter().map(|a| a.to_string()).collect()),
            calls: Mutex::new(Vec::new()),
            refuse: false,
        }
    }

    /// A model that refuses every call.
    fn refusing() -> FakeModel {
        FakeModel {
            answers: Mutex::new(Vec::new()),
            calls: Mutex::new(Vec::new()),
            refuse: true,
        }
    }

    /// The models that were asked, in order.
    fn calls(&self) -> Vec<String> {
        self.calls
            .lock()
            .map(|calls| calls.clone())
            .unwrap_or_default()
    }
}

impl ChatModel for FakeModel {
    fn complete(&self, _base_url: &str, model: &str, _prompt: &str) -> Result<String> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(model.to_string());
        }
        if self.refuse {
            return Err(stride_desktop::error::Error::http(
                "the model could not be reached",
            ));
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

/// A router that answers the three routes the rig uses.
struct FakeRouter {
    stored: Mutex<bool>,
    recorded: Mutex<Vec<Value>>,
}

impl FakeRouter {
    /// A router whose record route keeps what it is given.
    fn new(stored: bool) -> FakeRouter {
        FakeRouter {
            stored: Mutex::new(stored),
            recorded: Mutex::new(Vec::new()),
        }
    }

    /// The bodies the record route was given.
    fn recorded(&self) -> Vec<Value> {
        self.recorded.lock().map(|r| r.clone()).unwrap_or_default()
    }
}

impl Router for FakeRouter {
    fn call(&self, request: &Request<'_>) -> Response {
        let path = request.path.to_string();
        if path.ends_with("/update/payload") {
            return Response::json(
                200,
                &json!({
                    "activity_id": 1,
                    "payload_hash": "a-hash",
                    "text": "Distance — 10 km",
                    "unavailable": null,
                })
                .to_string(),
            );
        }
        if path.ends_with("/update/request") {
            let body: Value = serde_json::from_slice(request.body).unwrap_or(json!({}));
            let role = body["role"].as_str().unwrap_or_default().to_string();
            let findings = if role == "reviewer" {
                json!(["groundedness: the number 500 is not in the payload"])
            } else {
                json!([])
            };
            return Response::json(
                200,
                &json!({
                    "role": role,
                    "prompt": format!("the prompt for {role}"),
                    "payload_hash": "a-hash",
                    "payload": "Distance — 10 km",
                    "findings": findings,
                })
                .to_string(),
            );
        }
        if path.ends_with("/update") && request.method == "POST" {
            let body: Value = serde_json::from_slice(request.body).unwrap_or(json!({}));
            if let Ok(mut recorded) = self.recorded.lock() {
                recorded.push(body);
            }
            let stored = match self.stored.lock() {
                Ok(guard) => *guard,
                Err(poisoned) => *poisoned.into_inner(),
            };
            return Response::json(
                200,
                &json!({ "activity_id": 1, "stored": stored, "verdict": "ship" }).to_string(),
            );
        }
        Response::json(404, "{}")
    }
}

/// The settings the rig would have: on, local, and two families.
fn settings() -> ModelSettings {
    ModelSettings {
        enabled: true,
        allow_remote: false,
        writer_model: "the writer".to_string(),
        writer_family: "one family".to_string(),
        writer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        reviewer_model: "the reviewer".to_string(),
        reviewer_family: "another family".to_string(),
        reviewer_base_url: "http://127.0.0.1:1234/v1".to_string(),
        key_env: "STRIDE_MODEL_KEY".to_string(),
    }
}

#[test]
fn the_writer_is_called_once_and_the_reviewer_once() {
    let router = FakeRouter::new(true);
    let model = FakeModel::new(vec!["The run was 10 km in 50 m.", "ship"]);

    let written = update::write_update(&router, &model, &settings(), 1).unwrap();

    assert_eq!(written.draft.as_deref(), Some("The run was 10 km in 50 m."));
    assert_eq!(written.verdict, "ship");
    assert_eq!(
        model.calls(),
        vec!["the writer".to_string(), "the reviewer".to_string()],
        "the writer and the reviewer were each called once"
    );
    // And the update was recorded, with the hash of the payload.
    let recorded = router.recorded();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0]["payload_hash"].as_str(), Some("a-hash"));
    assert_eq!(recorded[0]["verdict"].as_str(), Some("ship"));
}

#[test]
fn a_refusal_is_a_refusal_and_nothing_is_kept() {
    let router = FakeRouter::new(true);
    let model = FakeModel::refusing();

    let refused = update::write_update(&router, &model, &settings(), 1);

    assert!(refused.is_err(), "a refusal was not a refusal");
    assert!(
        router.recorded().is_empty(),
        "a refused draft was recorded anyway"
    );
}

#[test]
fn a_draft_that_fails_the_checks_is_not_kept() {
    let router = FakeRouter::new(false);
    let model = FakeModel::new(vec!["The run was 10 km in 50 m.", "ship"]);

    let written = update::write_update(&router, &model, &settings(), 1).unwrap();

    assert!(
        written.draft.is_none(),
        "a draft that failed the checks was kept"
    );
    // The router was asked to record it, and it refused.
    assert_eq!(router.recorded().len(), 1);
}

#[test]
fn a_repair_is_a_second_pass() {
    let router = FakeRouter::new(true);
    let model = FakeModel::new(vec![
        "The first draft.",
        "repair: the number is not in the payload",
        "The second draft.",
        "ship",
    ]);

    let written = update::write_update(&router, &model, &settings(), 1).unwrap();

    assert_eq!(written.draft.as_deref(), Some("The second draft."));
    assert_eq!(
        model.calls(),
        vec![
            "the writer".to_string(),
            "the reviewer".to_string(),
            "the writer".to_string(),
            "the reviewer".to_string(),
        ],
        "the repair was a second pass"
    );
}

#[test]
fn a_discard_keeps_nothing() {
    let router = FakeRouter::new(true);
    let model = FakeModel::new(vec!["The draft.", "discard"]);

    let written = update::write_update(&router, &model, &settings(), 1).unwrap();

    assert!(written.draft.is_none());
    assert_eq!(written.verdict, "discard");
    assert!(
        router.recorded().is_empty(),
        "a discarded draft was recorded anyway"
    );
}

#[test]
fn a_feature_that_is_off_calls_nothing() {
    let router = FakeRouter::new(true);
    let model = FakeModel::new(vec!["The draft.", "ship"]);
    let mut off = settings();
    off.enabled = false;

    let written = update::write_update(&router, &model, &off, 1).unwrap();

    assert!(written.draft.is_none());
    assert!(model.calls().is_empty(), "a model was called");
}

#[test]
fn nothing_leaves_the_machine_unless_the_athlete_says_so() {
    let router = FakeRouter::new(true);
    let model = FakeModel::new(vec!["The draft.", "ship"]);
    let mut remote = settings();
    remote.writer_base_url = "https://api.openai.com/v1".to_string();

    let refused = update::write_update(&router, &model, &remote, 1);

    assert!(refused.is_err(), "a remote model was called");
    assert!(model.calls().is_empty(), "a model was called");
}

#[test]
fn a_reviewer_of_the_writer_s_own_family_is_refused() {
    let router = FakeRouter::new(true);
    let model = FakeModel::new(vec!["The draft.", "ship"]);
    let mut same = settings();
    same.reviewer_family = same.writer_family.clone();

    let refused = update::write_update(&router, &model, &same, 1);

    assert!(refused.is_err(), "a same-family reviewer was called");
    assert!(model.calls().is_empty(), "a model was called");
}

#[test]
fn the_settings_are_read_out_of_the_router_s_answer() {
    let answer = json!({
        "enabled": true,
        "allow_remote": false,
        "writer_model": "the writer",
        "writer_family": "one family",
        "writer_base_url": "http://127.0.0.1:1234/v1",
        "reviewer_model": "the reviewer",
        "reviewer_family": "another family",
        "reviewer_base_url": "http://127.0.0.1:1234/v1",
        "key_env": "STRIDE_MODEL_KEY",
    });
    let settings = ModelSettings::from_answer(&answer);
    assert!(settings.enabled);
    assert_eq!(settings.writer_model, "the writer");
    assert_eq!(settings.reviewer_family, "another family");
    assert_eq!(settings.key_env, "STRIDE_MODEL_KEY");
}
