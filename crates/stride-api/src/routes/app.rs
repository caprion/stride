//! The app routes: the health, the sync, the connection and the calendar.

use chrono::{NaiveDate, NaiveDateTime};
use serde_json::Value;
use stride_store::model::{note_kind, source, sync_kind};
use stride_store::{NewActivity, SyncState};

use crate::dto::{
    AuthConnect, AuthStatus, AuthTest, Calendar, CalendarDay, FitImport, Health, SyncHistory,
    SyncHistoryEntry, SyncRun, SyncSettings, SyncStatus, SyncedRun, SyncedRuns,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::{core_date, day_label, ok, today};

/// How far apart two starts may be and still be the same run: two minutes.
const DUPLICATE_START_SECONDS: i64 = 120;

/// How far apart two distances may be and still be the same run: two per
/// cent.
const DUPLICATE_DISTANCE: f64 = 0.02;

/// The liveness of the app.
pub fn health(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let store = api.store();
    let activities = store
        .count_activities()
        .map_err(|error| api.store_error("the runs could not be counted", &error))?;
    let wellness_days = store
        .count_wellness_daily()
        .map_err(|error| api.store_error("the wellness could not be counted", &error))?;
    let plan_sessions = store
        .count_plan_sessions()
        .map_err(|error| api.store_error("the plan could not be counted", &error))?;
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
        .map_err(|error| api.store_error("the sync state could not be read", &error))?;
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
        .map_err(|error| api.store_error("the sync state could not be read", &error))?;
    let last_sync_at = states
        .iter()
        .filter_map(|state| state.last_success_at)
        .max()
        .map(crate::state::format_instant);
    let activity_count = store
        .count_activities()
        .map_err(|error| api.store_error("the runs could not be counted", &error))?;
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
    // **Garmin answers the search with a list.** Some of its routes wrap
    // their payload in an object and some do not, so the reading here takes
    // both: a list is a list, and an object with an "activityList" in it
    // is the same list.
    let list = match &activities {
        Value::Array(items) => items.clone(),
        Value::Object(_) => activities
            .get("activityList")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    // **A pull is bounded.** Garmin's Personal API answers a few requests
    // a minute, so a pull that asked for everything would spend the whole
    // limit and come back with half of it. The bound is the configuration's,
    // and the next pull takes the next runs.
    let activity_budget = api.config().max_activities_per_sync;
    let mut pulled = 0usize;
    let mut newest: Option<String> = None;
    let mut attempts = 0usize;
    let mut stopped: Option<String> = None;
    for entry in list.iter() {
        if attempts >= activity_budget {
            break;
        }
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
        attempts += 1;
        let blob = match client.activity_fit(&activity_id) {
            Ok(blob) => blob,
            Err(error) => {
                // A rate limit or a lost connection ends the pull: asking
                // again would spend the limit on the same answer.
                if stops_the_pull(&error) {
                    stopped = Some(error.to_string());
                    break;
                }
                continue;
            }
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
            .map_err(|error| api.store_error("the run could not be written", &error))?;
        pulled += 1;
    }

    // The wellness of the last few days, and no more than the bound: the
    // wellness of a day is three requests, and a year of it is a rate
    // limit spent before anything has landed.
    let wellness_window = days.min(api.config().max_wellness_days_per_sync);
    let mut wellness_days = 0usize;
    for offset in 0..wellness_window {
        let date = current.add_days(-offset).to_iso();
        let row = match wellness_of(&client, &date) {
            Ok(row) => row,
            Err(error) => {
                stopped = Some(error.to_string());
                break;
            }
        };
        if let Some(row) = row {
            api.store()
                .upsert_wellness_daily(&row)
                .map_err(|error| api.store_error("the wellness could not be written", &error))?;
            wellness_days += 1;
        }
    }

    let finished_at = api.now();
    write_state(api, sync_kind::ACTIVITIES, newest, finished_at)?;
    write_state(api, sync_kind::WELLNESS, None, finished_at)?;
    match &stopped {
        Some(error) => api.set_garmin_last_error(Some(error.clone())),
        None => api.set_garmin_last_error(None),
    }

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
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let runs: Vec<SyncedRun> = runs
        .iter()
        .filter(|run| run.source == source::GARMIN_PERSONAL)
        .map(|run| SyncedRun {
            id: run.id,
            name: run.name.clone(),
            local_date: run.local_date.clone(),
            distance: render::distance(Some(run.distance_m)),
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
        .map_err(|error| api.store_error("the sync state could not be read", &error))?;
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
        .map_err(|error| api.store_error("the sync state could not be read", &error))?;
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
///
/// **The original file is stored with the run**: the FIT file is the source
/// of truth, and every derived series is recomputable from it.
///
/// **A duplicate is refused.** A file that is the same run as one that is
/// already there — within two minutes of the start and within two per cent
/// of the distance — adds no row, and the refusal says which run it is:
/// "already have this run: 12 Oct 2026, 10.2 km".
pub fn auth_fit_import(api: &Api, request: &Request) -> ApiResult<Value> {
    let encoded = request
        .body_str("fit_base64")
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ApiError::bad_request("there is no file in the request"))?;
    let bytes =
        decode_base64(&encoded).ok_or_else(|| ApiError::bad_request("the file is not base64"))?;
    if bytes.len() > api.config().max_fit_import_bytes {
        return Err(ApiError::bad_request("the file is larger than the limit"));
    }
    let decoded = stride_ingest::decode_fit(&bytes, "import.fit")
        .map_err(|_| ApiError::bad_request("the file is not a FIT file"))?;
    let default_date = today(api)?.to_iso();
    let local_date = decoded.local_date.clone().unwrap_or(default_date);

    // The run that is already there, if this file is one of them.
    if let Some(duplicate) = duplicate_of(api, &decoded)? {
        let distance = render::distance(Some(duplicate.distance_m))
            .map(|value| value.text)
            .unwrap_or_default();
        let date = match core_date(&duplicate.local_date) {
            Ok(date) => day_label(date),
            Err(_) => duplicate.local_date.clone(),
        };
        return Err(ApiError::bad_request(format!(
            "already have this run: {date}, {distance}"
        )));
    }

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
        .map_err(|error| api.store_error("the run could not be written", &error))?;
    let response = FitImport {
        imported: true,
        activity_id,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// A run that is already in the store.
struct Duplicate {
    /// The run's day.
    local_date: String,
    /// The run's distance, in metres.
    distance_m: f64,
}

/// The run this file is a copy of, if it is one.
///
/// **The rule is the schema's own**: a FIT import dedupes by start ±2
/// minutes and distance ±2 %. A file that is the same run as one that is
/// already there is refused rather than added twice.
fn duplicate_of(api: &Api, decoded: &stride_ingest::FitActivity) -> ApiResult<Option<Duplicate>> {
    let start = match decoded.start_time {
        Some(start) => start,
        None => return Ok(None),
    };
    let date = decoded
        .local_date
        .clone()
        .unwrap_or_else(|| start.format("%Y-%m-%d").to_string());
    // A run that starts at either side of midnight is on one of two days.
    let previous = match core_date(&date) {
        Ok(day) => day.add_days(-1).to_iso(),
        Err(_) => date.clone(),
    };
    let runs = api
        .store()
        .activities_between(&previous, &date)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    for run in runs.iter() {
        let minutes = (run.start_time - start).num_seconds().abs();
        if minutes > DUPLICATE_START_SECONDS {
            continue;
        }
        if run.distance_m <= 0.0 {
            continue;
        }
        let gap = (run.distance_m - decoded.distance_m).abs() / run.distance_m;
        if gap > DUPLICATE_DISTANCE {
            continue;
        }
        return Ok(Some(Duplicate {
            local_date: run.local_date.clone(),
            distance_m: run.distance_m,
        }));
    }
    Ok(None)
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
            .map_err(|error| api.store_error("the runs could not be read", &error))?;
        let summaries = super::summaries(api, &runs)?;
        days.push(CalendarDay {
            date,
            day,
            distance: render::distance(Some(runs.iter().map(|run| run.distance_m).sum())),
            duration: render::duration(Some(runs.iter().map(|run| run.moving_time_s).sum())),
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
///
/// `Err` means the pull must stop: Garmin is rate limiting, or the
/// connection is gone, and asking again would spend the limit on the same
/// answer. A day with no sleep is not an error — it is a day with no
/// sleep.
fn wellness_of(
    client: &stride_ingest::GarminClient,
    date: &str,
) -> Result<Option<stride_store::WellnessDaily>, stride_ingest::GarminError> {
    let sleep = match client.sleep_daily(date) {
        Ok(value) => value,
        Err(error) if stops_the_pull(&error) => return Err(error),
        Err(_) => Value::Null,
    };
    let hrv = client.hrv_daily(date).unwrap_or(Value::Null);
    let summary = client.daily_summary(date).unwrap_or(Value::Null);
    let sleep_s = field(&sleep, "dailySleep", "sleepTimeSeconds").and_then(|v| v.as_f64());
    let deep_sleep_s = field(&sleep, "dailySleep", "deepSleepSeconds").and_then(|v| v.as_f64());
    let rem_s = field(&sleep, "dailySleep", "remSleepSeconds").and_then(|v| v.as_f64());
    let light_sleep_s = field(&sleep, "dailySleep", "lightSleepSeconds").and_then(|v| v.as_f64());
    let awake_s = field(&sleep, "dailySleep", "awakeSleepSeconds").and_then(|v| v.as_f64());
    let hrv_ms = field(&hrv, "hrvSummary", "lastNightAvg").and_then(|v| v.as_f64());
    let resting_hr = field(&summary, "summaryDTO", "restingHeartRate").and_then(|v| v.as_f64());
    let stress_avg = field(&summary, "summaryDTO", "averageStressLevel").and_then(|v| v.as_f64());
    if sleep_s.is_none() && hrv_ms.is_none() && resting_hr.is_none() {
        return Ok(None);
    }
    Ok(Some(stride_store::WellnessDaily {
        date: date.to_string(),
        sleep_s,
        deep_sleep_s,
        rem_s,
        light_sleep_s,
        awake_s,
        hrv_ms,
        resting_hr,
        stress_avg,
        source: source::GARMIN_PERSONAL.to_string(),
        updated_at: chrono::Utc::now(),
    }))
}

/// A field of a payload, whether Garmin wrapped it or not.
///
/// Garmin's wellness routes answer with the payload itself on some routes
/// and with a wrapper on others. The reading here takes both, so a day of
/// sleep is a day of sleep whichever shape Garmin sends — and a change in
/// Garmin's shape does not silently empty the vitals.
fn field<'a>(value: &'a Value, wrapper: &str, name: &str) -> Option<&'a Value> {
    if let Some(inner) = value.get(wrapper) {
        if let Some(found) = inner.get(name) {
            return Some(found);
        }
    }
    value.get(name)
}

/// Whether a Garmin failure ends the pull.
///
/// A rate limit and a lost connection do: the answer will not change by
/// asking again, and the rate limit is a budget. A refused key and a
/// missing file do not: the pull carries on with the next run.
fn stops_the_pull(error: &stride_ingest::GarminError) -> bool {
    use stride_ingest::GarminError;
    matches!(
        error,
        GarminError::RateLimited { .. } | GarminError::Transport(_) | GarminError::NeedsReconnect
    )
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
        .map_err(|error| api.store_error("the sync state could not be read", &error))?;
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
        .map_err(|error| api.store_error("the sync state could not be written", &error))
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
        .map_err(|error| api.store_error("the notes could not be read", &error))
}

/// The time of the day, as the calendar counts it.
pub fn midnight(date: &str) -> Option<NaiveDateTime> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}
