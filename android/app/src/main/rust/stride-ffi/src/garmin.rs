//! The Garmin pull, on the phone.
//!
//! The reference is `backend/app/services/personal_sync.py` and
//! `backend/app/ingest/garmin_personal.py`. **This is a first cut**, and it
//! says so:
//!
//! * it imports the activity summary and the splits, and the wellness days;
//! * it does not download the FIT file, so the exact walk/run/stand
//!   segmentation and the route are not available for a pulled run — only for
//!   a FIT the athlete imports by hand;
//! * it does not mirror every wellness field.
//!
//! The Garmin client itself is `stride-garmin`, which owns the token
//! lifecycle: the Personal API key is exchanged for a 24-hour bearer token,
//! the token is cached in memory, a 401 forces exactly one exchange and one
//! retry, and a second 401 is reported rather than retried. There is no
//! refresh token, and there is never a loop.
//!
//! The key is used here and dropped here. The Kotlin shell unwraps it from
//! the Keystore for the length of one pull.

use chrono::{Duration, NaiveDate, Utc};
use serde_json::{json, Value};

use stride_api::backend::BackendError;
use stride_fit::parse_fit;
use stride_garmin::GarminClient;
use stride_store::model as store_model;

use crate::backend::PhoneBackend;

/// The source every pulled run is stored under.
///
/// The reference uses `garmin_personal`; the same string keeps the desktop's
/// snapshot and the phone's database interchangeable.
pub const SOURCE: &str = "garmin_personal";

/// Check a Personal API key without saving it.
///
/// The reference's `validate()` does the same two calls: the token exchange,
/// then a profile read. Both have to succeed before the key is worth keeping.
pub fn validate(api_key: &str) -> Result<Value, BackendError> {
    let client = GarminClient::new(api_key);
    let profile = client.validate().map_err(garmin_error)?;
    Ok(json!({
        "id": profile.id,
        "name": profile.athlete_name(),
    }))
}

/// Which Garmin endpoints this build can read.
pub fn sources() -> Value {
    let client = GarminClient::new("");
    let sources = client.sources();
    let mut map = serde_json::Map::new();
    for (name, available) in sources {
        map.insert(name.to_string(), json!(available));
    }
    Value::Object(map)
}

/// Pull from Garmin into the phone's database.
///
/// The answer counts what happened, because the phone's sync status shows it:
/// how many runs were found, how many were new, how many days of wellness
/// arrived.
pub fn sync(backend: &PhoneBackend, api_key: &str, days: i64) -> Result<Value, BackendError> {
    let client = GarminClient::new(api_key);
    let days = days.clamp(1, 90);
    let end = Utc::now().date_naive();
    let start = end - Duration::days(days as i64 - 1);

    let summaries = client
        .activities(start.and_time(chrono::NaiveTime::MIN).and_utc(), end.and_time(chrono::NaiveTime::MAX).and_utc())
        .map_err(garmin_error)?;

    let mut found = 0usize;
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut errors: Vec<Value> = Vec::new();

    for summary in &summaries {
        found += 1;
        let existing = backend
            .store()
            .activity_by_external(SOURCE, &summary.activity_id)
            .ok()
            .flatten();

        // An activity that is already here is left alone: the phone does not
        // re-download what it has, and it does not overwrite the athlete's
        // own edits.
        if existing.is_some() {
            updated += 1;
            continue;
        }

        let detail = match client.activity_detail(&summary.activity_id) {
            Ok(detail) => detail,
            Err(err) => {
                errors.push(json!({
                    "activity_id": summary.activity_id,
                    "error": err.to_string().chars().take(200).collect::<String>(),
                }));
                continue;
            }
        };

        match import_detail(backend, &detail, &summary.activity_id) {
            Ok(_) => created += 1,
            Err(err) => errors.push(json!({
                "activity_id": summary.activity_id,
                "error": err.to_string().chars().take(200).collect::<String>(),
            })),
        }
    }

    let wellness_days = sync_wellness(backend, &client, start, end, &mut errors);

    let status = if errors.is_empty() { "success" } else { "partial" };
    write_sync_run(backend, days, found, created, updated, status, &errors)?;

    Ok(json!({
        "activities_found": found,
        "activities_new": created,
        "activities_updated": updated,
        "wellness_days": wellness_days,
        "status": status,
        "errors": errors,
    }))
}

