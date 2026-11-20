//! Writing a run's update.
//!
//! **The rig owns the calling.** The router owns the rules: it builds the
//! payload, it renders the prompt, it runs the three checks and it decides
//! whether a draft may be kept. The rig asks a model to write, asks a
//! **second** model to review the draft against the payload, and keeps the
//! draft only when the reviewer ships it and the checks pass.
//!
//! ## The order of the calls
//!
//! ```text
//! the payload ──▶ the writer ──▶ the draft
//!                                   │
//!                     the checks ───┤  (the router runs them; the
//!                                   │   findings go to the reviewer)
//!                  the reviewer ◀───┘
//!                         │
//!          ship ──────────┼────────── repair ──▶ the writer again ──▶ the reviewer again
//!           │             │
//!      the update        └── discard: nothing is kept
//!      is kept
//! ```
//!
//! A run's update is written **when the run lands**, not on a timer.
//!
//! ## The rules the rig keeps
//!
//! * **nothing leaves the machine unless the athlete says so** — with
//!   `allow_remote` off, only a loopback base URL may be called;
//! * **the reviewer's family must differ from the writer's** — a reviewer
//!   of the writer's own family repeats the writer's misreadings;
//! * **a refusal is a refusal** — a model that will not answer, or cannot
//!   be reached, keeps nothing;
//! * **a draft that does not pass the checks is not kept** — the router
//!   checks it again when it is recorded, and refuses it.

use serde_json::{json, Value};

use crate::api::{Request, Router};
use crate::error::{Error, Result};
use crate::model::ChatModel;

/// The update's own settings, as the rig reads them.
///
/// The rig reads them out of the router's answer — the same answer the app
/// shows — so that there is one set of settings and not two.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelSettings {
    /// Is the feature on? It is off until the athlete turns it on.
    pub enabled: bool,
    /// May anything leave the machine? It may not, until the athlete says so.
    pub allow_remote: bool,
    /// The writer's model, by name.
    pub writer_model: String,
    /// The writer's family.
    pub writer_family: String,
    /// The base URL the writer is called at.
    pub writer_base_url: String,
    /// The reviewer's model, by name.
    pub reviewer_model: String,
    /// The reviewer's family.
    pub reviewer_family: String,
    /// The base URL the reviewer is called at.
    pub reviewer_base_url: String,
    /// The environment variable the key is read from.
    pub key_env: String,
}

impl ModelSettings {
    /// The settings, as the router's answer carries them.
    pub fn from_answer(answer: &Value) -> ModelSettings {
        ModelSettings {
            enabled: answer["enabled"].as_bool().unwrap_or(false),
            allow_remote: answer["allow_remote"].as_bool().unwrap_or(false),
            writer_model: text_of(answer, "writer_model"),
            writer_family: text_of(answer, "writer_family"),
            writer_base_url: text_of(answer, "writer_base_url"),
            reviewer_model: text_of(answer, "reviewer_model"),
            reviewer_family: text_of(answer, "reviewer_family"),
            reviewer_base_url: text_of(answer, "reviewer_base_url"),
            key_env: {
                let name = text_of(answer, "key_env");
                if name.is_empty() {
                    crate::model::DEFAULT_KEY_ENV.to_string()
                } else {
                    name
                }
            },
        }
    }
}

/// A text out of an answer.
fn text_of(answer: &Value, name: &str) -> String {
    answer[name].as_str().unwrap_or_default().trim().to_string()
}

/// The role the writer plays.
pub const WRITER: &str = "writer";
/// The role the reviewer plays.
pub const REVIEWER: &str = "reviewer";
/// The role the repair plays.
pub const REPAIR: &str = "repair";

/// The verdict the reviewer may reach.
pub const SHIP: &str = "ship";
/// The verdict the reviewer may reach.
pub const REPAIR_VERDICT: &str = "repair";
/// The verdict the reviewer may reach.
pub const DISCARD: &str = "discard";

