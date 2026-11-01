//! The settings: the weekly goal, the pause, and the note on a run.
//!
//! Three inputs change what the app says, and everything else in the
//! settings is plumbing:
//!
//! * **the goal** — a weekly distance. **A blank goal is no goal**: it is
//!   never a zero that reads as a failure.
//! * **the pause** — a start, an end and one of the athlete's own three
//!   reasons. While the pause is on, the adherence is suspended, and the
//!   plan's own answers say so.
//! * **the note** — the only free text the product stores.
//!
//! The three live in the `notes` table (the schema is frozen), and the
//! store owns the writing of them.

use serde_json::Value;
use stride_core::date::Date;
use stride_store::model::note_kind;

use crate::dto::{Notes, Pause, RunNote, Settings};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::{core_date, ok};

/// The reasons a pause may have.
///
/// The athlete's own three — travel, sick, injured. The list is closed: a
/// reason that is not in it is refused, so the app never shows a reason it
/// did not read.
pub const PAUSE_REASONS: [&str; 3] = ["travel", "sick", "injured"];

/// The settings: the goal, the pause, and what they mean now.
pub fn settings(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let goal = goal_of(api)?;
    let pause = pause_of(api, current)?;
    let paused = api
        .store()
        .pause_in_force(&current.to_iso())
        .map_err(|error| api.store_error("the pause could not be read", &error))?
        .is_some();
    let response = Settings {
        goal,
        pause,
        paused,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Write the settings.
///
/// The body carries the goal and the pause. The goal is a number of
/// kilometres — the schema's own unit — or nothing at all, and nothing at
/// all means no goal. The pause is a start, an end and one of the three
/// reasons, or nothing at all, and nothing at all means no pause.
///
/// The answer is the settings as they now are.
pub fn write_settings(api: &Api, request: &Request) -> ApiResult<Value> {
    let body = match &request.body {
        Some(body) => body.clone(),
        None => return Err(ApiError::bad_request("the settings carry nothing to write")),
    };

    let goal_given = body.get("goal_km").is_some();
    let pause_given = body.get("pause").is_some();
    if !goal_given && !pause_given {
        return Err(ApiError::bad_request("the settings carry nothing to write"));
    }

    if goal_given {
        let goal = match body.get("goal_km") {
            None | Some(Value::Null) => None,
            Some(Value::Number(number)) => {
                let value = number
                    .as_f64()
                    .filter(|value| value.is_finite() && *value >= 0.0)
                    .ok_or_else(|| ApiError::bad_request("the goal is not a distance"))?;
                Some(value)
            }
            Some(_) => return Err(ApiError::bad_request("the goal is not a distance")),
        };
        api.store()
            .set_goal(goal, api.now())
            .map_err(|error| api.store_error("the goal could not be written", &error))?;
    }

    if pause_given {
        let pause = match body.get("pause") {
            None | Some(Value::Null) => None,
            Some(Value::Object(map)) => {
                let start = map
                    .get("start_date")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| ApiError::bad_request("the pause has no first day"))?;
                let end = map
                    .get("end_date")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| ApiError::bad_request("the pause has no last day"))?;
                let reason = map
                    .get("reason")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_ascii_lowercase())
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| ApiError::bad_request("the pause has no reason"))?;
                if !PAUSE_REASONS.contains(&reason.as_str()) {
                    return Err(ApiError::bad_request(
                        "the reason for the pause is not one of travel, sick or injured",
                    ));
                }
                let start = core_date(&start)?.to_iso();
                let end = core_date(&end)?.to_iso();
                if end < start {
                    return Err(ApiError::bad_request("the pause ends before it starts"));
                }
                Some((start, end, reason))
            }
            Some(_) => return Err(ApiError::bad_request("the pause could not be read")),
        };
        match pause {
            None => {
                api.store()
                    .set_pause(None, None, None, api.now())
                    .map_err(|error| api.store_error("the pause could not be written", &error))?;
            }
            Some((start, end, reason)) => {
                api.store()
                    .set_pause(Some(&start), Some(&end), Some(&reason), api.now())
                    .map_err(|error| api.store_error("the pause could not be written", &error))?;
            }
        }
    }

    let current = super::today(api)?;
    let response = Settings {
        goal: goal_of(api)?,
        pause: pause_of(api, current)?,
        paused: api
            .store()
            .pause_in_force(&current.to_iso())
            .map_err(|error| api.store_error("the pause could not be read", &error))?
            .is_some(),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The notes.
///
/// With `activity_id`, the note on that run; without it, every note the
/// athlete has written.
pub fn notes(api: &Api, request: &Request) -> ApiResult<Value> {
    let activity_id = request.query.integer("activity_id")?;
    let notes = match activity_id {
        Some(id) => api
            .store()
            .notes_for_activity(id)
            .map_err(|error| api.store_error("the notes could not be read", &error))?,
        None => api
            .store()
            .notes_of_kind(note_kind::NOTE)
            .map_err(|error| api.store_error("the notes could not be read", &error))?,
    };
    let rendered: Vec<RunNote> = notes
        .iter()
        .filter_map(|note| {
            Some(RunNote {
                activity_id: note.activity_id?,
                text: note.text.clone().unwrap_or_default(),
                created_at: crate::state::format_instant(note.created_at),
            })
        })
        .collect();
    let response = Notes {
        notes: rendered,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Write the note on a run.
///
/// A run carries one note: the old one goes and the new one comes. An
/// empty text takes the note away.
pub fn write_note(api: &Api, request: &Request) -> ApiResult<Value> {
    let body = match &request.body {
        Some(body) => body.clone(),
        None => return Err(ApiError::bad_request("the note carries nothing to write")),
    };
    let activity_id = body
        .get("activity_id")
        .and_then(|value| value.as_i64())
        .ok_or_else(|| ApiError::bad_request("the note has no run"))?;
    let exists = api
        .store()
        .activity(activity_id)
        .map_err(|error| api.store_error("the run could not be read", &error))?
        .is_some();
    if !exists {
        return Err(ApiError::not_found(format!(
            "there is no run with the id {activity_id}"
        )));
    }
    let text = body
        .get("text")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    api.store()
        .set_note(activity_id, text.as_deref(), api.now())
        .map_err(|error| api.store_error("the note could not be written", &error))?;

    let written = api
        .store()
        .notes_for_activity(activity_id)
        .map_err(|error| api.store_error("the notes could not be read", &error))?;
    let note = written.iter().find_map(|note| {
        Some(RunNote {
            activity_id: note.activity_id?,
            text: note.text.clone().unwrap_or_default(),
            created_at: crate::state::format_instant(note.created_at),
        })
    });
    let response = Notes {
        notes: note.into_iter().collect(),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The weekly goal, as the core renders it.
///
/// `None` when there is no goal: **a blank means no goal**, and it is
/// never a zero that reads as a failure.
pub fn goal_of(api: &Api) -> ApiResult<Option<crate::dto::RenderedValue>> {
    let goal = api
        .store()
        .goal_km()
        .map_err(|error| api.store_error("the goal could not be read", &error))?;
    match goal {
        None => Ok(None),
        Some(kilometres) => Ok(render::distance(Some(kilometres * 1000.0))),
    }
}

/// The pause, as the settings show it.
///
/// The pause that is on, and the last one when the plan is not paused now.
pub fn pause_of(api: &Api, current: Date) -> ApiResult<Option<Pause>> {
    let in_force = api
        .store()
        .pause_in_force(&current.to_iso())
        .map_err(|error| api.store_error("the pause could not be read", &error))?;
    let note = match in_force {
        Some(note) => Some(note),
        None => api
            .store()
            .pause()
            .map_err(|error| api.store_error("the pause could not be read", &error))?,
    };
    Ok(note.and_then(|note| {
        Some(Pause {
            start_date: note.start_date?,
            end_date: note.end_date?,
            reason: note.text.unwrap_or_default(),
        })
    }))
}

/// The reason the adherence is suspended, when it is.
///
/// The athlete's own word for it: travel, sick or injured.
pub fn suspension_reason(api: &Api, current: Date) -> ApiResult<Option<String>> {
    let note = api
        .store()
        .pause_in_force(&current.to_iso())
        .map_err(|error| api.store_error("the pause could not be read", &error))?;
    Ok(note.and_then(|note| note.text))
}
