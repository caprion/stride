//! The app routes: the health, the sync, the connection and the calendar.

use chrono::{NaiveDate, NaiveDateTime};
use serde_json::Value;
use stride_core::units;
use stride_store::model::{note_kind, source, sync_kind};
use stride_store::{NewActivity, SyncState};

use crate::dto::{
    AuthConnect, AuthStatus, AuthTest, Calendar, CalendarDay, FitImport, Health, SyncHistory,
    SyncHistoryEntry, SyncRun, SyncSettings, SyncStatus, SyncedRun, SyncedRuns,
};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;

use super::{ok, today};

/// The liveness of the app.
pub fn health(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let store = api.store();
    let activities = store
        .count_activities()
        .map_err(|_| ApiError::internal("the runs could not be counted"))?;
    let wellness_days = store
        .count_wellness_daily()
        .map_err(|_| ApiError::internal("the wellness could not be counted"))?;
    let plan_sessions = store
        .count_plan_sessions()
        .map_err(|_| ApiError::internal("the plan could not be counted"))?;
    let response = Health {
        ok: true,
        service: "stride".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        garmin_connected: api.garmin_connected(),
        activities,
        wellness_days,
        plan_sessions,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The health of the sync window.
pub fn health_daily(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = today(api)?;
    let states = api
        .store()
        .all_sync_state()
        .map_err(|_| ApiError::internal("the sync state could not be read"))?;
    let newest = states
        .iter()
        .filter_map(|state| state.last_success_at)
        .max();
    let stale = match newest {
        None => true,
        Some(instant) => (api.now() - instant).num_hours() > api.config().stale_after_hours,
    };
    let response = crate::dto::HealthDaily {
        date: current.to_iso(),
        generated_at: api.generated_at(),
        stale,
    };
    ok(&response)
}

/// The state of the last sync.
pub fn sync_status(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let store = api.store();
    let states = store
        .all_sync_state()
        .map_err(|_| ApiError::internal("the sync state could not be read"))?;
    let last_sync_at = states
        .iter()
        .filter_map(|state| state.last_success_at)
        .max()
        .map(crate::state::format_instant);
    let activity_count = store
        .count_activities()
        .map_err(|_| ApiError::internal("the runs could not be counted"))?;
    let response = SyncStatus {
        last_sync_at,
        last_error: api.garmin_last_error(),
        activity_count,
    };
    ok(&response)
}

/// A pull from Garmin.
///
/// **One attempt.** The ingest crate performs exactly one token exchange
/// and one HTTP attempt per call, and the router does not retry: a
/// `needs_reconnect` is surfaced as the typed failure it is, and the
/// caller — the phone's pull policy, the desktop's — may make one more
/// attempt (`docs/contract/core-contract.md` §7).
///
/// The key is an argument: it is used for the calls that need it, and it is
/// not returned, not logged and not embedded in an error.
pub fn sync_run(api: &Api, request: &Request) -> ApiResult<Value> {
    let key = key_of(api, request)?;
    let days = match request.body_f64("days")? {
        Some(value) if value > 0.0 => value as i64,
        _ => api.config().sync_days,
    };
    let client = match key {
        Some(key) => api.garmin_client_for(&key),
        None => match api.garmin_client() {
            Some(client) => client,
            None => {
                return Err(ApiError::garmin_rejected(
                    "there is no Garmin key to pull with",
                ))
            }
        },
    };

    let current = today(api)?;
    let to = current.to_iso();
    let from = current.add_days(-days).to_iso();

    let activities = client
        .activities(0, 100, Some(&from), Some(&to))
        .map_err(|error| api.garmin_error(&error))?;
    let list = activities
        .get("activityList")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut pulled = 0usize;
    let mut newest: Option<String> = None;
    for entry in list.iter() {
        let activity_id = match entry
            .get("activityId")
            .and_then(|value| value.as_u64())
            .map(|value| value.to_string())
        {
            Some(id) => id,
            None => continue,
        };
        if newest.is_none() {
            newest = Some(activity_id.clone());
        }
        let blob = match client.activity_fit(&activity_id) {
            Ok(blob) => blob,
            Err(_) => continue,
        };
        let fit = match stride_ingest::extract_fit(&blob, &activity_id) {
            Ok(fit) => fit,
            Err(_) => continue,
        };
        let decoded = match stride_ingest::decode_fit(&fit, "activity.fit") {
            Ok(decoded) => decoded,
            Err(_) => continue,
        };
        let name = entry
            .get("activityName")
            .and_then(|value| value.as_str())
            .map(|text| text.to_string())
            .unwrap_or_else(|| decoded.name.clone());
        let local_date = decoded
            .local_date
            .clone()
            .or_else(|| {
                entry
                    .get("startTimeLocal")
                    .and_then(|value| value.as_str())
                    .map(|text| text.chars().take(10).collect())
            })
            .unwrap_or_else(|| to.clone());
        let start_time = decoded.start_time.unwrap_or(api.now());
        let row = NewActivity {
            source: source::GARMIN_PERSONAL.to_string(),
            external_id: activity_id,
            name,
            sport_type: decoded.sport_type.clone(),
            start_time,
            local_date,
            distance_m: decoded.distance_m,
            moving_time_s: decoded.moving_time_s,
            elapsed_time_s: decoded.elapsed_time_s,
            elevation_gain_m: decoded.elevation_gain_m,
            avg_hr: decoded.avg_hr.map(|rate| rate as f64),
            max_hr: decoded.max_hr.map(|rate| rate as f64),
            fit_blob: fit,
            created_at: api.now(),
            updated_at: api.now(),
        };
        api.store()
            .upsert_activity(&row)
            .map_err(|_| ApiError::internal("the run could not be written"))?;
        pulled += 1;
    }

    let mut wellness_days = 0usize;
    for offset in 0..days {
        let date = current.add_days(-offset).to_iso();
        let row = wellness_of(&client, &date);
        if let Some(row) = row {
            api.store()
                .upsert_wellness_daily(&row)
                .map_err(|_| ApiError::internal("the wellness could not be written"))?;
            wellness_days += 1;
        }
    }

    let finished_at = api.now();
    write_state(api, sync_kind::ACTIVITIES, newest, finished_at)?;
    write_state(api, sync_kind::WELLNESS, None, finished_at)?;
    api.set_garmin_last_error(None);

    let response = SyncRun {
        activities: pulled,
        wellness_days,
        finished_at: crate::state::format_instant(finished_at),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The runs that came from Garmin.
pub fn sync_runs(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = today(api)?;
    let runs = api
        .store()
        .activities_between("0000-01-01", &current.to_iso())
        .map_err(|_| ApiError::internal("the runs could not be read"))?;
    let runs: Vec<SyncedRun> = runs
        .iter()
        .filter(|run| run.source == source::GARMIN_PERSONAL)
        .map(|run| SyncedRun {
            id: run.id,
            name: run.name.clone(),
            local_date: run.local_date.clone(),
            distance_m: run.distance_m,
            updated_at: crate::state::format_instant(run.updated_at),
        })
        .collect();
    let response = SyncedRuns {
        runs,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The sync history: what has been synced, and when.
pub fn sync_history(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let states = api
        .store()
        .all_sync_state()
        .map_err(|_| ApiError::internal("the sync state could not be read"))?;
    let entries: Vec<SyncHistoryEntry> = states
        .iter()
        .map(|state| SyncHistoryEntry {
            kind: state.kind.clone(),
            last_success_at: state.last_success_at.map(crate::state::format_instant),
            token_expires_at: state.token_expires_at.map(crate::state::format_instant),
        })
        .collect();
    let response = SyncHistory {
        entries,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The sync settings.
pub fn sync_settings(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let config = api.config();
    let response = SyncSettings {
        sync_days: config.sync_days,
        history_months: config.history_months,
        health_history_days: config.health_history_days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The state of the connection.
pub fn auth_status(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let states = api
        .store()
        .all_sync_state()
        .map_err(|_| ApiError::internal("the sync state could not be read"))?;
    let last_sync_at = states
        .iter()
        .filter_map(|state| state.last_success_at)
        .max()
        .map(crate::state::format_instant);
    let connected = api.garmin_connected();
    let response = AuthStatus {
        connected,
        method: connected.then(|| "personal_key".to_string()),
        key_source: if connected {
            "personal_key".to_string()
        } else {
            "none".to_string()
        },
        last_sync_at,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Hold the Garmin key.
///
/// The key is an argument: it is held in memory for the length of a pull
/// and dropped after it. It is never returned, never logged and never
/// written down.
pub fn auth_personal_key(api: &Api, request: &Request) -> ApiResult<Value> {
    let key = request
        .body_str("key")
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ApiError::bad_request("there is no key in the request"))?;
    api.set_garmin_key(&key);
    let response = AuthConnect {
        connected: true,
        method: Some("personal_key".to_string()),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Test a Garmin key without holding it.
///
/// One token exchange and one attempt, and then the answer: the key works
/// or it does not. A key that is refused is the typed failure
/// `garmin_rejected`, and a session that has ended is `needs_reconnect`.
pub fn auth_personal_key_test(api: &Api, request: &Request) -> ApiResult<Value> {
    let key = request
        .body_str("key")
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ApiError::bad_request("there is no key in the request"))?;
    let client = api.garmin_client_for(&key);
    match client.profile() {
        Ok(_) => {
            let response = AuthTest {
                ok: true,
                generated_at: api.generated_at(),
            };
            ok(&response)
        }
        Err(error) => Err(api.garmin_error(&error)),
    }
}

/// Import a FIT file.
///
/// The file arrives as bytes — base64 in the body — and is decoded by the
/// ingest crate, which is the only part of Stride that reads a FIT file.
/// A file that cannot be decoded is a bad request, not a run.
pub fn auth_fit_import(api: &Api, request: &Request) -> ApiResult<Value> {
    let encoded = request
        .body_str("fit_base64")
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ApiError::bad_request("there is no file in the request"))?;
    let bytes = decode_base64(&encoded)
        .ok_or_else(|| ApiError::bad_request("the file is not base64"))?;
    if bytes.len() > api.config().max_fit_import_bytes {
        return Err(ApiError::bad_request("the file is larger than the limit"));
    }
    let decoded = stride_ingest::decode_fit(&bytes, "import.fit")
        .map_err(|_| ApiError::bad_request("the file is not a FIT file"))?;
    let default_date = today(api)?.to_iso();
    let local_date = decoded.local_date.clone().unwrap_or(default_date);
    let row = NewActivity {
        source: source::FIT_IMPORT.to_string(),
        external_id: decoded.external_id.clone(),
        name: decoded.name.clone(),
        sport_type: decoded.sport_type.clone(),
        start_time: decoded.start_time.unwrap_or(api.now()),
        local_date,
        distance_m: decoded.distance_m,
        moving_time_s: decoded.moving_time_s,
        elapsed_time_s: decoded.elapsed_time_s,
        elevation_gain_m: decoded.elevation_gain_m,
        avg_hr: decoded.avg_hr.map(|rate| rate as f64),
        max_hr: decoded.max_hr.map(|rate| rate as f64),
        fit_blob: bytes,
        created_at: api.now(),
        updated_at: api.now(),
    };
    let activity_id = api
        .store()
        .upsert_activity(&row)
        .map_err(|_| ApiError::internal("the run could not be written"))?;
    let response = FitImport {
        imported: true,
        activity_id,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Forget the Garmin key.
pub fn auth_logout(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    api.clear_garmin_key();
    let response = AuthConnect {
        connected: false,
        method: None,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The local month calendar: the runs of each day.
pub fn calendar(api: &Api, request: &Request) -> ApiResult<Value> {
    let current = today(api)?;
    let year = match request.query.integer("year")? {
        Some(value) => value as i32,
        None => current.year(),
    };
    let month = match request.query.integer("month")? {
        Some(value) => value as u32,
        None => current.month(),
    };
    let days_in_month = match NaiveDate::from_ymd_opt(year, month, 1) {
        Some(first) => {
            let next = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)
            };
            match next {
                Some(next) => (next - first).num_days() as u32,
                None => 30,
            }
        }
        None => {
            return Err(ApiError::bad_request("the month is not a month"));
        }
    };

    let mut days: Vec<CalendarDay> = Vec::new();
    for day in 1..=days_in_month {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .map(|date| date.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| format!("{year}-{month:02}-{day:02}"));
        let runs = api
            .store()
            .activities_for_date(&date)
            .map_err(|_| ApiError::internal("the runs could not be read"))?;
        let summaries: Vec<crate::dto::ActivitySummary> =
            runs.iter().map(super::activity_summary).collect();
        days.push(CalendarDay {
            date,
            day,
            distance_m: units::round_distance_m(runs.iter().map(|run| run.distance_m).sum()),
            moving_time_s: units::round_time_s(runs.iter().map(|run| run.moving_time_s).sum()),
            activities: summaries,
        });
    }
    let response = Calendar {
        year,
        month,
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The key a pull uses: the body's, or the one the state holds.
fn key_of(api: &Api, request: &Request) -> ApiResult<Option<String>> {
    match request.body_str("key") {
        Some(text) if !text.trim().is_empty() => Ok(Some(text.trim().to_string())),
        _ => {
            let _ = api;
            Ok(None)
        }
    }
}

/// The wellness of one day, as Garmin reports it.
fn wellness_of(
    client: &stride_ingest::GarminClient,
    date: &str,
) -> Option<stride_store::WellnessDaily> {
    let sleep = client.sleep_daily(date).ok()?;
    let hrv = client.hrv_daily(date).ok();
    let summary = client.daily_summary(date).ok();
    let sleep = sleep.get("dailySleep");
    let hrv = hrv
        .as_ref()
        .and_then(|value| value.get("hrvSummary"))
        .and_then(|value| value.get("lastNightAvg"))
        .and_then(|value| value.as_f64());
    let summary = summary
        .as_ref()
        .and_then(|value| value.get("summaryDTO"));
    let resting_hr = summary
        .and_then(|value| value.get("restingHeartRate"))
        .and_then(|value| value.as_f64());
    let stress_avg = summary
        .and_then(|value| value.get("averageStressLevel"))
        .and_then(|value| value.as_f64());
    if sleep.is_none() && hrv.is_none() && resting_hr.is_none() {
        return None;
    }
    Some(stride_store::WellnessDaily {
        date: date.to_string(),
        sleep_s: sleep
            .and_then(|value| value.get("sleepTimeSeconds"))
            .and_then(|value| value.as_f64()),
        deep_sleep_s: sleep
            .and_then(|value| value.get("deepSleepSeconds"))
            .and_then(|value| value.as_f64()),
        rem_s: sleep
            .and_then(|value| value.get("remSleepSeconds"))
            .and_then(|value| value.as_f64()),
        light_sleep_s: sleep
            .and_then(|value| value.get("lightSleepSeconds"))
            .and_then(|value| value.as_f64()),
        awake_s: sleep
            .and_then(|value| value.get("awakeSleepSeconds"))
            .and_then(|value| value.as_f64()),
        hrv_ms: hrv,
        resting_hr,
        stress_avg,
        source: source::GARMIN_PERSONAL.to_string(),
        updated_at: chrono::Utc::now(),
    })
}

/// Write the state of one kind of sync.
fn write_state(
    api: &Api,
    kind: &str,
    cursor: Option<String>,
    succeeded_at: chrono::DateTime<chrono::Utc>,
) -> ApiResult<()> {
    let previous = api
        .store()
        .sync_state(kind)
        .map_err(|_| ApiError::internal("the sync state could not be read"))?;
    let previous_cursor = previous.as_ref().and_then(|state| state.cursor.clone());
    let previous_expiry = previous.and_then(|state| state.token_expires_at);
    let state = SyncState {
        kind: kind.to_string(),
        cursor: cursor.or(previous_cursor),
        last_success_at: Some(succeeded_at),
        token_expires_at: previous_expiry,
        updated_at: succeeded_at,
    };
    api.store()
        .write_sync_state(&state)
        .map_err(|_| ApiError::internal("the sync state could not be written"))
}

/// Decode a base64 document.
///
/// The import route carries a FIT file as text, because a binary body does
/// not survive every host's request parsing. The decoder is the standard
/// one: it takes the alphabet, ignores the padding and refuses anything
/// that is not in the alphabet.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(text.len() / 4 * 3);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for character in text.chars() {
        if character == '=' || character.is_whitespace() {
            continue;
        }
        let value = match character {
            'A'..='Z' => character as u32 - 'A' as u32,
            'a'..='z' => character as u32 - 'a' as u32 + 26,
            '0'..='9' => character as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            _ => return None,
        };
        buffer = (buffer << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

/// The notes of the athlete, read for the settings.
pub fn notes(api: &Api) -> ApiResult<usize> {
    api.store()
        .notes_of_kind(note_kind::NOTE)
        .map(|notes| notes.len())
        .map_err(|_| ApiError::internal("the notes could not be read"))
}

/// The time of the day, as the calendar counts it.
pub fn midnight(date: &str) -> Option<NaiveDateTime> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}