/// What the rig wrote, and what it found.
#[derive(Clone, Debug, PartialEq)]
pub struct Written {
    /// The draft that was kept, when one was.
    pub draft: Option<String>,
    /// The verdict that was reached.
    pub verdict: String,
    /// The findings the checks and the reviewer produced.
    pub findings: Vec<String>,
}

/// Write the update of one run.
///
/// `Ok(None)` means there is no update: the feature is off, the run has no
/// file to write from, the reviewer discarded the draft, or the draft did
/// not pass the checks. An `Err` means something went wrong — a model that
/// could not be reached, or a router that would not answer — and **nothing
/// is kept** either way.
pub fn write_update(
    router: &dyn Router,
    model: &dyn ChatModel,
    settings: &ModelSettings,
    activity_id: i64,
) -> Result<Written> {
    if !settings.enabled {
        return Ok(Written {
            draft: None,
            verdict: String::new(),
            findings: Vec::new(),
        });
    }
    check_the_rules(settings)?;

    let payload = call(router, "GET", &payload_path(activity_id), None)?;
    let Some(hash) = payload["payload_hash"].as_str() else {
        // No payload: there is nothing to write from.
        return Ok(Written {
            draft: None,
            verdict: String::new(),
            findings: Vec::new(),
        });
    };
    let hash = hash.to_string();

    // The first pass.
    let draft = write_draft(router, model, settings, activity_id, None, &[])?;
    let review = review_draft(router, model, settings, activity_id, &draft)?;

    match review.verdict.as_str() {
        SHIP => {
            let kept = record(router, settings, activity_id, &draft, &hash, SHIP)?;
            Ok(Written {
                draft: kept.then_some(draft),
                verdict: SHIP.to_string(),
                findings: review.findings,
            })
        }
        REPAIR_VERDICT => {
            // A second pass, with the reviewer's findings.
            let second = write_draft(
                router,
                model,
                settings,
                activity_id,
                Some(&draft),
                &review.findings,
            )?;
            let review = review_draft(router, model, settings, activity_id, &second)?;
            if review.verdict != SHIP {
                return Ok(Written {
                    draft: None,
                    verdict: review.verdict,
                    findings: review.findings,
                });
            }
            let kept = record(router, settings, activity_id, &second, &hash, SHIP)?;
            Ok(Written {
                draft: kept.then_some(second),
                verdict: SHIP.to_string(),
                findings: review.findings,
            })
        }
        _ => Ok(Written {
            draft: None,
            verdict: DISCARD.to_string(),
            findings: review.findings,
        }),
    }
}

/// The rules the rig keeps before it calls anything.
fn check_the_rules(settings: &ModelSettings) -> Result<()> {
    if settings.writer_model.trim().is_empty() {
        return Err(Error::config("no writer model has been chosen"));
    }
    if settings.reviewer_model.trim().is_empty() {
        return Err(Error::config("no reviewer model has been chosen"));
    }
    // The reviewer's family must differ from the writer's.
    let writer = settings.writer_family.trim();
    let reviewer = settings.reviewer_family.trim();
    if !writer.is_empty() && writer.eq_ignore_ascii_case(reviewer) {
        return Err(Error::config(
            "the reviewer's family is the writer's family, and a reviewer of the writer's own \
             family repeats the writer's misreadings",
        ));
    }
    // Nothing leaves the machine unless the athlete says so.
    if !settings.allow_remote {
        if !crate::model::is_local(&settings.writer_base_url) {
            return Err(Error::config(
                "the writer's model is not on this machine, and the setting that would allow it \
                 is off",
            ));
        }
        if !crate::model::is_local(&settings.reviewer_base_url) {
            return Err(Error::config(
                "the reviewer's model is not on this machine, and the setting that would allow it \
                 is off",
            ));
        }
    }
    Ok(())
}

