//! The settings: the weekly goal, the pause, the storage, and the note on
//! a run.
//!
//! Four inputs change what the app says, and everything else in the
//! settings is plumbing:
//!
//! * **the goal** — a weekly distance. **A blank goal is no goal**: it is
//!   never a zero that reads as a failure.
//! * **the pause** — a start, an end and one of the athlete's own three
//!   reasons. While the pause is on, the adherence is suspended, and the
//!   plan's own answers say so.
//! * **the storage** — what is on the device and how big it is.
//! * **the note** — the only free text the product stores.
//!
//! The goal, the pause and the note live in the `notes` table (the schema
//! is frozen, and it holds them already), and the store owns the writing of
//! them.

use serde_json::Value;
use stride_core::date::Date;
use stride_store::model::note_kind;

use crate::dto::{
    NoteResult, SettingsDeleteResult, SettingsExportResult, SettingsGoal, SettingsPause,
    SettingsState, SettingsStorage,
};
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

/// The word the delete takes.
///
/// The typed confirmation is the caller's: the word has to arrive with the
/// request, so a delete cannot happen by accident.
pub const DELETE_CONFIRMATION: &str = "DELETE";

/// The settings: the goal, the pause and the storage.
pub fn settings(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let response = state_of(api, current)?;
    ok(&response)
}

/// Write the settings.
///
/// The body carries the goal and the pause. The goal is a number of
/// kilometres — the schema's own unit — or nothing at all, and nothing at
/// all means no goal. The pause is a start, an end and one of the three
/// reasons (or no reason at all), or nothing at all, and nothing at all
/// means no pause.
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
                    .filter(|text| !text.is_empty());
                if let Some(reason) = reason.as_ref() {
                    if !PAUSE_REASONS.contains(&reason.as_str()) {
                        return Err(ApiError::bad_request(
                            "the reason for the pause is not one of travel, sick or injured",
                        ));
                    }
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
                    .set_pause(Some(&start), Some(&end), reason.as_deref(), api.now())
                    .map_err(|error| api.store_error("the pause could not be written", &error))?;
            }
        }
    }

    let current = super::today(api)?;
    let response = state_of(api, current)?;
    ok(&response)
}

/// Write a backup.
///
/// The export is the store's own: a directory with the database and a
/// manifest, written into the directory the host chose. The answer says
/// that it was written and how big it is.
pub fn export_backup(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let written = crate::routes::export::write_export(api)?;
    let response = SettingsExportResult {
        exported: written.is_some(),
        size: render::bytes(written.map(|export| export.database_bytes)),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Delete everything.
///
/// The word has to travel with the request: the api refuses the call
/// unless it carries the confirmation, so a delete cannot happen by
/// accident.
pub fn delete_everything(api: &Api, request: &Request) -> ApiResult<Value> {
    let body = match &request.body {
        Some(body) => body.clone(),
        None => return Err(ApiError::bad_request("the delete carries no confirmation")),
    };
    let confirmation = body
        .get("confirm")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    if confirmation != DELETE_CONFIRMATION {
        return Err(ApiError::bad_request(
            "the delete did not carry the word that confirms it",
        ));
    }
    api.store()
        .delete_everything()
        .map_err(|error| api.store_error("the data could not be deleted", &error))?;
    let response = SettingsDeleteResult {
        deleted: true,
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
    let activity_id = match request.param_id("id") {
        Ok(id) => id,
        Err(_) => body
            .get("activity_id")
            .and_then(|value| value.as_i64())
            .ok_or_else(|| ApiError::bad_request("the note has no run"))?,
    };
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
    let response = NoteResult {
        note: written
            .iter()
            .find_map(|note| note.text.clone().filter(|text| !text.is_empty())),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The settings as they now are.
fn state_of(api: &Api, current: Date) -> ApiResult<SettingsState> {
    let goal_km = api
        .store()
        .goal_km()
        .map_err(|error| api.store_error("the goal could not be read", &error))?;
    let goal = SettingsGoal {
        value: match goal_km {
            None => None,
            Some(kilometres) => render::distance(Some(kilometres * 1000.0)),
        },
        km: goal_km,
    };
    let pause = pause_of(api, current)?;
    let storage = SettingsStorage {
        size: render::bytes(api.store().size_bytes()),
        activity_count: api
            .store()
            .count_activities()
            .map_err(|error| api.store_error("the runs could not be counted", &error))?,
        wellness_day_count: api
            .store()
            .count_wellness_daily()
            .map_err(|error| api.store_error("the wellness could not be counted", &error))?,
        plan_session_count: api
            .store()
            .count_plan_sessions()
            .map_err(|error| api.store_error("the plan could not be counted", &error))?,
        note_count: api
            .store()
            .count_notes()
            .map_err(|error| api.store_error("the notes could not be counted", &error))?,
    };
    Ok(SettingsState {
        goal,
        pause,
        storage,
        generated_at: api.generated_at(),
    })
}

/// The pause, as the settings show it.
///
/// The pause that is on, and the last one when the plan is not paused now.
pub fn pause_of(api: &Api, current: Date) -> ApiResult<Option<SettingsPause>> {
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
        Some(SettingsPause {
            start_date: note.start_date?,
            end_date: note.end_date?,
            reason: note.text,
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

/// The notes of the athlete, for the routes that read them.
pub fn notes(api: &Api) -> ApiResult<usize> {
    api.store()
        .notes_of_kind(note_kind::NOTE)
        .map(|notes| notes.len())
        .map_err(|error| api.store_error("the notes could not be read", &error))
}
