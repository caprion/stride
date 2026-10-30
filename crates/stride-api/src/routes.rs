//! The routes.
//!
//! One function per endpoint of the Python app. Every response shape, every
//! rounding and every error body is a port of the reference implementation;
//! the references are named in the doc comment of each function.
//!
//! The router does not decide *what the data is*. It decides how the data is
//! presented, and that presentation is shared by the desktop and the phone,
//! because both of them call these functions.

use serde_json::{json, Map, Value};

use crate::analytics;
use crate::backend::{
    ActivityFilter, ActivityPatch, ActivityRow, Backend, ImportRequest, PlanBlockFields,
    PlanPhaseFields, PlanSessionFields, PlanWeekFields, SettingsPatch,
};
use crate::error::{validation_error, ApiError, BackendError};
use crate::num::{mean, pace_s_per_km, round_to};
use crate::query::{body_object, parse_body, Query};
use crate::shapes;

/// The context every handler runs in.
pub struct Ctx<'a> {
    pub backend: &'a dyn Backend,
    /// The day the date-dependent routes count from.
    pub today: chrono::NaiveDate,
}

/// The metrics the fitness trend can chart.
///
/// The reference: `services.fingerprint.METRICS`.
pub const METRICS: [(&str, &str, &str); 11] = [
    ("cadence_avg", "Average cadence", "spm"),
    ("cadence_cv", "Cadence variability", "%"),
    ("cadence_drift_pct", "Cadence drift", "%"),
    ("hr_drift_pct", "Heart rate drift", "%"),
    ("hr_drift_bpm", "Heart rate drift", "bpm"),
    ("pace_drift_pct", "Pace drift", "%"),
    ("decoupling_pct", "Aerobic decoupling", "%"),
    (
        "efficiency_first",
        "Efficiency factor",
        "m/s per HR fraction",
    ),
    ("pace_cv", "Pace variability", "%"),
    ("stride_length_cm", "Stride length", "cm"),
    ("form_score", "Form score", "points"),
];

/// The comparison matrix.
///
/// The reference: `services.comparison.PARAMETERS`. The `better` says which
/// direction is good, so the UI can highlight a winner without hard-coding
/// the rule per row.
pub const PARAMETERS: [(&str, &str, &str, &str, &str); 15] = [
    ("distance_m", "Distance", "km", "neutral", "distance"),
    ("moving_time_s", "Moving time", "", "neutral", "duration"),
    ("pace_s_per_km", "Average pace", "/km", "lower", "pace"),
    (
        "elevation_gain_m",
        "Elevation gain",
        "m",
        "neutral",
        "elevation",
    ),
    ("avg_hr", "Average heart rate", "bpm", "neutral", "hr"),
    ("hr_drift_bpm", "HR drift", "bpm", "lower", "number"),
    ("pace_drift_pct", "Pace drift", "%", "lower", "number"),
    (
        "decoupling_pct",
        "Aerobic decoupling",
        "%",
        "lower",
        "number",
    ),
    ("cadence_avg", "Average cadence", "spm", "neutral", "number"),
    ("cadence_cv", "Cadence stability", "%", "lower", "number"),
    (
        "efficiency_first",
        "Efficiency factor",
        "",
        "higher",
        "number",
    ),
    ("form_score", "Form score", "", "higher", "number"),
    ("running_time_s", "Running time", "", "neutral", "duration"),
    (
        "walking_time_s",
        "Walking excluded",
        "",
        "lower",
        "duration",
    ),
    (
        "stopped_time_s",
        "Standing excluded",
        "",
        "lower",
        "duration",
    ),
];

/// The verdicts the plan engine can return.
///
/// The reference: `api/plan.ALL_VERDICTS`.
pub const ALL_VERDICTS: [&str; 8] = [
    "hit",
    "missed_pace",
    "short",
    "missed",
    "substituted",
    "long",
    "bonus",
    "not_yet",
];

/// The families of measurement the physiology sources endpoint reports.
const SOURCE_FAMILIES: [(&str, &[&str]); 6] = [
    (
        "sleep",
        &[
            "sleep_s",
            "sleep_score",
            "deep_sleep_s",
            "rem_s",
            "light_sleep_s",
            "awake_s",
        ],
    ),
    ("hrv", &["hrv_ms", "hrv_status"]),
    ("resting_hr", &["resting_hr"]),
    ("stress", &["stress_avg", "stress_max"]),
    (
        "body_battery",
        &["body_battery_min", "body_battery_avg", "body_battery_max"],
    ),
    (
        "training_status",
        &[
            "training_readiness",
            "acute_load",
            "chronic_load",
            "vo2max",
            "endurance_score",
            "running_tolerance",
        ],
    ),
];

/// The measurement fields of the physiology table — everything but the date,
/// the source and the timestamp.
const PHYSIOLOGY_FIELDS: [&str; 22] = [
    "sleep_s",
    "sleep_score",
    "deep_sleep_s",
    "rem_s",
    "light_sleep_s",
    "awake_s",
    "hrv_ms",
    "hrv_status",
    "resting_hr",
    "max_hr",
    "stress_avg",
    "stress_max",
    "body_battery_min",
    "body_battery_avg",
    "body_battery_max",
    "steps",
    "training_readiness",
    "acute_load",
    "chronic_load",
    "vo2max",
    "endurance_score",
    "running_tolerance",
];

/// The physiology row as a map, so a field can be read by name.
fn physiology_map(row: &crate::backend::PhysiologyRow) -> Map<String, Value> {
    shapes::physiology_payload(row)
        .as_object()
        .cloned()
        .unwrap_or_default()
}

// ------------------------------------------------------------------ health

/// `GET /api/v1/health` — is the instance up, and is it useful?
///
/// The reference: `main.health`.
pub fn health(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let counts = ctx.backend.counts()?;
    let status = ctx.backend.public_status()?;
    Ok(json!({
        "ok": true,
        "service": "stride",
        "garmin": {
            "connected": status.garmin_connected,
            "method": status.garmin_method,
            "key_source": if status.has_personal_api_key { "key_vault" } else { "none" },
            "last_sync_at": status.last_sync_at,
        },
        "data": {
            "activities": counts.activities,
            "from_garmin": counts.from_garmin,
            "physiology_days": counts.physiology_days,
            "plan_sessions": counts.plan_sessions,
        },
    }))
}

/// `GET /api/v1/health/daily` — the daily metric table.
///
/// The reference: `api/misc.daily`.
pub fn health_daily(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let from = query.optional("from");
    let to = query.optional("to");
    let rows = ctx.backend.daily_metrics(from, to)?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "date": row.date,
                "resting_hr": row.resting_hr,
                "max_hr": row.max_hr,
                "sleep_s": row.sleep_s,
                "deep_sleep_s": row.deep_sleep_s,
                "hrv_ms": row.hrv_ms,
                "stress": row.stress,
                "steps": row.steps,
                "body_battery": row.body_battery,
                "readiness": row.readiness,
                "distance_m": row.distance_m,
            })
        })
        .collect();
    Ok(json!({ "items": items }))
}

// -------------------------------------------------------------------- auth

/// `GET /api/v1/auth/status` — the state, without the secrets.
///
/// The reference: `api/auth.auth_status`.
pub fn auth_status(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let status = ctx.backend.public_status()?;
    let network = ctx.backend.network_status()?;
    let mut payload = serde_json::to_value(&status)
        .map_err(|err| ApiError::internal(format!("could not render the status: {err}")))?;
    if let Some(object) = payload.as_object_mut() {
        object.remove("demo_visible");
        object.insert("network".into(), network);
    }
    Ok(payload)
}

/// Drop the demo flag from a status.
///
/// The sample dataset is gone, and the flag that announced it goes with it:
/// the reference no longer answers it, and a phone that still answered it
/// would be telling the athlete it is showing them runs that are not theirs.
fn without_demo_flag(payload: &mut Value) {
    if let Some(object) = payload.as_object_mut() {
        object.remove("demo_visible");
    }
}

/// `GET /api/v1/auth/pair-code` — mint a pairing code.
///
/// The reference: `api/auth.pair_code`.
pub fn pair_code(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    Ok(ctx.backend.pair_code()?)
}

/// `POST /api/v1/auth/pair` — exchange a pairing code for a credential.
///
/// The reference: `api/auth.pair`. The Python route answers 204 with no body
/// and puts the credential in a cookie; the phone has no cookie jar, so the
/// backend answers with the credential in the body instead.
pub fn pair(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let code = parsed
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| validation_error(vec!["body", "code"], "Field required", "missing"))?;
    if code.len() < 4 || code.len() > 32 {
        return Err(validation_error(
            vec!["body", "code"],
            "String should have between 4 and 32 characters",
            "string_length",
        ));
    }
    match ctx.backend.pair(code)? {
        Some(value) => Ok(value),
        None => Ok(Value::Null),
    }
}

/// `GET /api/v1/auth/devices` — the paired devices.
///
/// The reference: `api/auth.list_devices`.
pub fn devices(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    Ok(json!({ "devices": ctx.backend.devices()? }))
}

/// `DELETE /api/v1/auth/devices/{device_id}` — revoke a device.
///
/// The reference: `api/auth.revoke_device`.
pub fn revoke_device(ctx: &Ctx<'_>, device_id: &str) -> Result<Value, ApiError> {
    Ok(ctx.backend.revoke_device(device_id)?)
}

/// `POST /api/v1/auth/garmin/personal-key` — connect with a Personal API key.
///
/// The reference: `api/auth.connect_personal_key`.
pub fn connect_personal_key(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let api_key = personal_key_from(body)?;
    Ok(ctx.backend.connect_personal_key(&api_key)?)
}

/// `POST /api/v1/auth/garmin/personal-key/test` — validate a key, store nothing.
///
/// The reference: `api/auth.test_personal_key`.
pub fn test_personal_key(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let api_key = personal_key_from(body)?;
    Ok(ctx.backend.test_personal_key(&api_key)?)
}

/// The Personal API key of the body, with FastAPI's length check.
fn personal_key_from(body: Option<&str>) -> Result<String, ApiError> {
    let parsed = body_object(body)?;
    let key = parsed
        .get("api_key")
        .and_then(Value::as_str)
        .ok_or_else(|| validation_error(vec!["body", "api_key"], "Field required", "missing"))?;
    if key.len() < 20 {
        return Err(validation_error(
            vec!["body", "api_key"],
            "String should have at least 20 characters",
            "string_too_short",
        ));
    }
    Ok(key.trim().to_string())
}