/// Import one activity's detail as a run.
///
/// The mapping is the reference's `_canonical_from_detail`: the local start
/// time is authoritative, the sport comes from the activity type, and the
/// splits come from the lap summaries when Garmin sent them.
fn import_detail(
    backend: &PhoneBackend,
    detail: &Value,
    activity_id: &str,
) -> Result<i64, BackendError> {
    let summary = detail
        .get("summaryDTO")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let type_dto = detail.get("activityTypeDTO").and_then(Value::as_object);
    let metadata = detail.get("metadataDTO").and_then(Value::as_object);

    let name = detail
        .get("activityName")
        .and_then(Value::as_str)
        .unwrap_or("Run")
        .to_string();
    let sport_type = type_dto
        .and_then(|dto| dto.get("typeKey"))
        .and_then(Value::as_str)
        .unwrap_or("running")
        .to_string();

    let start_raw = summary
        .get("startTimeLocal")
        .and_then(Value::as_str)
        .or_else(|| detail.get("startTimeLocal").and_then(Value::as_str));
    let start = start_raw.and_then(parse_local_time);
    let local_date = start
        .map(|value| value.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let day_of_week = start.map(|value| value.format("%A").to_string());

    let activity = store_model::Activity {
        id: None,
        athlete_id: None,
        source: SOURCE.to_string(),
        external_id: activity_id.to_string(),
        name,
        sport_type,
        description: None,
        start_time: start.map(|value| value.and_utc()).unwrap_or_else(Utc::now),
        local_date,
        day_of_week,
        distance_m: number(&summary, "distance").unwrap_or(0.0),
        moving_time_s: number(&summary, "movingDuration")
            .or_else(|| number(&summary, "duration"))
            .unwrap_or(0.0),
        elapsed_time_s: number(&summary, "elapsedDuration")
            .or_else(|| number(&summary, "duration"))
            .unwrap_or(0.0),
        elevation_gain_m: number(&summary, "elevationGain").unwrap_or(0.0),
        avg_hr: number(&summary, "averageHR"),
        max_hr: number(&summary, "maxHR"),
        calories: number(&summary, "calories"),
        avg_cadence: number(&summary, "averageRunCadence"),
        max_cadence: number(&summary, "maxRunCadence"),
        avg_speed_ms: number(&summary, "averageSpeed"),
        max_speed_ms: number(&summary, "maxSpeed"),
        running_time_s: None,
        walking_time_s: None,
        stopped_time_s: None,
        running_distance_m: None,
        perceived_exertion: None,
        device: device_name(metadata),
        gear: None,
        is_trail: sport_type == "trail_running",
        is_commute: false,
        polyline: None,
        geo_points: None,
        raw: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let id = backend
        .store()
        .upsert_activity(&activity)
        .map_err(|err| BackendError::Store(err.to_string()))?;

    // The splits, when Garmin sent them.
    let laps = lap_summaries(detail);
    if !laps.is_empty() {
        backend
            .store()
            .replace_activity_laps(id, &laps)
            .map_err(|err| BackendError::Store(err.to_string()))?;
    }

    Ok(id)
}

/// The splits of a detail payload.
fn lap_summaries(detail: &Value) -> Vec<store_model::ActivityLap> {
    let Some(laps) = detail
        .get("lapSummaries")
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    laps.iter()
        .enumerate()
        .map(|(index, lap)| store_model::ActivityLap {
            id: None,
            activity_id: 0,
            lap_index: index as i64 + 1,
            distance_m: lap.get("distance").and_then(Value::as_f64).unwrap_or(0.0),
            duration_s: lap.get("duration").and_then(Value::as_f64).unwrap_or(0.0),
            avg_speed_ms: lap.get("averageSpeed").and_then(Value::as_f64),
            max_speed_ms: lap.get("maxSpeed").and_then(Value::as_f64),
            avg_hr: lap.get("averageHR").and_then(Value::as_f64),
            max_hr: lap.get("maxHR").and_then(Value::as_f64),
            avg_cadence: lap.get("averageRunCadence").and_then(Value::as_f64),
            max_cadence: lap.get("maxRunCadence").and_then(Value::as_f64),
            stride_length_m: None,
            elevation_gain_m: lap.get("elevationGain").and_then(Value::as_f64),
            elevation_loss_m: None,
            start_elevation_m: None,
            end_elevation_m: None,
            running_distance_m: None,
            elapsed_s: Some(lap.get("duration").and_then(Value::as_f64).unwrap_or(0.0)),
        })
        .collect()
}

/// Pull the wellness of a window, day by day.
///
/// The reference's `sync_wellness` does the same: the daily summary is the
/// backbone, and the sleep and HRV calls fill in the rest.
fn sync_wellness(
    backend: &PhoneBackend,
    client: &GarminClient,
    start: NaiveDate,
    end: NaiveDate,
    errors: &mut Vec<Value>,
) -> usize {
    let mut days = 0usize;
    let mut day = start;
    while day <= end {
        let key = day.format("%Y-%m-%d").to_string();
        let mut row = store_model::DailyPhysiology {
            date: key.clone(),
            ..Default::default()
        };

        match client.daily_summary(&key) {
            Ok(payload) => apply_daily_summary(&mut row, &payload),
            Err(err) => errors.push(json!({
                "day": key,
                "error": err.to_string().chars().take(200).collect::<String>(),
            })),
        }
        if let Ok(payload) = client.sleep(&key) {
            apply_sleep(&mut row, &payload);
        }
        if let Ok(payload) = client.hrv(&key) {
            apply_hrv(&mut row, &payload);
        }

        if backend
            .store()
            .upsert_physiology(&row)
            .is_ok()
        {
            days += 1;
        }
        day += Duration::days(1);
    }
    days
}

/// The daily summary's fields.
fn apply_daily_summary(row: &mut store_model::DailyPhysiology, payload: &Value) {
    row.resting_hr = payload
        .get("restingHeartRate")
        .and_then(Value::as_f64)
        .or_else(|| {
            payload
                .get("summaryDTO")
                .and_then(|dto| dto.get("restingHeartRate"))
                .and_then(Value::as_f64)
        });
    row.max_hr = payload
        .get("maxHeartRate")
        .and_then(Value::as_f64)
        .or_else(|| {
            payload
                .get("summaryDTO")
                .and_then(|dto| dto.get("maxHeartRate"))
                .and_then(Value::as_f64)
        });
    row.steps = payload.get("totalSteps").and_then(Value::as_f64).or_else(|| {
        payload
            .get("summaryDTO")
            .and_then(|dto| dto.get("totalSteps"))
            .and_then(Value::as_f64)
    });
    row.body_battery_avg = payload.get("bodyBatteryValue").and_then(Value::as_f64);
    row.stress_avg = payload.get("stressValue").and_then(Value::as_f64);
}

/// The sleep payload's fields.
fn apply_sleep(row: &mut store_model::DailyPhysiology, payload: &Value) {
    let total = payload
        .get("totalSleepSeconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            payload
                .get("dailySleepDTO")
                .and_then(|dto| dto.get("totalSleepSeconds"))
                .and_then(Value::as_f64)
        });
    row.sleep_s = total;
    row.sleep_score = payload.get("sleepScore").and_then(Value::as_f64).or_else(|| {
        payload
            .get("dailySleepDTO")
            .and_then(|dto| dto.get("sleepScore"))
            .and_then(Value::as_f64)
    });
    row.deep_sleep_s = payload
        .get("deepSleepSeconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            payload
                .get("dailySleepDTO")
                .and_then(|dto| dto.get("deepSleepSeconds"))
                .and_then(Value::as_f64)
        });
    row.rem_s = payload.get("remSleepSeconds").and_then(Value::as_f64).or_else(|| {
        payload
            .get("dailySleepDTO")
            .and_then(|dto| dto.get("remSleepSeconds"))
            .and_then(Value::as_f64)
    });
    row.light_sleep_s = payload
        .get("lightSleepSeconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            payload
                .get("dailySleepDTO")
                .and_then(|dto| dto.get("lightSleepSeconds"))
                .and_then(Value::as_f64)
        });
    row.awake_s = payload.get("awakeSleepSeconds").and_then(Value::as_f64).or_else(|| {
        payload
            .get("dailySleepDTO")
            .and_then(|dto| dto.get("awakeSleepSeconds"))
            .and_then(Value::as_f64)
    });
}