/// The first pass, or the second one.
fn write_draft(
    router: &dyn Router,
    model: &dyn ChatModel,
    settings: &ModelSettings,
    activity_id: i64,
    draft: Option<&str>,
    findings: &[String],
) -> Result<String> {
    let role = if draft.is_none() { WRITER } else { REPAIR };
    let mut body = json!({ "role": role });
    if let Some(draft) = draft {
        body["draft"] = Value::String(draft.to_string());
    }
    if !findings.is_empty() {
        body["findings"] = json!(findings);
    }
    let answer = call(router, "POST", &request_path(activity_id), Some(&body))?;
    let prompt = answer["prompt"]
        .as_str()
        .ok_or_else(|| Error::http("the router answered without a prompt"))?
        .to_string();
    model.complete(
        &settings.writer_base_url,
        &settings.writer_model,
        &prompt,
    )
}

/// The reviewer's answer, and what it found.
fn review_draft(
    router: &dyn Router,
    model: &dyn ChatModel,
    settings: &ModelSettings,
    activity_id: i64,
    draft: &str,
) -> Result<Review> {
    let body = json!({ "role": REVIEWER, "draft": draft });
    let answer = call(router, "POST", &request_path(activity_id), Some(&body))?;
    let prompt = answer["prompt"]
        .as_str()
        .ok_or_else(|| Error::http("the router answered without a prompt"))?
        .to_string();
    let mut findings: Vec<String> = answer["findings"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|text| text.to_string())
                .collect()
        })
        .unwrap_or_default();
    let said = model.complete(
        &settings.reviewer_base_url,
        &settings.reviewer_model,
        &prompt,
    )?;
    let (verdict, said) = split_verdict(&said);
    if !said.is_empty() {
        findings.push(said);
    }
    Ok(Review { verdict, findings })
}

/// The reviewer's answer, read.
struct Review {
    /// The verdict: `ship`, `repair` or `discard`.
    verdict: String,
    /// What the reviewer found, and what the checks found.
    findings: Vec<String>,
}

/// The verdict in an answer, and the rest of it.
///
/// The reviewer answers with one word and may name what to correct. An
/// answer that names no verdict is a refusal to ship, and nothing is kept.
fn split_verdict(answer: &str) -> (String, String) {
    let trimmed = answer.trim();
    let lowered = trimmed.to_lowercase();
    for verdict in [SHIP, REPAIR_VERDICT, DISCARD] {
        if lowered.starts_with(verdict) {
            let rest = trimmed[verdict.len()..].trim();
            let rest = rest.trim_start_matches([':', ',', '-', '—', ' ']);
            return (verdict.to_string(), rest.to_string());
        }
    }
    (DISCARD.to_string(), trimmed.to_string())
}

/// Record the update, and say whether it was kept.
///
/// The router checks the draft again against the payload it builds now: an
/// update is only kept when it is true to the payload the model was given.
fn record(
    router: &dyn Router,
    settings: &ModelSettings,
    activity_id: i64,
    draft: &str,
    hash: &str,
    verdict: &str,
) -> Result<bool> {
    let body = json!({
        "writer": settings.writer_model,
        "reviewer": settings.reviewer_model,
        "verdict": verdict,
        "payload_hash": hash,
        "draft": draft,
    });
    let answer = call(router, "POST", &update_path(activity_id), Some(&body))?;
    Ok(answer["stored"].as_bool().unwrap_or(false))
}

/// Ask the router, and read its answer.
fn call(router: &dyn Router, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
    let encoded = body.map(|body| body.to_string()).unwrap_or_default();
    let request = Request {
        method,
        path,
        query: "",
        body: encoded.as_bytes(),
    };
    let answer = router.call(&request);
    let parsed: Value = serde_json::from_slice(&answer.body).unwrap_or(Value::Null);
    if answer.status != 200 {
        let detail = parsed["detail"].as_str().unwrap_or_default();
        return Err(Error::http(if detail.is_empty() {
            format!("the router did not answer {path}")
        } else {
            detail.to_string()
        }));
    }
    Ok(parsed)
}

/// The path of a run's payload.
fn payload_path(activity_id: i64) -> String {
    format!("/api/v1/activities/{activity_id}/update/payload")
}

/// The path of the prompt.
fn request_path(activity_id: i64) -> String {
    format!("/api/v1/activities/{activity_id}/update/request")
}

/// The path of the update itself.
fn update_path(activity_id: i64) -> String {
    format!("/api/v1/activities/{activity_id}/update")
}