/// `POST /api/v1/auth/garmin/developer-key` — start the OAuth handshake.
///
/// The reference: `api/auth.start_developer_key`. The phone build has no
/// OAuth client, so the backend answers 501 there.
pub fn garmin_developer_key(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    Ok(ctx.backend.garmin_oauth("developer_key", &parsed)?)
}

/// `POST /api/v1/auth/garmin/exchange` — finish the OAuth handshake.
///
/// The reference: `api/auth.exchange`.
pub fn garmin_exchange(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    Ok(ctx.backend.garmin_oauth("exchange", &parsed)?)
}

/// `POST /api/v1/auth/garmin/login` — log in with an email and a password.
///
/// The reference: `api/auth.login`.
pub fn garmin_login(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    Ok(ctx.backend.garmin_oauth("login", &parsed)?)
}

/// `POST /api/v1/auth/garmin/mfa` — answer the MFA challenge.
///
/// The reference: `api/auth.submit_mfa`.
pub fn garmin_mfa(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    Ok(ctx.backend.garmin_oauth("mfa", &parsed)?)
}

/// `POST /api/v1/auth/garmin/logout` — drop the Garmin credentials.
///
/// The reference: `api/auth.logout`.
pub fn garmin_logout(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    Ok(ctx.backend.garmin_oauth("logout", &json!({}))?)
}

/// `POST /api/v1/auth/garmin/fit-import` — import one or more FIT files.
///
/// The reference: `api/auth.fit_import`.
pub fn fit_import(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let files = parse_uploads(body)?;
    let mut imported = Vec::new();
    let mut errors = Vec::new();
    for (filename, bytes) in files {
        match ctx.backend.import_fit(&filename, &bytes) {
            Ok(value) => imported.push(value),
            Err(err) => errors.push(json!({
                "filename": filename,
                "error": err.to_string().chars().take(300).collect::<String>(),
            })),
        }
    }
    Ok(json!({
        "imported": imported.len(),
        "activities": imported,
        "errors": errors,
    }))
}

/// The uploaded files, from a multipart body or from the JSON the FFI sends.
fn parse_uploads(body: Option<&str>) -> Result<Vec<(String, Vec<u8>)>, ApiError> {
    let Some(raw) = body else {
        return Ok(Vec::new());
    };
    let trimmed = raw.trim_start();
    if trimmed.starts_with('{') {
        let parsed = parse_body(Some(raw))?;
        let files = parsed
            .get("files")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut out = Vec::new();
        for file in files {
            let filename = file
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or("activity.fit")
                .to_string();
            let content = file
                .get("content_base64")
                .and_then(Value::as_str)
                .unwrap_or("");
            let bytes = crate::util::base64_decode(content).ok_or_else(|| {
                validation_error(
                    vec!["body", "files"],
                    "File content is not base64",
                    "value_error",
                )
            })?;
            out.push((filename, bytes));
        }
        return Ok(out);
    }
    Ok(parse_multipart(raw))
}

/// A minimal multipart parser: enough for the FIT upload form.
fn parse_multipart(raw: &str) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let Some(boundary) = raw
        .split("--")
        .filter_map(|chunk| chunk.trim_start().split('\n').next())
        .find(|line| line.starts_with("----") || boundary_guess(line))
    else {
        return out;
    };
    let boundary = boundary.trim().to_string();
    for part in raw.split(&boundary) {
        let part = part.trim_start_matches(['\r', '\n']);
        if part.is_empty() || part.starts_with("--") {
            continue;
        }
        let Some(split) = part.find("\r\n\r\n") else {
            continue;
        };
        let (headers, content) = part.split_at(split);
        let content = &content[4..];
        let content = content.trim_end_matches(['\r', '\n']);
        let filename = headers
            .lines()
            .find_map(|line| {
                let lower = line.to_ascii_lowercase();
                if lower.starts_with("content-disposition") {
                    line.split("filename=")
                        .nth(1)
                        .map(|name| name.trim_matches('"').to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "activity.fit".to_string());
        out.push((filename, content.as_bytes().to_vec()));
    }
    out
}

/// Whether a line looks like a multipart boundary.
fn boundary_guess(line: &str) -> bool {
    line.starts_with("--") && line.len() > 8 && !line.contains(' ')
}

// -------------------------------------------------------------- activities

/// `GET /api/v1/activities` — the activity feed.
///
/// The reference: `api/activities.list_activities`.
pub fn list_activities(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let page = query.int("page", 1, Some(1), None)?;
    let per_page = query.int("per_page", 20, Some(1), Some(200))?;
    let sport = query.optional("sport").map(str::to_string);
    let date_from = query.optional("from").map(str::to_string);
    let date_to = query.optional("to").map(str::to_string);
    let search = query.optional("q").map(str::to_string);
    let sort = query.optional("sort").unwrap_or("date").to_string();
    let demo_visible = ctx.backend.public_status()?.demo_visible;

    let filter = ActivityFilter {
        sport: sport.clone(),
        date_from,
        date_to,
        query: search,
        sort: sort.clone(),
        page,
        per_page,
        demo_visible,
        all_sports: false,
    };
    let rows = ctx.backend.activities(&filter)?;
    let total = ctx.backend.activity_count(&filter)?;
    // `?fields=` trims what nobody draws — the route preview of two hundred
    // runs is 450 KB of points the feed never shows. It is opt-in: a client
    // that asks for nothing gets the whole summary.
    let wanted = query.field_list("fields");
    let items: Vec<Value> = rows
        .iter()
        .map(shapes::activity_summary)
        .map(|item| shapes::pick(&item, wanted.as_ref(), None))
        .collect();
    Ok(json!({
        "items": items,
        "total": total,
        "page": page,
        "per_page": per_page,
        "pages": std::cmp::max(1, (total + per_page - 1) / per_page),
    }))
}

/// `GET /api/v1/activities/{activity_id}` — one run, in full.
///
/// The reference: `api/activities.get_activity`.
pub fn get_activity(ctx: &Ctx<'_>, activity_id: i64, query: &Query) -> Result<Value, ApiError> {
    let row = ctx
        .backend
        .activity(activity_id)?
        .ok_or_else(|| ApiError::not_found("Activity not found."))?;
    let mut payload = shapes::activity_summary(&row);
    // `?fields=` keeps only the named keys of the answer.
    let wanted = query.field_list("fields");
    let laps = ctx.backend.activity_laps(activity_id)?;
    let streams = ctx.backend.activity_streams(activity_id)?;
    let zones = ctx.backend.activity_zones(activity_id)?;
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "laps".into(),
            Value::Array(laps.iter().map(shapes::lap_payload).collect()),
        );
        object.insert(
            "streams".into(),
            Value::Object(
                streams
                    .iter()
                    .map(|stream| {
                        (
                            stream.kind.clone(),
                            json!({
                                "seconds": stream.seconds,
                                "values": stream.values,
                                "unit": stream.unit,
                            }),
                        )
                    })
                    .collect(),
            ),
        );
        object.insert(
            "hr_zones".into(),
            Value::Array(zones.iter().map(shapes::zone_payload).collect()),
        );
        object.insert(
            "geo_points".into(),
            Value::Array(
                ctx.backend
                    .activity_geo(activity_id)?
                    .unwrap_or_default()
                    .iter()
                    .map(|point| json!([point[0], point[1]]))
                    .collect(),
            ),
        );
    }
    Ok(shapes::pick(&payload, wanted.as_ref(), None))
}

/// `PATCH /api/v1/activities/{activity_id}` — change the editable fields.
///
/// The reference: `api/activities.patch_activity`. A field that is absent or
/// null is left alone, exactly as the Python's `exclude_unset` does.
pub fn patch_activity(
    ctx: &Ctx<'_>,
    activity_id: i64,
    body: Option<&str>,
) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let mut patch = ActivityPatch::default();
    if let Some(value) = parsed.get("name") {
        if !value.is_null() {
            patch.name = value.as_str().map(str::to_string);
        }
    }
    if let Some(value) = parsed.get("description") {
        if !value.is_null() {
            patch.description = value.as_str().map(str::to_string);
        }
    }
    if let Some(value) = parsed.get("perceived_exertion") {
        if !value.is_null() {
            patch.perceived_exertion = value.as_i64();
        }
    }
    if let Some(value) = parsed.get("is_trail") {
        if !value.is_null() {
            patch.is_trail = value.as_bool();
        }
    }
    if let Some(value) = parsed.get("is_commute") {
        if !value.is_null() {
            patch.is_commute = value.as_bool();
        }
    }
    let row = ctx
        .backend
        .patch_activity(activity_id, &patch)?
        .ok_or_else(|| ApiError::not_found("Activity not found."))?;
    Ok(shapes::activity_summary(&row))
}

/// `GET /api/v1/auth/garmin/callback` — the OAuth redirect target.
///
/// The reference: `api/auth.garmin_callback`. The Python route renders an
/// HTML page; the router answers with the title and the message of that page,
/// and the HTTP layer renders it.
pub fn garmin_callback(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let verifier = query.optional("oauth_verifier").unwrap_or_default();
    if verifier.is_empty() {
        return Ok(json!({
            "title": "Almost there",
            "message": "Garmin did not include an authorisation code. Start the connection again and copy the code Garmin shows you.",
            "success": false,
        }));
    }
    let body = json!({
        "oauth_token": query.optional("oauth_token").unwrap_or_default(),
        "oauth_verifier": verifier,
    });
    match ctx.backend.garmin_oauth("exchange", &body) {
        Ok(_) => Ok(json!({
            "title": "Garmin connected",
            "message": "Your watch data will sync automatically. You can close this tab.",
            "success": true,
        })),
        Err(err) => Ok(json!({
            "title": "Connection failed",
            "message": err.to_string(),
            "success": false,
        })),
    }
}

