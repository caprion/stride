//! The run update: the payload, the checks, and the stored update.
//!
//! **The rig owns the calling; the router owns the rules.** The rig asks
//! the model to write, asks a second model to review, and asks this router
//! what to send it and whether a draft may be kept. Everything that decides
//! something — the payload, the three checks, the prompt — lives in the
//! core, and this module is the seam to it.
//!
//! ## What the payload may carry
//!
//! Metrics only: the run's own facts, the splits, the pace, the heart
//! rate, the time in the athlete's own zones, the run's own mechanics, the
//! model's name and the day it was read. Never the positions, never the
//! athlete's note, never their name, never the plan. The payload route is
//! the answer to "what would be sent", and it says exactly that.
//!
//! ## What may be kept
//!
//! A draft that passes the three checks, and that the reviewer shipped.
//! Anything else keeps nothing: there is no row for a run that has no
//! update, and the answer says why.

use serde_json::Value;
use stride_core::strings;
use stride_core::update::{check_draft, Check, Report};
use stride_store::NewRunUpdate;

use crate::dto::{
    RunUpdate, RunUpdateResponse, UpdateCheck, UpdatePayloadResponse, UpdateRequestResponse,
    UpdateSettingsResponse,
};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;
use crate::update::payload as payload_of;

use super::ok;

/// The roles a model can be asked to play.
const ROLES: [&str; 3] = ["writer", "reviewer", "repair"];