/// The HRV payload's fields.
fn apply_hrv(row: &mut store_model::DailyPhysiology, payload: &Value) {
    row.hrv_ms = payload
        .get("hrvSummary")
        .and_then(|dto| dto.get("lastNightAvg"))
        .and_then(Value::as_f64)
        .or_else(|| payload.get("lastNightAvg").and_then(Value::as_f64));
    row.hrv_status = payload
        .get("hrvSummary")
        .and_then(|dto| dto.get("status"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("status")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
}

/// Import a FIT file the athlete picked on the phone.
///
/// The parser is `stride-fit`, the same crate the golden fixtures test, so a
/// file that is wrong is an error and not a wrong run.
pub fn import_fit(
    backend: &PhoneBackend,
    filename: &str,
    bytes: &[u8],
) -> Result<Value, BackendError> {
    let parsed = parse_fit(bytes, filename).map_err(|err| BackendError::Domain(err.to_string()))?;

    let existing = backend
        .store()
        .activity_by_external(&parsed.source, &parsed.external_id)
        .ok()
        .flatten();

    let mut activity = store_model::Activity {
        id: existing.as_ref().and_then(|row| row.id),
        athlete_id: None,
        source: parsed.source.clone(),
        external_id: parsed.external_id.clone(),
        name: parsed.name.clone(),
        sport_type: parsed.sport_type.clone(),
        description: None,
        start_time: parsed.start_time,
        local_date: parsed.local_date.clone(),
        day_of_week: parsed.day_of_week.clone(),
        distance_m: parsed.distance_m,
        moving_time_s: parsed.moving_time_s,
        elapsed_time_s: parsed.elapsed_time_s,
        elevation_gain_m: parsed.elevation_gain_m,
        avg_hr: parsed.avg_hr,
        max_hr: parsed.max_hr,
        calories: parsed.calories,
        avg_cadence: parsed.avg_cadence,
        max_cadence: parsed.max_cadence,
        avg_speed_ms: parsed.avg_speed_ms,
        max_speed_ms: parsed.max_speed_ms,
        running_time_s: parsed.running_time_s,
        walking_time_s: parsed.walking_time_s,
        stopped_time_s: parsed.stopped_time_s,
        running_distance_m: parsed.running_distance_m,
        perceived_exertion: None,
        device: parsed.device.clone(),
        gear: None,
        is_trail: parsed.is_trail,
        is_commute: parsed.is_commute,
        polyline: None,
        geo_points: parsed
            .geo_points
            .as_ref()
            .map(|points| json!(points).to_string().into()),
        raw: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    activity.touch();

    let id = backend
        .store()
        .upsert_activity(&activity)
        .map_err(|err| BackendError::Store(err.to_string()))?;

    let laps: Vec<store_model::ActivityLap> = parsed
        .laps
        .iter()
        .map(|lap| store_model::ActivityLap {
            id: None,
            activity_id: id,
            lap_index: lap.lap_index,
            distance_m: lap.distance_m,
            duration_s: lap.duration_s,
            avg_speed_ms: lap.avg_speed_ms,
            max_speed_ms: lap.max_speed_ms,
            avg_hr: lap.avg_hr,
            max_hr: lap.max_hr,
            avg_cadence: lap.avg_cadence,
            max_cadence: lap.max_cadence,
            stride_length_m: lap.stride_length_m,
            elevation_gain_m: lap.elevation_gain_m,
            elevation_loss_m: lap.elevation_loss_m,
            start_elevation_m: lap.start_elevation_m,
            end_elevation_m: lap.end_elevation_m,
            running_distance_m: lap.running_distance_m,
            elapsed_s: Some(lap.elapsed_s),
        })
        .collect();
    backend
        .store()
        .replace_activity_laps(id, &laps)
        .map_err(|err| BackendError::Store(err.to_string()))?;

    for (kind, stream) in &parsed.streams {
        let seconds: Vec<i64> = stream.seconds.clone();
        let values: Vec<Option<f64>> = stream.values.iter().copied().map(Some).collect();
        let row = store_model::ActivityStream {
            id: None,
            activity_id: id,
            kind: kind.clone(),
            seconds: json!(seconds),
            values: json!(values),
            label: None,
            unit: Some(stream.unit.clone()),
        };
        backend
            .store()
            .upsert_activity_stream(&row)
            .map_err(|err| BackendError::Store(err.to_string()))?;
    }

    Ok(json!({
        "id": id,
        "name": parsed.name,
        "local_date": parsed.local_date,
        "distance_m": parsed.distance_m,
        "running_time_s": parsed.running_time_s,
        "walking_time_s": parsed.walking_time_s,
        "stopped_time_s": parsed.stopped_time_s,
        "running_distance_m": parsed.running_distance_m,
        "source": parsed.source,
        "external_id": parsed.external_id,
    }))
}

/// Write the run of a pull into the sync log.
fn write_sync_run(
    backend: &PhoneBackend,
    days: i64,
    found: usize,
    created: usize,
    updated: usize,
    status: &str,
    errors: &[Value],
) -> Result<(), BackendError> {
    let message = if errors.is_empty() {
        Some(format!(
            "{created} new, {updated} already known, {found} found"
        ))
    } else {
        Some(format!(
            "{created} new, {updated} already known, {found} found, {} errors",
            errors.len()
        ))
    };
    let run = store_model::SyncRun {
        id: None,
        source: SOURCE.to_string(),
        started_at: Utc::now(),
        finished_at: Some(Utc::now()),
        status: status.to_string(),
        days,
        activities_found: found as i64,
        activities_new: created as i64,
        activities_updated: updated as i64,
        message,
        error: (!errors.is_empty()).then(|| {
            errors
                .iter()
                .map(|error| {
                    error
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or("an error")
                        .to_string()
                })
                .collect::<Vec<String>>()
                .join("; ")
        }),
    };
    backend
        .store()
        .upsert_sync_run(&run)
        .map(|_| ())
        .map_err(|err| BackendError::Store(err.to_string()))
}

/// A number out of an object.
fn number(object: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
    object.get(key).and_then(Value::as_f64)
}

/// The watch that recorded the run.
fn device_name(metadata: Option<&serde_json::Map<String, Value>>) -> Option<String> {
    let metadata = metadata?;
    if let Some(sensors) = metadata.get("sensors").and_then(Value::as_array) {
        if let Some(first) = sensors.first().and_then(Value::as_object) {
            let manufacturer = first.get("manufacturer").and_then(Value::as_str);
            let product = first.get("product").and_then(Value::as_str);
            let name = format!(
                "{} {}",
                manufacturer.unwrap_or_default(),
                product.unwrap_or_default()
            )
            .trim()
            .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    metadata
        .get("manufacturer")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// A local time, as Garmin sends it.
fn parse_local_time(raw: &str) -> Option<chrono::NaiveDateTime> {
    for format in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
    ] {
        if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(raw, format) {
            return Some(parsed);
        }
    }
    None
}

/// The Garmin failure, as the backend's failure.
fn garmin_error(err: stride_garmin::GarminError) -> BackendError {
    match err {
        stride_garmin::GarminError::Unauthorized => BackendError::Domain(
            "Garmin rejected the key. It may have expired or been revoked.".to_string(),
        ),
        stride_garmin::GarminError::RateLimited { .. } => BackendError::Domain(
            "Garmin is rate limiting the phone. The pull will be retried later.".to_string(),
        ),
        other => BackendError::Domain(other.to_string()),
    }
}