/// `GET /api/v1/activities/{activity_id}/geo` — the route of one run.
///
/// The reference: `api/activities.activity_geo`.
pub fn activity_geo(ctx: &Ctx<'_>, activity_id: i64) -> Result<Value, ApiError> {
    if ctx.backend.activity(activity_id)?.is_none() {
        return Err(ApiError::not_found("Activity not found."));
    }
    let points = ctx.backend.activity_geo(activity_id)?.unwrap_or_default();
    let bbox = if points.is_empty() {
        Value::Null
    } else {
        let lats: Vec<f64> = points.iter().map(|point| point[0]).collect();
        let lons: Vec<f64> = points.iter().map(|point| point[1]).collect();
        json!([
            lons.iter().cloned().fold(f64::INFINITY, f64::min),
            lats.iter().cloned().fold(f64::INFINITY, f64::min),
            lons.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            lats.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ])
    };
    Ok(json!({
        "points": points.iter().map(|point| json!([point[0], point[1]])).collect::<Vec<Value>>(),
        "bbox": bbox,
        "has_geo": !points.is_empty(),
    }))
}

/// `GET /api/v1/activities/{activity_id}/streams/{kind}` — one series.
///
/// The reference: `api/activities.activity_stream`.
pub fn activity_stream(ctx: &Ctx<'_>, activity_id: i64, kind: &str) -> Result<Value, ApiError> {
    let streams = ctx.backend.activity_streams(activity_id)?;
    let stream = streams
        .iter()
        .find(|stream| stream.kind == kind)
        .ok_or_else(|| ApiError::not_found("Stream not found."))?;
    Ok(shapes::stream_payload(stream))
}

// ------------------------------------------------------------------- stats

/// `GET /api/v1/stats/summary` — totals, gains and the recent runs.
///
/// The reference: `api/stats.summary`.
pub fn stats_summary(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let mut payload = analytics::summary(&rows, ctx.today, demo_visible);
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "recent".into(),
            analytics::recent_activities(&rows, demo_visible, 8),
        );
    }
    Ok(payload)
}

/// `GET /api/v1/stats/weekly` — the volume of each week.
///
/// The reference: `api/stats.weekly`.
pub fn stats_weekly(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let weeks = query.int("weeks", 26, Some(1), Some(200))?;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    // `?fields=` keeps the named fields of each bucket.
    let wanted = query.field_list("fields");
    Ok(json!({
        "items": trim_items(
            analytics::weekly_volume(&rows, demo_visible, weeks as usize),
            wanted.as_ref(),
        ),
    }))
}

/// `GET /api/v1/stats/monthly` — the volume of each month.
///
/// The reference: `api/stats.monthly`.
pub fn stats_monthly(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let months = query.int("months", 12, Some(1), Some(60))?;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let wanted = query.field_list("fields");
    Ok(json!({
        "items": trim_items(
            analytics::monthly_volume(&rows, demo_visible, months as usize),
            wanted.as_ref(),
        ),
    }))
}

/// `GET /api/v1/stats/pace-trends` — the weekly pace statistics.
///
/// The reference: `api/stats.pace_trends`.
pub fn stats_pace_trends(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let weeks = query.int("weeks", 26, Some(1), Some(200))?;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let wanted = query.field_list("fields");
    Ok(json!({
        "items": trim_items(
            analytics::pace_trends(&rows, demo_visible, weeks as usize),
            wanted.as_ref(),
        ),
    }))
}

/// `GET /api/v1/stats/hr-zones` — the heart-rate zone distribution.
///
/// The reference: `api/stats.hr_zones`.
pub fn stats_hr_zones(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let weeks = query.int("weeks", 12, Some(1), Some(104))?;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let mut laps = Vec::new();
    for row in &rows {
        laps.extend(ctx.backend.activity_laps(row.id)?);
    }
    Ok(analytics::hr_zones(
        &rows,
        &laps,
        demo_visible,
        weeks as usize,
        ctx.today,
    ))
}

/// `GET /api/v1/stats/personal-records` — the best performances.
///
/// The reference: `api/stats.personal_records`.
pub fn stats_personal_records(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let activities = analytics::runs(&rows, demo_visible);
    let wanted = query.field_list("fields");
    Ok(json!({
        "items": trim_items(analytics::personal_records(&activities), wanted.as_ref()),
    }))
}

/// `GET /api/v1/stats/heatmap` — the daily distance of a year.
///
/// The reference: `api/stats.heatmap`.
pub fn stats_heatmap(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let year = match query.optional("year") {
        Some(raw) => raw.parse::<i32>().map_err(|_| {
            validation_error(
                vec!["query", "year"],
                "Input should be a valid integer, unable to parse string as an integer",
                "int_parsing",
            )
        })?,
        None => chrono::Datelike::year(&ctx.today),
    };
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    Ok(analytics::heatmap(&rows, demo_visible, year))
}

/// `GET /api/v1/stats/elevation` — the climb of each week.
///
/// The reference: `api/stats.elevation`.
pub fn stats_elevation(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let weeks = query.int("weeks", 26, Some(1), Some(200))?;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let volume = analytics::weekly_volume(&rows, demo_visible, weeks as usize);
    let items: Vec<Value> = volume
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    json!({
                        "week_key": item.get("week_key").cloned().unwrap_or(Value::Null),
                        "elevation_gain_m": item.get("elevation_gain_m").cloned().unwrap_or(Value::Null),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(json!({ "items": items }))
}

/// `GET /api/v1/calendar` — the training calendar of one month.
///
/// The reference: `api/misc.calendar`.
pub fn calendar(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let year = query.int(
        "year",
        chrono::Datelike::year(&ctx.today) as i64,
        None,
        None,
    )? as i32;
    let month = query.int(
        "month",
        chrono::Datelike::month(&ctx.today) as i64,
        Some(1),
        Some(12),
    )? as u32;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    // `?fields=` keeps the named fields of each run card.
    let wanted = query.field_list("fields");
    let mut calendar = analytics::calendar(&rows, demo_visible, year, month);
    if let Some(days) = calendar.get_mut("days").and_then(Value::as_array_mut) {
        for day in days {
            let Some(object) = day.as_object_mut() else {
                continue;
            };
            let cards = object
                .get("activities")
                .cloned()
                .unwrap_or(Value::Array(Vec::new()));
            object.insert("activities".into(), trim_items(cards, wanted.as_ref()));
        }
    }
    Ok(calendar)
}

// ------------------------------------------------------------------- sync

/// `POST /api/v1/sync/run` — start a sync.
///
/// The reference: `api/misc.run_sync`.
pub fn sync_run(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let days = parsed.get("days").and_then(Value::as_i64);
    let background = parsed
        .get("background")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    Ok(ctx.backend.run_sync(days, background)?)
}

/// `GET /api/v1/sync/status` — the state of the last sync.
///
/// The reference: `api/misc.sync_status`.
pub fn sync_status(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let status = ctx.backend.sync_status()?;
    let state = ctx.backend.public_status()?;
    let mut payload = match status {
        Value::Object(object) => object,
        other => {
            let mut object = Map::new();
            object.insert("status".into(), other);
            object
        }
    };
    payload.insert("last_sync_at".into(), json!(state.last_sync_at));
    payload.insert("connected".into(), json!(state.garmin_connected));
    payload.insert("method".into(), json!(state.garmin_method));
    Ok(Value::Object(payload))
}

/// `GET /api/v1/sync/runs` — the sync log.
///
/// The reference: `api/misc.sync_runs`.
pub fn sync_runs(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let limit = query.int("limit", 20, Some(1), Some(100))?;
    Ok(json!({ "items": ctx.backend.sync_runs(limit as usize)? }))
}

/// `GET /api/v1/sync/history` — the history windows that are configured.
///
/// The reference: `api/misc.history_settings`.
pub fn history_settings(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let status = ctx.backend.public_status()?;
    Ok(json!({
        "history_months": status.history_months,
        "health_history_days": status.health_history_days,
        "sync_days": status.sync_days,
        "limits": {
            "activities": "Everything the Garmin account holds — the Activity API has no short retention window.",
            "health": "Garmin keeps roughly a year of sleep, heart rate, stress and body battery.",
        },
    }))
}

/// `POST /api/v1/sync/history` — load a chosen span of history.
///
/// The reference: `api/misc.load_history`.
pub fn load_history(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let activities = parsed.get("activities").and_then(Value::as_i64);
    let health_days = parsed.get("health_days").and_then(Value::as_i64);
    let background = parsed
        .get("background")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let mut result = ctx
        .backend
        .load_history(activities, health_days, background)?;
    if let Some(object) = result.as_object_mut() {
        object.insert("activities_months".into(), json!(activities));
        object.insert("health_days".into(), json!(health_days));
    }
    Ok(result)
}

/// `PATCH /api/v1/sync/settings` — change the sync settings.
///
/// The reference: `api/misc.patch_settings`.
pub fn patch_settings(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let mut patch = SettingsPatch::default();
    if let Some(days) = parsed.get("sync_days").and_then(Value::as_i64) {
        patch.sync_days = Some(days.clamp(1, 3650));
    }
    if let Some(auto) = parsed.get("auto_sync").and_then(Value::as_bool) {
        patch.auto_sync = Some(auto);
    }
    if let Some(minutes) = parsed.get("sync_interval_minutes").and_then(Value::as_i64) {
        patch.sync_interval_minutes = Some(minutes.clamp(15, 1440));
    }
    if let Some(route) = parsed.get("network_route").and_then(Value::as_str) {
        if !matches!(route, "auto" | "direct" | "exit_node") {
            return Err(ApiError::bad_request("Unknown network route."));
        }
        patch.network_route = Some(route.to_string());
    }
    if let Some(node) = parsed.get("exit_node").and_then(Value::as_str) {
        patch.exit_node = Some(node.to_string());
    }
    // The demo flag is not a setting: the sample dataset is gone, and a
    // request that names it is ignored, the way the reference ignores it.
    let status = ctx.backend.update_settings(&patch)?;
    let mut payload = serde_json::to_value(&status)
        .map_err(|err| ApiError::internal(format!("could not render the status: {err}")))?;
    without_demo_flag(&mut payload);
    Ok(payload)
}

// ---------------------------------------------------------------- network

/// `GET /api/v1/network/status` — the exit-node configuration.
///
/// The reference: `api/misc.network_status`.
pub fn network_status(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let mut payload = ctx.backend.network_status()?;
    let status = ctx.backend.public_status()?;
    let nodes = ctx.backend.exit_nodes()?;
    let identity = ctx.backend.public_identity()?;
    let resolved = status.exit_node.clone().or_else(|| first_node_name(&nodes));
    if let Some(object) = payload.as_object_mut() {
        let route = object.get("route").cloned().unwrap_or(Value::Null);
        object.insert("public_identity".into(), identity);
        object.insert("effective_route".into(), route);
        object.insert("route".into(), json!(status.network_route));
        object.insert("nodes".into(), Value::Array(nodes));
        object.insert("resolved_exit_node".into(), json!(resolved));
    }
    Ok(payload)
}

/// The address of the exit node that would be used if none is pinned.
///
/// The reference: `network.resolve_exit_node` — the first online node.
fn first_node_name(nodes: &[Value]) -> Option<String> {
    nodes
        .iter()
        .find(|node| node.get("online").and_then(Value::as_bool).unwrap_or(false))
        .or_else(|| nodes.first())
        .and_then(|node| {
            node.get("ip")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| node.get("name").and_then(Value::as_str).map(str::to_string))
                .or_else(|| node.as_str().map(str::to_string))
        })
}

/// `POST /api/v1/network/route` — choose the egress route.
///
/// The reference: `api/misc.set_route`.
pub fn set_route(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let route = parsed
        .get("route")
        .and_then(Value::as_str)
        .ok_or_else(|| validation_error(vec!["body", "route"], "Field required", "missing"))?;
    if !matches!(route, "direct" | "exit_node") {
        return Err(ApiError::bad_request(
            "Route must be 'direct' or 'exit_node'.",
        ));
    }
    Ok(ctx.backend.set_network_route(route)?)
}

// ------------------------------------------------------------------- plan

/// `GET /api/v1/plan/blocks` — every block.
///
/// The reference: `api/plan.list_blocks`.
pub fn plan_blocks(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let blocks = ctx.backend.plan_blocks()?;
    let wanted = query.field_list("fields");
    let mut items = Vec::new();
    for block in &blocks {
        let (phases, weeks, sessions) = block_counts(ctx, block.id)?;
        items.push(shapes::pick(
            &shapes::plan_block_payload(block, phases, weeks, sessions),
            wanted.as_ref(),
            None,
        ));
    }
    Ok(json!({ "items": items }))
}

/// The number of phases, weeks and sessions of a block.
fn block_counts(ctx: &Ctx<'_>, block_id: i64) -> Result<(usize, usize, usize), ApiError> {
    let phases = ctx.backend.plan_phases(block_id)?;
    let mut weeks = 0usize;
    let mut sessions = 0usize;
    for phase in &phases {
        let phase_weeks = ctx.backend.plan_weeks(phase.id)?;
        weeks += phase_weeks.len();
        for week in &phase_weeks {
            sessions += ctx.backend.plan_sessions(week.id)?.len();
        }
    }
    Ok((phases.len(), weeks, sessions))
}

/// `POST /api/v1/plan/blocks` — create a block.
///
/// The reference: `api/plan.create_block`.
pub fn create_plan_block(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let fields = PlanBlockFields {
        name: parsed
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Training block")
            .to_string(),
        start_date: parsed
            .get("start_date")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| ctx.today.format("%Y-%m-%d").to_string()),
        end_date: parsed
            .get("end_date")
            .and_then(Value::as_str)
            .map(str::to_string),
        goal: parsed
            .get("goal")
            .and_then(Value::as_str)
            .map(str::to_string),
        status: parsed
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_string),
        source: parsed
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_string),
        notes: parsed
            .get("notes")
            .and_then(Value::as_str)
            .map(str::to_string),
    };
    let block = ctx.backend.create_plan_block(&fields)?;
    plan_block_detail(ctx, &block)
}