/// The stored update of a run.
///
/// A run without an update has none: the answer says so with a `null`, and
/// not with an empty draft.
pub fn stored(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    if run.is_none() {
        return Err(ApiError::not_found(format!(
            "there is no run with the id {id}"
        )));
    }
    let update = api
        .store()
        .run_update(id)
        .map_err(|error| api.store_error("the run's update could not be read", &error))?;
    let update = match update {
        Some(update) => Some(RunUpdate {
            writer: update.writer,
            reviewer: update.reviewer,
            payload_hash: update.payload_hash,
            draft: update.draft,
            checks: checks_of(&update.checks)?,
            verdict: update.verdict,
            created_at: crate::state::format_instant(update.created_at),
        }),
        None => None,
    };
    let response = RunUpdateResponse {
        activity_id: id,
        update,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The payload of a run: the facts a model may use, and nothing else.
///
/// This is the answer to "what would be sent", and the app shows it before
/// the first send.
pub fn payload(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let Some(payload) = payload_of(api, id)? else {
        let response = UpdatePayloadResponse {
            activity_id: id,
            payload_hash: None,
            payload: None,
            text: String::new(),
            unavailable: Some(no_file()),
            generated_at: api.generated_at(),
        };
        return ok(&response);
    };
    let response = UpdatePayloadResponse {
        activity_id: id,
        payload_hash: Some(payload.hash.clone()),
        payload: Some(payload.payload),
        text: payload.text,
        unavailable: None,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// What the rig should send to a model next.
///
/// The prompt is the core's — the product's own instruction to the model —
/// and it is rendered here, with the payload and, for the reviewer and the
/// repair, the draft and the findings.
pub fn model_request(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let role = request
        .body_str("role")
        .map(|text| text.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if !ROLES.contains(&role.as_str()) {
        return Err(ApiError::bad_request(
            "the role is not one of writer, reviewer or repair",
        ));
    }
    let Some(payload) = payload_of(api, id)? else {
        return Err(ApiError::bad_request(no_file()));
    };

    let draft = request
        .body_str("draft")
        .map(|text| text.to_string())
        .unwrap_or_default();
    // The mechanical findings of the draft, and the reviewer's own findings
    // when there are any. Both go into the prompt.
    let mut findings: Vec<String> = Vec::new();
    if !draft.trim().is_empty() {
        let report = check_draft(&payload.payload, &draft);
        for finding in &report.findings {
            findings.push(format!("{}: {}", finding.check.name(), finding.detail));
        }
    }
    for finding in body_findings(request) {
        findings.push(finding);
    }
    let findings_text = if findings.is_empty() {
        String::new()
    } else {
        findings.join("\n")
    };

    let prompt = match role.as_str() {
        "writer" => strings::render("update.prompt.writer", &[("payload", payload.text())]),
        "reviewer" => strings::render(
            "update.prompt.reviewer",
            &[
                ("payload", payload.text()),
                ("draft", &draft),
                ("findings", &findings_text),
            ],
        ),
        _ => strings::render(
            "update.prompt.repair",
            &[
                ("payload", payload.text()),
                ("draft", &draft),
                ("findings", &findings_text),
            ],
        ),
    };
    let prompt = match prompt {
        Some(prompt) => prompt,
        None => return Err(ApiError::internal("the prompt could not be written")),
    };

    let payload_text = payload.text().to_string();
    let response = UpdateRequestResponse {
        role,
        prompt,
        payload_hash: payload.hash,
        payload: payload_text,
        findings,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Record an update — the rig's second pass, and the last step.
///
/// The draft is **checked again here**, against the payload this router
/// builds now: an update is only kept when it is true to the payload the
/// model was given, and the hash says which payload that was.
pub fn record(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    if run.is_none() {
        return Err(ApiError::not_found(format!(
            "there is no run with the id {id}"
        )));
    }
    let draft = request
        .body_str("draft")
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    if draft.is_empty() {
        return Err(ApiError::bad_request("the draft carries no words"));
    }
    let writer = request
        .body_str("writer")
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    let reviewer = request
        .body_str("reviewer")
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    let verdict = request
        .body_str("verdict")
        .map(|text| text.trim().to_ascii_lowercase())
        .unwrap_or_default();

    let Some(payload) = payload_of(api, id)? else {
        return Err(ApiError::bad_request(no_file()));
    };
    // The payload the model was given is the one this router builds now.
    // When the two differ, the draft was written from something else.
    if let Some(given) = request.body_str("payload_hash") {
        if given.trim() != payload.hash {
            return Err(ApiError::bad_request(
                "the payload is not the one the model was given",
            ));
        }
    }

    let report = check_draft(&payload.payload, &draft);
    let checks = checks_of_report(&report);
    let verdict = if report.passed() {
        verdict
    } else {
        "discard".to_string()
    };

    if !report.passed() || verdict != "ship" {
        // Nothing is kept: a draft that does not pass the checks, and one
        // the reviewer did not ship, keep nothing.
        let response = RecordResponse {
            activity_id: id,
            stored: false,
            update: None,
            checks,
            verdict,
            payload_hash: Some(payload.hash),
            generated_at: api.generated_at(),
        };
        return ok(&response);
    }

    let stored = NewRunUpdate {
        activity_id: id,
        writer: writer.clone(),
        reviewer: reviewer.clone(),
        payload_hash: payload.hash.clone(),
        draft: draft.clone(),
        checks: checks_json(&checks),
        verdict: verdict.clone(),
        created_at: api.now(),
    };
    api.store()
        .write_run_update(&stored)
        .map_err(|error| api.store_error("the update could not be written", &error))?;

    let response = RecordResponse {
        activity_id: id,
        stored: true,
        update: Some(RunUpdate {
            writer,
            reviewer,
            payload_hash: payload.hash.clone(),
            draft,
            checks: checks.clone(),
            verdict,
            created_at: crate::state::format_instant(stored.created_at),
        }),
        checks,
        verdict: "ship".to_string(),
        payload_hash: Some(payload.hash),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The update's own settings.
pub fn settings(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let stored = api
        .store()
        .update_settings(api.now())
        .map_err(|error| api.store_error("the update's settings could not be read", &error))?;
    let response = UpdateSettingsResponse {
        enabled: stored.enabled,
        allow_remote: stored.allow_remote,
        writer_model: stored.writer_model,
        writer_family: stored.writer_family,
        writer_base_url: stored.writer_base_url,
        reviewer_model: stored.reviewer_model,
        reviewer_family: stored.reviewer_family,
        reviewer_base_url: stored.reviewer_base_url,
        key_env: stored.key_env,
        updated_at: crate::state::format_instant(stored.updated_at),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Write the update's own settings.
///
/// The **key is not written here**: the settings name the environment
/// variable it is read from, and the key itself never reaches the
/// database, the export or a log line.
pub fn write_settings(api: &Api, request: &Request) -> ApiResult<Value> {
    let body = match &request.body {
        Some(body) => body.clone(),
        None => return Err(ApiError::bad_request("the settings carry nothing to write")),
    };
    let current = api
        .store()
        .update_settings(api.now())
        .map_err(|error| api.store_error("the update's settings could not be read", &error))?;

    let enabled = boolean_of(&body, "enabled").unwrap_or(current.enabled);
    let allow_remote = boolean_of(&body, "allow_remote").unwrap_or(current.allow_remote);
    let writer_model = text_of(&body, "writer_model").unwrap_or(current.writer_model);
    let writer_family = text_of(&body, "writer_family").unwrap_or(current.writer_family);
    let writer_base_url = text_of(&body, "writer_base_url").unwrap_or(current.writer_base_url);
    let reviewer_model = text_of(&body, "reviewer_model").unwrap_or(current.reviewer_model);
    let reviewer_family = text_of(&body, "reviewer_family").unwrap_or(current.reviewer_family);
    let reviewer_base_url =
        text_of(&body, "reviewer_base_url").unwrap_or(current.reviewer_base_url);
    let key_env = text_of(&body, "key_env").unwrap_or(current.key_env);

    // The reviewer's family has to differ from the writer's: a reviewer of
    // the writer's own family repeats the writer's misreadings.
    if !writer_family.trim().is_empty()
        && writer_family
            .trim()
            .eq_ignore_ascii_case(reviewer_family.trim())
    {
        return Err(ApiError::bad_request(
            "the reviewer's family is the writer's family",
        ));
    }

    let settings = stride_store::UpdateSettings {
        enabled,
        allow_remote,
        writer_model,
        writer_family,
        writer_base_url,
        reviewer_model,
        reviewer_family,
        reviewer_base_url,
        key_env,
        updated_at: api.now(),
    };
    api.store()
        .write_update_settings(&settings)
        .map_err(|error| api.store_error("the update's settings could not be written", &error))?;

    let response = UpdateSettingsResponse {
        enabled: settings.enabled,
        allow_remote: settings.allow_remote,
        writer_model: settings.writer_model,
        writer_family: settings.writer_family,
        writer_base_url: settings.writer_base_url,
        reviewer_model: settings.reviewer_model,
        reviewer_family: settings.reviewer_family,
        reviewer_base_url: settings.reviewer_base_url,
        key_env: settings.key_env,
        updated_at: crate::state::format_instant(settings.updated_at),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The answer to a record: what was kept, and what the checks found.
#[derive(serde::Serialize)]
struct RecordResponse {
    /// The run's id.
    activity_id: i64,
    /// Was the update kept?
    stored: bool,
    /// The update, when it was kept.
    update: Option<RunUpdate>,
    /// The checks and their findings. Empty when the update was kept.
    checks: Vec<UpdateCheck>,
    /// The verdict that was reached.
    verdict: String,
    /// The hash of the payload the draft was checked against.
    payload_hash: Option<String>,
    /// When this answer was produced.
    generated_at: String,
}

/// The checks, as the stored update carries them.
fn checks_of_report(report: &Report) -> Vec<UpdateCheck> {
    Check::ALL
        .iter()
        .map(|check| UpdateCheck {
            check: check.name().to_string(),
            findings: report
                .findings_of(*check)
                .into_iter()
                .map(|finding| finding.detail.clone())
                .collect(),
        })
        .collect()
}

/// The checks, read back out of the store.
fn checks_of(text: &str) -> ApiResult<Vec<UpdateCheck>> {
    let parsed: Vec<UpdateCheck> = serde_json::from_str(text)
        .map_err(|_| ApiError::internal("the update's checks could not be read"))?;
    Ok(parsed)
}

/// The checks, as the store keeps them.
fn checks_json(checks: &[UpdateCheck]) -> String {
    serde_json::to_string(checks).unwrap_or_else(|_| "[]".to_string())
}

/// The findings the reviewer sent with the request.
fn body_findings(request: &Request) -> Vec<String> {
    let Some(Value::Array(items)) = request.body.as_ref().and_then(|body| body.get("findings"))
    else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| item.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect()
}

/// A boolean out of a body.
fn boolean_of(body: &Value, name: &str) -> Option<bool> {
    body.get(name).and_then(|value| value.as_bool())
}

/// A text out of a body.
fn text_of(body: &Value, name: &str) -> Option<String> {
    body.get(name)
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// The core's own words for a run with no file.
fn no_file() -> String {
    strings::text("fingerprint.unavailable")
        .unwrap_or("the run has no file to read")
        .to_string()
}