/// `GET /api/v1/plan/blocks/{block_id}` — one block, in full.
///
/// The reference: `api/plan.get_block`.
pub fn get_plan_block(ctx: &Ctx<'_>, block_id: i64) -> Result<Value, ApiError> {
    let block = ctx
        .backend
        .plan_block(block_id)?
        .ok_or_else(|| ApiError::not_found("Plan block not found."))?;
    plan_block_detail(ctx, &block)
}

/// The full shape of a block.
///
/// The reference: `api/plan._block_detail`.
fn plan_block_detail(
    ctx: &Ctx<'_>,
    block: &crate::backend::PlanBlockRow,
) -> Result<Value, ApiError> {
    let phases = ctx.backend.plan_phases(block.id)?;
    let mut phase_items = Vec::new();
    let mut week_count = 0usize;
    let mut session_count = 0usize;
    for phase in &phases {
        let weeks = ctx.backend.plan_weeks(phase.id)?;
        week_count += weeks.len();
        let mut week_items = Vec::new();
        for week in &weeks {
            session_count += ctx.backend.plan_sessions(week.id)?.len();
            week_items.push(week_detail(ctx, week, Some(phase))?);
        }
        let mut payload = shapes::plan_phase_payload(phase);
        if let Some(object) = payload.as_object_mut() {
            object.insert("weeks".into(), Value::Array(week_items));
        }
        phase_items.push(payload);
    }
    Ok(json!({
        "block": shapes::plan_block_payload(block, phases.len(), week_count, session_count),
        "phases": phase_items,
    }))
}

/// The full shape of a week.
///
/// The reference: `api/plan._week_detail`.
fn week_detail(
    ctx: &Ctx<'_>,
    week: &crate::backend::PlanWeekRow,
    phase: Option<&crate::backend::PlanPhaseRow>,
) -> Result<Value, ApiError> {
    let (sessions, planned_km, actual_km, completed) = week_totals(ctx, week)?;
    let planned_count = sessions.len();
    Ok(json!({
        "week": shapes::plan_week_payload(week),
        "phase": phase.map(shapes::plan_phase_payload).unwrap_or(Value::Null),
        "sessions": sessions,
        "planned_km": planned_km,
        "actual_km": actual_km,
        "planned_count": planned_count,
        "completed_count": completed,
        "adherence_pct": if planned_count > 0 {
            json!(round_to(completed as f64 / planned_count as f64 * 100.0, 1))
        } else {
            Value::Null
        },
    }))
}

/// The sessions of a week and the planned-versus-actual totals.
///
/// The reference: `api/plan._week_detail` and `api/plan._sessions_with_results`.
fn week_totals(
    ctx: &Ctx<'_>,
    week: &crate::backend::PlanWeekRow,
) -> Result<(Vec<Value>, f64, f64, usize), ApiError> {
    let (Some(start), Some(end)) = (week.start_date.as_deref(), week.end_date.as_deref()) else {
        return Ok((Vec::new(), 0.0, 0.0, 0));
    };
    let rows = ctx.backend.plan_sessions_with_results(start, end)?;
    let mut sessions = Vec::new();
    let mut planned = 0.0;
    let mut actual = 0.0;
    let mut completed = 0usize;
    for row in &rows {
        let mut payload = shapes::plan_session_payload(&row.session);
        if let Some(object) = payload.as_object_mut() {
            object.insert("result".into(), row.result.clone());
            object.insert(
                "activity".into(),
                row.activity
                    .as_ref()
                    .map(shapes::plan_activity)
                    .unwrap_or(Value::Null),
            );
        }
        planned += row.session.target_distance_m.unwrap_or(0.0);
        if let Some(activity) = &row.activity {
            actual += activity.distance_m;
            completed += 1;
        }
        sessions.push(payload);
    }
    Ok((
        sessions,
        round_to(planned / 1000.0, 2),
        round_to(actual / 1000.0, 2),
        completed,
    ))
}

/// `POST /api/v1/plan/blocks/{block_id}/phases` — add a phase.
///
/// The reference: `api/plan.add_phase`.
pub fn add_plan_phase(ctx: &Ctx<'_>, block_id: i64, body: Option<&str>) -> Result<Value, ApiError> {
    if ctx.backend.plan_block(block_id)?.is_none() {
        return Err(ApiError::not_found("Plan block not found."));
    }
    let parsed = body_object(body)?;
    let fields = PlanPhaseFields {
        name: parsed
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Phase")
            .to_string(),
        start_date: parsed
            .get("start_date")
            .and_then(Value::as_str)
            .map(str::to_string),
        end_date: parsed
            .get("end_date")
            .and_then(Value::as_str)
            .map(str::to_string),
        target_weekly_km_min: parsed.get("target_weekly_km_min").and_then(Value::as_f64),
        target_weekly_km_max: parsed.get("target_weekly_km_max").and_then(Value::as_f64),
        target_sessions: parsed.get("target_sessions").and_then(Value::as_i64),
        pace_easy_low: parsed.get("pace_easy_low").and_then(Value::as_f64),
        pace_easy_high: parsed.get("pace_easy_high").and_then(Value::as_f64),
        pace_tempo_low: parsed.get("pace_tempo_low").and_then(Value::as_f64),
        pace_tempo_high: parsed.get("pace_tempo_high").and_then(Value::as_f64),
        pace_interval_low: parsed.get("pace_interval_low").and_then(Value::as_f64),
        pace_interval_high: parsed.get("pace_interval_high").and_then(Value::as_f64),
        pace_long_low: parsed.get("pace_long_low").and_then(Value::as_f64),
        pace_long_high: parsed.get("pace_long_high").and_then(Value::as_f64),
        hr_zone_target: parsed.get("hr_zone_target").and_then(Value::as_i64),
        notes: parsed
            .get("notes")
            .and_then(Value::as_str)
            .map(str::to_string),
    };
    let phase = ctx.backend.create_plan_phase(block_id, &fields)?;
    Ok(shapes::plan_phase_payload(&phase))
}

/// `POST /api/v1/plan/phases/{phase_id}/weeks` — add a week.
///
/// The reference: `api/plan.add_week`.
pub fn add_plan_week(ctx: &Ctx<'_>, phase_id: i64, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let fields = PlanWeekFields {
        week_number: parsed
            .get("week_number")
            .and_then(Value::as_i64)
            .unwrap_or(1),
        start_date: parsed
            .get("start_date")
            .and_then(Value::as_str)
            .map(str::to_string),
        end_date: parsed
            .get("end_date")
            .and_then(Value::as_str)
            .map(str::to_string),
        target_km: parsed.get("target_km").and_then(Value::as_f64),
        key_workout: parsed
            .get("key_workout")
            .and_then(Value::as_str)
            .map(str::to_string),
        notes: parsed
            .get("notes")
            .and_then(Value::as_str)
            .map(str::to_string),
    };
    let week = ctx.backend.create_plan_week(phase_id, &fields)?;
    Ok(shapes::plan_week_payload(&week))
}

/// `POST /api/v1/plan/weeks/{week_id}/sessions` — add a session.
///
/// The reference: `api/plan.add_session`.
pub fn add_plan_session(
    ctx: &Ctx<'_>,
    week_id: i64,
    body: Option<&str>,
) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let fields = PlanSessionFields {
        date: parsed
            .get("date")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        session_type: parsed
            .get("session_type")
            .and_then(Value::as_str)
            .unwrap_or("easy")
            .to_string(),
        title: parsed
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        details: parsed
            .get("details")
            .and_then(Value::as_str)
            .map(str::to_string),
        target_distance_m: parsed.get("target_distance_m").and_then(Value::as_f64),
        pace_low_s_km: parsed.get("pace_low_s_km").and_then(Value::as_f64),
        pace_high_s_km: parsed.get("pace_high_s_km").and_then(Value::as_f64),
        hr_zone_target: parsed.get("hr_zone_target").and_then(Value::as_i64),
        status: parsed
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_string),
        source: parsed
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_string),
        notes: parsed
            .get("notes")
            .and_then(Value::as_str)
            .map(str::to_string),
    };
    let session = ctx.backend.create_plan_session(week_id, &fields)?;
    Ok(shapes::plan_session_payload(&session))
}

/// `POST /api/v1/plan/import` — import a plan document.
///
/// The reference: `api/plan.import_document`.
pub fn import_plan(ctx: &Ctx<'_>, body: Option<&str>) -> Result<Value, ApiError> {
    let parsed = body_object(body)?;
    let content = parsed
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| validation_error(vec!["body", "content"], "Field required", "missing"))?;
    let request = ImportRequest {
        format: parsed
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("markdown")
            .to_string(),
        content: content.to_string(),
        name: parsed
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string),
        anchor: parsed
            .get("anchor")
            .and_then(Value::as_str)
            .unwrap_or("activities")
            .to_string(),
    };
    Ok(ctx.backend.import_plan(&request)?)
}

/// `GET /api/v1/plan/current` — the active block and the week we are in.
///
/// The reference: `api/plan.current`.
pub fn plan_current(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    // The block carries every phase, every week and every session: 107 KB
    // that no screen draws. `?fields=week` asks for the week alone.
    let wanted = query.field_list("fields");
    let Some(block) = active_block(ctx)? else {
        return Ok(json!({ "block": Value::Null, "week": Value::Null }));
    };
    let detail = plan_block_detail(ctx, &block)?;
    let week = current_week(ctx, &block)?;
    Ok(shapes::pick(
        &json!({ "block": detail, "week": week }),
        wanted.as_ref(),
        None,
    ))
}

/// The block in force: the active one, or the most recent if none is active.
///
/// The reference: `api/plan._active_block`.
fn active_block(ctx: &Ctx<'_>) -> Result<Option<crate::backend::PlanBlockRow>, ApiError> {
    let blocks = ctx.backend.plan_blocks()?;
    let active = blocks
        .iter()
        .filter(|block| block.status == "active")
        .max_by(|a, b| a.start_date.cmp(&b.start_date));
    Ok(active.cloned().or_else(|| blocks.first().cloned()))
}

/// The week we are in, or the nearest one.
///
/// The reference: `api/plan._current_week`.
fn current_week(ctx: &Ctx<'_>, block: &crate::backend::PlanBlockRow) -> Result<Value, ApiError> {
    let mut weeks = Vec::new();
    for phase in ctx.backend.plan_phases(block.id)? {
        weeks.extend(ctx.backend.plan_weeks(phase.id)?);
    }
    if weeks.is_empty() {
        return Ok(Value::Null);
    }
    let today = ctx.today.format("%Y-%m-%d").to_string();
    let dated: Vec<&crate::backend::PlanWeekRow> = weeks
        .iter()
        .filter(|week| week.start_date.is_some() && week.end_date.is_some())
        .collect();
    let chosen = dated
        .iter()
        .find(|week| {
            week.start_date
                .as_deref()
                .is_some_and(|start| start <= today.as_str())
                && week
                    .end_date
                    .as_deref()
                    .is_some_and(|end| end >= today.as_str())
        })
        .copied()
        .or_else(|| {
            dated
                .iter()
                .min_by_key(|week| {
                    let start = week.start_date.as_deref().unwrap_or("");
                    let start_date =
                        chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap_or(ctx.today);
                    (start_date - ctx.today).num_days().abs()
                })
                .copied()
        });
    let Some(week) = chosen else {
        return Ok(Value::Null);
    };
    let phase = ctx
        .backend
        .plan_phases(block.id)?
        .into_iter()
        .find(|phase| phase.id == week.phase_id);
    week_detail(ctx, week, phase.as_ref())
}

/// `GET /api/v1/plan/week` — the week that contains a date.
///
/// The reference: `api/plan.get_week`.
pub fn plan_week(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    // `?fields=` keeps the named fields of each planned session.
    let wanted = query.field_list("fields");
    let day = query.required("date")?;
    let target = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .map_err(|_| ApiError::bad_request("date must be YYYY-MM-DD."))?;
    let start = target
        - chrono::Duration::days(chrono::Datelike::weekday(&target).num_days_from_monday() as i64);
    let end = start + chrono::Duration::days(6);
    let start_text = start.format("%Y-%m-%d").to_string();
    let end_text = end.format("%Y-%m-%d").to_string();

    let mut weeks = Vec::new();
    for block in ctx.backend.plan_blocks()? {
        for phase in ctx.backend.plan_phases(block.id)? {
            for week in ctx.backend.plan_weeks(phase.id)? {
                let overlaps = week
                    .start_date
                    .as_deref()
                    .is_some_and(|value| value <= end_text.as_str())
                    && week
                        .end_date
                        .as_deref()
                        .is_some_and(|value| value >= start_text.as_str());
                if overlaps {
                    weeks.push((week, phase.clone()));
                }
            }
        }
    }
    weeks.sort_by_key(|(week, _)| week.week_number);

    let active_id = active_block(ctx)?.map(|block| block.id);
    let chosen = weeks
        .iter()
        .find(|(_, phase)| active_id.map_or(true, |id| phase.block_id == id));

    let Some((week, phase)) = chosen else {
        let sessions = session_items(ctx, &start_text, &end_text)?;
        return Ok(json!({
            "week": Value::Null,
            "phase": Value::Null,
            "sessions": trim_sessions(Value::Array(sessions), wanted.as_ref()),
            "planned_km": 0.0,
            "actual_km": 0.0,
            "planned_count": 0,
            "completed_count": 0,
            "adherence_pct": Value::Null,
        }));
    };
    let mut detail = week_detail(ctx, week, Some(phase))?;
    if let Some(object) = detail.as_object_mut() {
        if let Some(sessions) = object.get("sessions").cloned() {
            object.insert("sessions".into(), trim_sessions(sessions, wanted.as_ref()));
        }
    }
    Ok(detail)
}

/// The items of a list, with only the named fields.
fn trim_items(items: Value, wanted: Option<&Vec<String>>) -> Value {
    let Some(items) = items.as_array() else {
        return items;
    };
    Value::Array(
        items
            .iter()
            .map(|item| shapes::pick(item, wanted, None))
            .collect(),
    )
}

/// The planned sessions, with only the named fields.
fn trim_sessions(sessions: Value, wanted: Option<&Vec<String>>) -> Value {
    let Some(items) = sessions.as_array() else {
        return sessions;
    };
    Value::Array(
        items
            .iter()
            .map(|session| shapes::pick(session, wanted, None))
            .collect(),
    )
}

/// The sessions of a date range, each with its result and activity.
fn session_items(ctx: &Ctx<'_>, from: &str, to: &str) -> Result<Vec<Value>, ApiError> {
    let rows = ctx.backend.plan_sessions_with_results(from, to)?;
    let mut out = Vec::new();
    for row in &rows {
        let mut payload = shapes::plan_session_payload(&row.session);
        if let Some(object) = payload.as_object_mut() {
            object.insert("result".into(), row.result.clone());
            object.insert(
                "activity".into(),
                row.activity
                    .as_ref()
                    .map(shapes::plan_activity)
                    .unwrap_or(Value::Null),
            );
        }
        out.push(payload);
    }
    Ok(out)
}

/// `GET /api/v1/plan/results` — the verdicts of a date range.
///
/// The reference: `api/plan.results`.
pub fn plan_results(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let (from, to) = plan_date_range(ctx, query)?;
    Ok(json!({ "items": ctx.backend.plan_results(&from, &to)? }))
}

/// The date range of the plan results: the request, or the active plan.
///
/// The reference: `api/plan._plan_date_range`.
fn plan_date_range(ctx: &Ctx<'_>, query: &Query) -> Result<(String, String), ApiError> {
    if let (Some(from), Some(to)) = (query.optional("from"), query.optional("to")) {
        return Ok((from.to_string(), to.to_string()));
    }
    let today = ctx.today.format("%Y-%m-%d").to_string();
    let Some(block) = active_block(ctx)? else {
        return Ok((today.clone(), today));
    };
    let start = block.start_date.clone();
    let end = block.end_date.unwrap_or_else(|| today.clone());
    Ok((start, end))
}

/// `GET /api/v1/plan/sources` — the plans Stride can offer.
///
/// Two kinds of plan, and they are not the same kind of thing: the plan the
/// athlete wrote down (a *prescription*) and the plan they already have (a
/// *baseline*, inferred from their own history). Each is judged by its own
/// ruler, chosen by the source's `prescribed` flag, so a baseline never says
/// "missed": nobody wrote it down.
///
/// The reference: `api/plan.plan_sources_route`.
pub fn plan_sources(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let (from, to) = plan_date_range(ctx, query)?;
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        sport: None,
        date_from: Some(from.clone()),
        date_to: Some(to.clone()),
        query: None,
        sort: "date".to_string(),
        page: 1,
        per_page: 5_000,
        demo_visible,
        all_sports: true,
    })?;
    let activities: Vec<stride_core::Activity> = rows.iter().map(core_fact).collect();

    let inferred = stride_core::plan_sources::infer_plan(&activities, &from, &to);
    let inferred = serde_json::to_value(&inferred)
        .map_err(|err| ApiError::internal(format!("the inferred plan did not render: {err}")))?;
    let garmin = garmin_source(ctx, &from, &to)?;

    let today = ctx.today.format("%Y-%m-%d").to_string();
    Ok(json!({
        "inferred": source_with_verdicts(&inferred, &activities, &today),
        "garmin": source_with_verdicts(&garmin, &activities, &today),
    }))
}

/// A plan a source produced, with the verdict on each of its sessions.
///
/// The verdict function is chosen by the source's own `prescribed` flag: a
/// prescription is judged hit/missed, a baseline is judged less than usual
/// and never says "missed".
fn source_with_verdicts(
    payload: &Value,
    activities: &[stride_core::Activity],
    today: &str,
) -> Value {
    let prescribed = payload
        .get("prescribed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let sessions = payload
        .get("sessions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let judged: Vec<Value> = sessions
        .iter()
        .filter_map(|session| {
            let parsed: stride_core::PlanSession = serde_json::from_value(session.clone()).ok()?;
            let day = parsed.date.clone();
            let activity = activity_on(activities, &day);
            let verdict = stride_core::verdict_for_source(&parsed, activity, today, prescribed);
            let mut value = session.clone();
            if let Some(object) = value.as_object_mut() {
                let rendered = serde_json::to_value(&verdict).unwrap_or(Value::Null);
                if let Some(fields) = rendered.as_object() {
                    for (name, field) in fields {
                        object.insert(name.clone(), field.clone());
                    }
                }
            }
            // The words the screen shows.
            let (label, message) = if prescribed {
                (verdict_label(&verdict.verdict), String::new())
            } else {
                let baseline = stride_core::plan_sources::baseline_verdict(&parsed, activity);
                (baseline.verdict_label, baseline.message)
            };
            if let Some(object) = value.as_object_mut() {
                object.insert("verdict_label".to_string(), json!(label));
                object.insert("message".to_string(), json!(message));
            }
            Some(value)
        })
        .collect();
    let mut payload = payload.clone();
    if let Some(object) = payload.as_object_mut() {
        object.insert("sessions".to_string(), Value::Array(judged));
    }
    payload
}

/// The words a prescribed verdict gets.
fn verdict_label(verdict: &str) -> String {
    match verdict {
        "missed_pace" => "missed pace".to_string(),
        "not_yet" => "not yet due".to_string(),
        "long" => "longer than planned".to_string(),
        other => other.to_string(),
    }
}

/// The run that answered a session on that day, if there was one.
fn activity_on<'a>(
    activities: &'a [stride_core::Activity],
    day: &str,
) -> Option<&'a stride_core::Activity> {
    activities.iter().find(|activity| {
        activity.local_date == day
            && matches!(
                activity.sport_type.as_str(),
                "running" | "trail_running" | "treadmill_running"
            )
    })
}

/// The run, as the plan sources want it: the date, the sport and the distance.
fn core_fact(row: &ActivityRow) -> stride_core::Activity {
    stride_core::Activity {
        id: row.id,
        name: row.name.clone(),
        local_date: row.local_date.clone(),
        start_date: row.local_date.clone(),
        day_of_week: row.day_of_week.clone(),
        sport_type: row.sport_type.clone(),
        source: row.source.clone(),
        distance_m: row.distance_m,
        moving_time_s: Some(row.moving_time_s),
        elapsed_time_s: Some(row.elapsed_time_s),
        ..stride_core::Activity::default()
    }
}

/// The plan the athlete wrote down, as Garmin's scheduled workouts.
///
/// With no Garmin key the source says so instead of pretending to be empty.
fn garmin_source(ctx: &Ctx<'_>, from: &str, to: &str) -> Result<Value, ApiError> {
    let unavailable = |reason: String| {
        json!({
            "source": "garmin",
            "prescribed": true,
            "description": stride_core::plan_sources::GARMIN_DESCRIPTION,
            "available": false,
            "reason": reason,
            "sessions": [],
        })
    };
    let Some(key) = ctx.backend.garmin_personal_key()? else {
        return Ok(unavailable(
            "No Garmin key is configured, so the scheduled workouts cannot be read.".to_string(),
        ));
    };
    let client = stride_garmin::GarminClient::new(key);
    let mut scheduled: Vec<serde_json::Value> = Vec::new();
    for (year, month) in months(from, to) {
        match client.scheduled_workouts(year, month) {
            Ok(rows) => {
                for row in rows {
                    scheduled.push(serde_json::to_value(row).unwrap_or(Value::Null));
                }
            }
            Err(err) => {
                return Ok(unavailable(format!(
                    "The scheduled workouts could not be read: {err}"
                )));
            }
        }
    }
    let mut details: Vec<serde_json::Value> = Vec::new();
    for row in &scheduled {
        let Some(id) = row.get("workout_id").and_then(Value::as_str) else {
            continue;
        };
        if let Ok(detail) = client.workout_detail(id) {
            details.push(serde_json::to_value(detail).unwrap_or(Value::Null));
        }
    }
    let scheduled: Vec<stride_core::plan_sources::ScheduledWorkout> = scheduled
        .iter()
        .filter_map(|row| serde_json::from_value(row.clone()).ok())
        .collect();
    let details: Vec<stride_core::plan_sources::WorkoutDetail> = details
        .iter()
        .filter_map(|row| serde_json::from_value(row.clone()).ok())
        .collect();
    let plan = stride_core::plan_sources::workouts_plan(&scheduled, &details);
    let mut payload = serde_json::to_value(&plan)
        .map_err(|err| ApiError::internal(format!("the Garmin plan did not render: {err}")))?;
    if let Some(object) = payload.as_object_mut() {
        object.insert("available".to_string(), json!(true));
        object.insert("reason".to_string(), Value::Null);
    }
    Ok(payload)
}

/// The months a range covers.
fn months(from: &str, to: &str) -> Vec<(i64, i64)> {
    use chrono::Datelike;
    let Ok(first) = chrono::NaiveDate::parse_from_str(&from[..from.len().min(10)], "%Y-%m-%d")
    else {
        return Vec::new();
    };
    let Ok(last) = chrono::NaiveDate::parse_from_str(&to[..to.len().min(10)], "%Y-%m-%d") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut year = first.year();
    let mut month = first.month();
    while (year, month) <= (last.year(), last.month()) {
        out.push((year as i64, month as i64));
        month += 1;
        if month > 12 {
            year += 1;
            month = 1;
        }
    }
    out
}

/// `GET /api/v1/plan/adherence` — how much of the plan was kept.
///
/// The reference: `api/plan.get_adherence`.
pub fn plan_adherence(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let (from, to) = plan_date_range(ctx, query)?;
    let mut summary = ctx.backend.plan_adherence(&from, &to)?;
    let counts = summary
        .get("by_verdict")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut by_verdict = Map::new();
    for verdict in ALL_VERDICTS {
        let value = counts.get(verdict).cloned().unwrap_or(json!(0));
        by_verdict.insert(verdict.to_string(), value);
    }
    if let Some(object) = summary.as_object_mut() {
        object.insert("by_verdict".into(), Value::Object(by_verdict));
    }
    Ok(summary)
}

/// `GET /api/v1/plan/today` — today's plan, today's readiness, one advice.
///
/// The reference: `api/plan.get_today`.
pub fn plan_today(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    // `?fields=` keeps the named keys of the answer: the week and the
    // readiness drivers are the heavy parts.
    let wanted = query.field_list("fields");
    let today = ctx.today.format("%Y-%m-%d").to_string();
    let block = active_block(ctx)?;
    let rows = ctx.backend.plan_sessions_with_results(&today, &today)?;
    let mut sessions = Vec::new();
    for row in &rows {
        let mut payload = shapes::plan_session_payload(&row.session);
        if let Some(object) = payload.as_object_mut() {
            object.insert("result".into(), row.result.clone());
            object.insert(
                "activity".into(),
                row.activity
                    .as_ref()
                    .map(shapes::plan_activity)
                    .unwrap_or(Value::Null),
            );
        }
        sessions.push(payload);
    }

    let week = ctx
        .backend
        .plan_blocks()?
        .into_iter()
        .flat_map(|block| {
            ctx.backend
                .plan_phases(block.id)
                .into_iter()
                .flatten()
                .filter_map(|phase| ctx.backend.plan_weeks(phase.id).ok())
                .flatten()
        })
        .filter(|week| {
            week.start_date
                .as_deref()
                .is_some_and(|start| start <= today.as_str())
                && week
                    .end_date
                    .as_deref()
                    .is_some_and(|end| end >= today.as_str())
        })
        .min_by_key(|week| week.week_number);
    let week_detail = match &week {
        Some(week) => week_detail(ctx, week, None)?,
        None => Value::Null,
    };

    let readiness = ctx.backend.readiness(&today).unwrap_or(Value::Null);

    if sessions.is_empty() {
        let (recommendation, rationale) = if block.is_none() {
            (
                "No session is planned for today. Import a plan to get a recommendation here.",
                "There is no planned session to compare against or adjust.",
            )
        } else {
            (
                "Nothing is planned for today — treat it as a rest day.",
                "Your plan does not schedule a session for today.",
            )
        };
        return Ok(shapes::pick(
            &json!({
                "date": today,
                "session": Value::Null,
                "week": week_detail,
                "readiness": readiness,
                "recommendation": recommendation,
                "rationale": rationale,
                "has_plan": block.is_some(),
                "has_session": false,
            }),
            wanted.as_ref(),
            None,
        ));
    }

    let session = sessions[0].clone();
    let (recommendation, rationale) = recommend(ctx, &session, week.as_ref(), &today, &readiness)?;
    Ok(shapes::pick(
        &json!({
            "date": today,
            "session": session,
            "week": week_detail,
            "readiness": readiness,
            "recommendation": recommendation,
            "rationale": rationale,
            "has_plan": true,
            "has_session": true,
        }),
        wanted.as_ref(),
        None,
    ))
}

/// One sentence of advice, with the reason behind it.
///
/// The reference: `api/plan._recommend`. Three rules the advice never breaks:
///
/// * a session already run today is done: the advice is to recover, not to
///   "complete" what is behind the athlete;
/// * a week behind its target is never a reason to run more later: missed
///   kilometres are not made up, because cramming them is how a single long
///   run spikes past what the legs are used to;
/// * only a readiness dip adjusts a session, and it only ever makes it easier.
fn recommend(
    _ctx: &Ctx<'_>,
    session: &Value,
    _week: Option<&crate::backend::PlanWeekRow>,
    today: &str,
    readiness: &Value,
) -> Result<(String, String), ApiError> {
    let title = session
        .get("title")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .or_else(|| session.get("session_type").and_then(Value::as_str))
        .unwrap_or("session")
        .to_string();
    let kind = session
        .get("session_type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if let Some(done) = session.get("activity").filter(|value| !value.is_null()) {
        if done.get("local_date").and_then(Value::as_str) == Some(today) {
            let km = done.get("distance_m").and_then(Value::as_f64).unwrap_or(0.0) / 1000.0;
            return Ok((
                format!(
                    "Done for today: {km:.1} km logged. Recover, and let tomorrow take care of itself."
                ),
                "Today's session already has a run against it.".to_string(),
            ));
        }
    }

    let mut reasons: Vec<String> = Vec::new();
    if let Some(score) = readiness.get("score").and_then(Value::as_f64) {
        if score < 50.0 {
            let grade = readiness
                .get("grade")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            reasons.push(format!("readiness is {:.0}/100 ({grade})", score));
        }
    }

    if matches!(kind, "rest" | "strength") {
        if let Some(first) = reasons.first() {
            return Ok((
                format!("Keep today's {kind} light — {first}."),
                format!("Adjusted because {}.", reasons.join("; and ")),
            ));
        }
        return Ok((
            format!("Nothing running is scheduled today; treat it as a planned {kind} day."),
            "Your plan schedules no running for today.".to_string(),
        ));
    }
    if let Some(first) = reasons.first() {
        return Ok((
            format!("Keep today's {title} easy and on the short side — {first}."),
            format!("Adjusted because {}.", reasons.join("; and ")),
        ));
    }
    Ok((
        format!("Complete the planned {title} as written."),
        "No readiness dip today.".to_string(),
    ))
}

// -------------------------------------------------------------- physiology

/// The physiology date window, with FastAPI's check.
///
/// The reference: `api/physiology.date_range`. The default window is a year:
/// Garmin keeps roughly a year of wellness history.
fn date_range(ctx: &Ctx<'_>, query: &Query) -> Result<(String, String), ApiError> {
    let last = query
        .optional("to")
        .map(str::to_string)
        .unwrap_or_else(|| ctx.today.format("%Y-%m-%d").to_string());
    let first = query
        .optional("from")
        .map(str::to_string)
        .unwrap_or_else(|| {
            (ctx.today - chrono::Duration::days(364))
                .format("%Y-%m-%d")
                .to_string()
        });
    if first > last {
        return Err(validation_error(
            vec!["query", "from"],
            "from must not be after to",
            "value_error",
        ));
    }
    Ok((first, last))
}

/// `GET /api/v1/physiology/daily` — the daily rows, optionally narrowed.
///
/// The reference: `api/physiology.daily`.
///
/// The default answer is the ten fields the Body screen draws — the sleep,
/// the HRV and the resting heart rate, and nothing else. A year of every
/// Garmin column is 189,811 bytes and a year of those ten is 70,091, and
/// the phone is the one that pays for the difference.
///
/// A caller that names its own fields still gets them, including the ones
/// the default leaves out: the desktop keeps its table, and the harness
/// tool reads the source of a day.
pub fn physiology_daily(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let wanted = query.field_list("fields");
    let (from, to) = date_range(ctx, query)?;
    let rows = ctx.backend.physiology(&from, &to)?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            shapes::pick(
                &shapes::physiology_payload(row),
                wanted.as_ref(),
                Some(shapes::BODY_FIELDS.as_slice()),
            )
        })
        .collect();
    Ok(json!({ "items": items }))
}

/// `GET /api/v1/physiology/latest` — the most recent day and its readiness.
///
/// The reference: `api/physiology.latest`.
///
/// The day is trimmed to [`shapes::BODY_FIELDS`], the same ten fields the
/// daily rows answer with by default.
pub fn physiology_latest(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let wanted = query.field_list("fields");
    let Some(row) = ctx.backend.physiology_latest()? else {
        return Ok(json!({ "day": Value::Null, "readiness": Value::Null }));
    };
    let readiness = ctx.backend.readiness(&row.date).unwrap_or(Value::Null);
    Ok(json!({
        "day": shapes::pick(
            &shapes::physiology_payload(&row),
            wanted.as_ref(),
            Some(shapes::BODY_FIELDS.as_slice()),
        ),
        "readiness": readiness,
    }))
}

/// `GET /api/v1/physiology/sleep` — the sleep columns only.
///
/// The reference: `api/physiology.sleep`.
pub fn physiology_sleep(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let (from, to) = date_range(ctx, query)?;
    let rows = ctx.backend.physiology(&from, &to)?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            let fields = physiology_map(row);
            let mut out = Map::new();
            for field in [
                "date",
                "sleep_s",
                "sleep_score",
                "deep_sleep_s",
                "rem_s",
                "light_sleep_s",
                "awake_s",
            ] {
                out.insert(
                    field.to_string(),
                    fields.get(field).cloned().unwrap_or(Value::Null),
                );
            }
            Value::Object(out)
        })
        .collect();
    Ok(json!({ "items": items }))
}

/// `GET /api/v1/physiology/hrv` — the HRV series and its baselines.
///
/// The reference: `api/physiology.hrv`.
pub fn physiology_hrv(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let wanted = query.field_list("fields");
    let (from, to) = date_range(ctx, query)?;
    let rows = ctx.backend.physiology(&from, &to)?;
    let mut items = Vec::new();
    let mut latest_baseline = Value::Null;
    for row in &rows {
        let baseline = ctx.backend.baselines(&row.date).unwrap_or(Value::Null);
        let hrv_baseline = baseline.get("hrv_ms").cloned().unwrap_or(Value::Null);
        let rhr_baseline = baseline.get("resting_hr").cloned().unwrap_or(Value::Null);
        latest_baseline = hrv_baseline.clone();
        let item = json!({
            "date": row.date,
            "hrv_ms": row.hrv_ms,
            "status": row.hrv_status,
            "baseline": hrv_baseline,
            "resting_hr": row.resting_hr,
            "rhr_baseline": rhr_baseline,
        });
        items.push(shapes::pick(&item, wanted.as_ref(), None));
    }
    Ok(json!({ "items": items, "baseline": latest_baseline }))
}

/// `GET /api/v1/physiology/readiness` — the readiness of a date range.
///
/// The reference: `api/physiology.readiness`.
pub fn physiology_readiness(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let wanted = query.field_list("fields");
    let (from, to) = date_range(ctx, query)?;
    let rows = ctx.backend.readiness_range(&from, &to)?;
    let items: Vec<Value> = rows
        .iter()
        .map(|item| shapes::pick(item, wanted.as_ref(), None))
        .collect();
    Ok(json!({ "items": items }))
}

/// `GET /api/v1/physiology/sources` — what is actually stored.
///
/// The reference: `api/physiology.sources`. Stored measurements prove
/// availability; connection capability alone does not.
pub fn physiology_sources(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let rows = ctx.backend.physiology("0000-01-01", "9999-12-31")?;
    let status = ctx.backend.public_status()?;
    let populated: Vec<&crate::backend::PhysiologyRow> = rows
        .iter()
        .filter(|row| {
            let fields = physiology_map(row);
            PHYSIOLOGY_FIELDS
                .iter()
                .any(|field| fields.get(*field).is_some_and(|value| !value.is_null()))
        })
        .collect();

    let missing_note = if status.garmin_method.as_deref() == Some("developer_key") {
        "The Activity API developer key does not grant Health API access; connect a Garmin account."
    } else if !status.garmin_connected {
        "Connect a Garmin account and sync health data."
    } else {
        "No measurements stored for this family; sync health data and check device support."
    };
    let reason = if populated.is_empty() {
        Some(missing_note.to_string())
    } else {
        None
    };

    let mut endpoints = Vec::new();
    for (name, columns) in SOURCE_FAMILIES {
        let available = rows.iter().any(|row| {
            let fields = physiology_map(row);
            columns
                .iter()
                .any(|column| fields.get(*column).is_some_and(|value| !value.is_null()))
        });
        endpoints.push(json!({
            "name": name,
            "available": available,
            "note": if available {
                "Stored measurements available."
            } else {
                missing_note
            },
        }));
    }

    let last_synced = populated
        .iter()
        .filter_map(|row| row.updated_at.clone())
        .max();

    Ok(json!({
        "available": !populated.is_empty(),
        "reason": reason,
        "auth_method": status.garmin_method,
        "endpoints": endpoints,
        "last_synced_at": last_synced,
        "days_available": populated.len(),
    }))
}

// ---------------------------------------------------------------- fitness

/// `GET /api/v1/fitness/fingerprint` — the fingerprints of a date range.
///
/// The reference: `api/fitness.fingerprints`.
pub fn fitness_fingerprint(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let (from, to) = date_range(ctx, query)?;
    // A fingerprint row carries 29 fields and the list screen draws 13.
    // `?fields=` keeps the ones a client names.
    let wanted = query.field_list("fields");
    let items: Vec<Value> = ctx
        .backend
        .fingerprints(&from, &to)?
        .iter()
        .map(|item| shapes::pick(item, wanted.as_ref(), None))
        .collect();
    Ok(json!({ "items": items }))
}

/// `GET /api/v1/fitness/activity/{id}` — the fingerprint of one run.
///
/// The reference: `api/fitness.activity_fingerprint`.
pub fn fitness_activity(ctx: &Ctx<'_>, id: i64, query: &Query) -> Result<Value, ApiError> {
    let item = ctx.backend.fingerprint(id)?;
    if item.is_null() || item.get("activity_id").is_none() {
        return Err(ApiError::not_found("Activity not found"));
    }
    let zones = zone_shares(ctx, id, &item)?;
    let mut payload = item;
    if let Some(object) = payload.as_object_mut() {
        object.insert("zones".into(), Value::Array(zones));
    }
    Ok(shapes::pick(
        &payload,
        query.field_list("fields").as_ref(),
        None,
    ))
}

/// The share of the run spent in each heart-rate zone.
///
/// The reference: `api/fitness._zone_shares`.
fn zone_shares(ctx: &Ctx<'_>, activity_id: i64, item: &Value) -> Result<Vec<Value>, ApiError> {
    let model = ctx.backend.athlete_zone_model(activity_id)?;
    let stored = ctx.backend.activity_zones(activity_id)?;
    let distribution = item
        .get("zone_distribution")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let total: f64 = distribution.values().filter_map(Value::as_f64).sum::<f64>();
    let mut zone_numbers: Vec<i64> = distribution
        .keys()
        .filter_map(|key| key.parse::<i64>().ok())
        .collect();
    for zone in &model {
        if let Some(zone_no) = zone.get("zone_no").and_then(Value::as_i64) {
            zone_numbers.push(zone_no);
        }
    }
    zone_numbers.sort_unstable();
    zone_numbers.dedup();

    let mut shares = Vec::new();
    for zone_no in zone_numbers {
        let mut metadata = model
            .iter()
            .find(|zone| zone.get("zone_no").and_then(Value::as_i64) == Some(zone_no))
            .cloned()
            .unwrap_or_else(|| {
                json!({
                    "zone_no": zone_no,
                    "name": format!("Zone {zone_no}"),
                    "low_bpm": Value::Null,
                    "high_bpm": Value::Null,
                })
            });
        if let Some(row) = stored.iter().find(|zone| zone.zone_no == zone_no) {
            metadata = json!({
                "zone_no": row.zone_no,
                "name": row.name,
                "low_bpm": row.low_bpm,
                "high_bpm": row.high_bpm,
            });
        }
        let seconds = distribution
            .get(&zone_no.to_string())
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let mut payload = metadata.as_object().cloned().unwrap_or_default();
        payload.insert("time_s".into(), json!(seconds));
        payload.insert(
            "percent".into(),
            json!(if total > 0.0 {
                seconds / total * 100.0
            } else {
                0.0
            }),
        );
        shares.push(Value::Object(payload));
    }
    Ok(shares)
}

/// `GET /api/v1/fitness/trends` — one metric over time.
///
/// The reference: `api/fitness.trends` and `services.fingerprint.fingerprint_trend`.
pub fn fitness_trends(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let metric = query.optional("metric").unwrap_or("decoupling_pct");
    let known = METRICS.iter().find(|(key, _, _)| *key == metric);
    let Some((metric, label, unit)) = known else {
        let names: Vec<&str> = METRICS.iter().map(|(key, _, _)| *key).collect();
        return Err(validation_error(
            vec!["query", "metric"],
            &format!("Unsupported metric; choose from {names:?}"),
            "value_error",
        ));
    };
    let (from, to) = date_range(ctx, query)?;
    let items = ctx.backend.fingerprints(&from, &to)?;

    let mut points: Vec<(String, i64, f64)> = Vec::new();
    for item in &items {
        let date = item
            .get("local_date")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let activity_id = item.get("activity_id").and_then(Value::as_i64).unwrap_or(0);
        let Some(value) = item.get(metric).and_then(Value::as_f64) else {
            continue;
        };
        if !value.is_finite() {
            continue;
        }
        points.push((date, activity_id, value));
    }
    points.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let half = points.len() / 2;
    let first: Vec<f64> = points[..half].iter().map(|point| point.2).collect();
    let second: Vec<f64> = points[half..].iter().map(|point| point.2).collect();
    let baseline = mean(&first);
    let recent = mean(&second);
    let trend = match (baseline, recent) {
        (Some(baseline), Some(recent)) if baseline != 0.0 => {
            Some((recent - baseline) / baseline.abs() * 100.0)
        }
        _ => None,
    };

    Ok(json!({
        "metric": metric,
        "points": points
            .iter()
            .map(|(date, activity_id, value)| json!({
                "date": date,
                "value": value,
                "activity_id": activity_id,
            }))
            .collect::<Vec<Value>>(),
        "baseline": baseline,
        "trend_pct": trend,
        "label": label,
        "unit": unit,
        "better": better_for(metric),
    }))
}

/// The direction that is good for a metric.
fn better_for(metric: &str) -> &'static str {
    match metric {
        "cadence_avg" | "efficiency_first" | "form_score" => "higher",
        "cadence_cv" | "hr_drift_pct" | "hr_drift_bpm" | "pace_drift_pct" | "decoupling_pct"
        | "pace_cv" => "lower",
        _ => "neutral",
    }
}

/// `GET /api/v1/fitness/zones` — the zone totals of a date range.
///
/// The reference: `api/fitness.zones`.
pub fn fitness_zones(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let (from, to) = date_range(ctx, query)?;
    let items = ctx.backend.fingerprints(&from, &to)?;
    let mut totals: Vec<(i64, Map<String, Value>)> = Vec::new();
    for item in &items {
        let Some(activity_id) = item.get("activity_id").and_then(Value::as_i64) else {
            continue;
        };
        for zone in zone_shares(ctx, activity_id, item)? {
            let zone_no = zone.get("zone_no").and_then(Value::as_i64).unwrap_or(0);
            let time = zone.get("time_s").and_then(Value::as_f64).unwrap_or(0.0);
            let position = totals.iter().position(|(number, _)| *number == zone_no);
            let index = match position {
                Some(index) => index,
                None => {
                    let mut map = zone.as_object().cloned().unwrap_or_default();
                    map.insert("time_s".into(), json!(0.0));
                    totals.push((zone_no, map));
                    totals.len() - 1
                }
            };
            let entry = &mut totals[index].1;
            let changed = ["low_bpm", "high_bpm"]
                .iter()
                .any(|field| entry.get(*field) != zone.get(*field));
            if changed {
                entry.insert("low_bpm".into(), Value::Null);
                entry.insert("high_bpm".into(), Value::Null);
            }
            let current = entry.get("time_s").and_then(Value::as_f64).unwrap_or(0.0);
            entry.insert("time_s".into(), json!(current + time));
        }
    }
    totals.sort_by_key(|(zone_no, _)| *zone_no);
    let total: f64 = totals
        .iter()
        .map(|(_, value)| value.get("time_s").and_then(Value::as_f64).unwrap_or(0.0))
        .sum();
    let zones: Vec<Value> = totals
        .into_iter()
        .map(|(_, mut value)| {
            let time = value.get("time_s").and_then(Value::as_f64).unwrap_or(0.0);
            value.insert(
                "percent".into(),
                json!(if total > 0.0 {
                    time / total * 100.0
                } else {
                    0.0
                }),
            );
            Value::Object(value)
        })
        .collect();
    Ok(json!({ "zones": zones, "total_time_s": total }))
}

/// `GET /api/v1/gear` — the shoes.
///
/// The reference: `api/fitness.gear`.
pub fn gear(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    Ok(json!({ "items": ctx.backend.gear()? }))
}

// ---------------------------------------------------------------- compare

/// `GET /api/v1/compare/candidates` — the runs that can be compared.
///
/// The reference: `api/compare.list_candidates`.
pub fn compare_candidates(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let limit = query.int("limit", 300, Some(1), Some(1000))?;
    Ok(json!({ "items": ctx.backend.compare_candidates(limit as usize)? }))
}

/// `GET /api/v1/compare` — two or more runs, side by side.
///
/// The reference: `api/compare.compare_runs`.
pub fn compare(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let raw = query.required("ids")?;
    let mut wanted: Vec<i64> = Vec::new();
    for chunk in raw.split(',') {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        if let Ok(id) = chunk.parse::<i64>() {
            wanted.push(id);
        }
    }
    if wanted.is_empty() {
        return Ok(json!({
            "runs": [],
            "parameters": parameters_json(),
            "missing": [],
        }));
    }
    wanted.truncate(8);
    let mut payload = ctx.backend.compare(&wanted)?;
    if let Some(object) = payload.as_object_mut() {
        object.insert("parameters".into(), parameters_json());
        object
            .entry("missing".to_string())
            .or_insert(Value::Array(Vec::new()));
    }
    // `?fields=` keeps the named fields of each run: the 120-point profile
    // is the heaviest part of a comparison.
    let fields = query.field_list("fields");
    if let Some(object) = payload.as_object_mut() {
        let runs = object
            .get("runs")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()));
        object.insert("runs".into(), trim_items(runs, fields.as_ref()));
    }
    Ok(payload)
}

/// The comparison matrix, as the API renders it.
fn parameters_json() -> Value {
    Value::Array(
        PARAMETERS
            .iter()
            .map(|(key, label, unit, better, format)| {
                json!({
                    "key": key,
                    "label": label,
                    "unit": unit,
                    "better": better,
                    "format": format,
                })
            })
            .collect(),
    )
}

// ----------------------------------------------------------------- tiles

/// `GET /api/v1/tiles/policy` — where the map tiles come from.
///
/// The reference: `api/tiles.tile_policy`.
pub fn tile_policy(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    Ok(ctx.backend.tile_policy()?)
}

/// The tile coordinates of a request, validated against the zoom's grid.
///
/// The reference: `api/tiles._coordinates`. `x` and `y` are indices into a
/// `2**zoom` grid, so at zoom 12 they run to 4095 — they are not small
/// numbers.
pub fn tile_coordinates(z: &str, x: &str, y: &str) -> Option<(i64, i64, i64)> {
    let zoom: i64 = z.parse().ok()?;
    let xi: i64 = x.parse().ok()?;
    let yi: i64 = y.parse().ok()?;
    if !(0..=19).contains(&zoom) {
        return None;
    }
    let limit = 1i64 << zoom;
    if xi < 0 || xi >= limit || yi < 0 || yi >= limit {
        return None;
    }
    Some((zoom, xi, yi))
}

/// `GET /api/v1/tiles/{z}/{x}/{y}.png` — one map tile.
///
/// The reference: `api/tiles.get_tile`. The tile is fetched by the athlete's
/// own machine and cached, so the browser never reveals its address to a
/// third party.
pub fn tile(ctx: &Ctx<'_>, params: &crate::router::Params) -> Result<crate::Response, ApiError> {
    let (z, x, y) = match crate::routes::tile_coordinates(
        params.get("z").unwrap_or_default(),
        params.get("x").unwrap_or_default(),
        params.get("y").unwrap_or_default(),
    ) {
        Some(coordinates) => coordinates,
        None => return Err(ApiError::bad_request("Invalid tile coordinates.")),
    };
    let tile = ctx.backend.tile(z, x, y)?;
    Ok(crate::Response::Binary {
        content_type: "image/png".to_string(),
        bytes: tile.bytes,
    })
}

/// The pace of an activity, for the places that need it.
pub fn activity_pace(row: &ActivityRow) -> Option<f64> {
    pace_s_per_km(row.distance_m, row.moving_time_s)
}

/// Turn a backend failure into the status the Python would have returned.
pub fn from_backend(err: BackendError) -> ApiError {
    ApiError::from(err)
}
