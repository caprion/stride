//! The response shapes.
//!
//! Every key here is in the same order and carries the same meaning as the
//! Python app, because the React app consumes them and is not being
//! rewritten. The references are `backend/app/api/activities.py`,
//! `backend/app/api/plan.py` and `backend/app/services/analytics.py`.

use serde_json::{json, Map, Value};

use crate::backend::{
    ActivityRow, LapRow, PhysiologyRow, PlanBlockRow, PlanPhaseRow, PlanSessionRow, PlanWeekRow,
    StreamRow, ZoneRow,
};
use crate::num::{duration_label, pace_label, pace_s_per_km, round_to};

/// The Garmin Connect link for a run, or `None` when the run did not come
/// from Garmin.
///
/// The reference: `api/activities._garmin_url`.
pub fn garmin_url(source: &str, external_id: &str) -> Option<String> {
    if !matches!(source, "garmin_personal" | "garmin" | "garmin_fit") {
        return None;
    }
    if external_id.is_empty() {
        return None;
    }
    Some(format!(
        "https://connect.garmin.com/modern/activity/{external_id}"
    ))
}

/// A decimated version of the track, for feed thumbnails.
///
/// The reference: `api/activities._route_preview`. Full tracks run to
/// thousands of points; a feed page shows dozens of cards, so the track is
/// sampled down to a fixed budget before it is shipped.
pub fn route_preview(points: Option<&Vec<[f64; 2]>>, limit: usize) -> Option<Value> {
    let points = points?;
    if points.len() < 2 {
        return None;
    }
    if points.len() <= limit {
        let all: Vec<Value> = points
            .iter()
            .map(|point| json!([point[0], point[1]]))
            .collect();
        return Some(Value::Array(all));
    }
    let step = points.len() as f64 / limit as f64;
    let mut sampled: Vec<Value> = Vec::with_capacity(limit + 1);
    for index in 0..limit {
        let pick = ((index as f64 * step) as usize).min(points.len() - 1);
        sampled.push(json!([points[pick][0], points[pick][1]]));
    }
    let last = points[points.len() - 1];
    sampled.push(json!([last[0], last[1]]));
    Some(Value::Array(sampled))
}

/// The summary of a run, as the activity list and the activity detail render it.
///
/// The reference: `api/activities._summary`.
pub fn activity_summary(row: &ActivityRow) -> Value {
    let pace = pace_s_per_km(row.distance_m, row.moving_time_s);
    let mut out = Map::new();
    out.insert("id".into(), json!(row.id));
    out.insert("source".into(), json!(row.source));
    out.insert("external_id".into(), json!(row.external_id));
    out.insert("name".into(), json!(row.name));
    out.insert("sport_type".into(), json!(row.sport_type));
    out.insert("description".into(), json!(row.description));
    out.insert("start_time".into(), json!(row.start_time));
    out.insert("local_date".into(), json!(row.local_date));
    out.insert("day_of_week".into(), json!(row.day_of_week));
    out.insert("distance_m".into(), json!(row.distance_m));
    out.insert("moving_time_s".into(), json!(row.moving_time_s));
    out.insert(
        "moving_time_label".into(),
        json!(duration_label(Some(row.moving_time_s))),
    );
    out.insert("elapsed_time_s".into(), json!(row.elapsed_time_s));
    out.insert("elevation_gain_m".into(), json!(row.elevation_gain_m));
    out.insert("avg_hr".into(), json!(row.avg_hr));
    out.insert("max_hr".into(), json!(row.max_hr));
    out.insert("calories".into(), json!(row.calories));
    out.insert("avg_cadence".into(), json!(row.avg_cadence));
    out.insert("max_cadence".into(), json!(row.max_cadence));
    out.insert("avg_speed_ms".into(), json!(row.avg_speed_ms));
    out.insert("max_speed_ms".into(), json!(row.max_speed_ms));
    out.insert(
        "pace_s_per_km".into(),
        json!(pace.map(|value| round_to(value, 2))),
    );
    out.insert("pace_label".into(), json!(pace_label(pace)));
    out.insert("perceived_exertion".into(), json!(row.perceived_exertion));
    out.insert("device".into(), json!(row.device));
    out.insert("gear".into(), json!(row.gear));
    out.insert("is_trail".into(), json!(row.is_trail));
    out.insert("is_commute".into(), json!(row.is_commute));
    out.insert(
        "garmin_url".into(),
        json!(garmin_url(&row.source, &row.external_id)),
    );
    out.insert("has_geo".into(), json!(row.has_geo()));
    out.insert(
        "route_preview".into(),
        json!(route_preview(row.geo_points.as_ref(), 90)),
    );
    out.insert("lap_count".into(), json!(row.lap_count));
    out.insert("running_time_s".into(), json!(row.running_time_s));
    out.insert("walking_time_s".into(), json!(row.walking_time_s));
    out.insert("stopped_time_s".into(), json!(row.stopped_time_s));
    out.insert("running_distance_m".into(), json!(row.running_distance_m));
    Value::Object(out)
}

/// The compact card the calendar and the statistics pages use.
///
/// The reference: `services.analytics._activity_card`.
pub fn activity_card(row: &ActivityRow) -> Value {
    let pace = pace_s_per_km(row.distance_m, row.moving_time_s);
    json!({
        "id": row.id,
        "source": row.source,
        "name": row.name,
        "sport_type": row.sport_type,
        "start_time": row.start_time,
        "local_date": row.local_date,
        "distance_m": row.distance_m,
        "moving_time_s": row.moving_time_s,
        "moving_time_label": duration_label(Some(row.moving_time_s)),
        "elevation_gain_m": row.elevation_gain_m,
        "avg_hr": row.avg_hr,
        "pace_s_per_km": pace.map(|value| round_to(value, 2)),
        "pace_label": pace_label(pace),
        "has_geo": row.has_geo(),
    })
}

/// The subset of a run the plan surface shows.
///
/// The reference: `api/plan._activity`. It is deliberately narrower than
/// [`activity_summary`]: no Garmin link, no route preview, no pace label.
pub fn plan_activity(row: &ActivityRow) -> Value {
    let pace = pace_s_per_km(row.distance_m, row.moving_time_s);
    json!({
        "id": row.id,
        "source": row.source,
        "external_id": row.external_id,
        "name": row.name,
        "sport_type": row.sport_type,
        "description": row.description,
        "start_time": row.start_time,
        "local_date": row.local_date,
        "day_of_week": row.day_of_week,
        "distance_m": row.distance_m,
        "moving_time_s": row.moving_time_s,
        "moving_time_label": duration_label(Some(row.moving_time_s)),
        "elapsed_time_s": row.elapsed_time_s,
        "elevation_gain_m": row.elevation_gain_m,
        "avg_hr": row.avg_hr,
        "max_hr": row.max_hr,
        "calories": row.calories,
        "avg_cadence": row.avg_cadence,
        "max_cadence": row.max_cadence,
        "avg_speed_ms": row.avg_speed_ms,
        "max_speed_ms": row.max_speed_ms,
        "pace_s_per_km": pace.map(|value| round_to(value, 2)),
        "pace_label": Value::Null,
        "perceived_exertion": row.perceived_exertion,
        "device": row.device,
        "gear": row.gear,
        "is_trail": row.is_trail,
        "is_commute": row.is_commute,
        "has_geo": row.has_geo(),
        "route_preview": Value::Null,
        "lap_count": row.lap_count,
    })
}

/// One split of a run.
///
/// The reference: `api/activities.get_activity`.
pub fn lap_payload(lap: &LapRow) -> Value {
    let pace = lap
        .avg_speed_ms
        .filter(|speed| *speed != 0.0)
        .map(|speed| 1000.0 / speed);
    json!({
        "lap_index": lap.lap_index,
        "distance_m": lap.distance_m,
        "distance_label": format!("{:.2} km", lap.distance_m / 1000.0),
        "duration_s": lap.duration_s,
        "duration_label": duration_label(Some(lap.duration_s)),
        "avg_speed_ms": lap.avg_speed_ms,
        "pace_s_per_km": pace.map(|value| round_to(value, 2)),
        "pace_label": pace_label(pace),
        "avg_hr": lap.avg_hr,
        "max_hr": lap.max_hr,
        "avg_cadence": lap.avg_cadence,
        "max_cadence": lap.max_cadence,
        "stride_length_m": lap.stride_length_m,
        "elevation_gain_m": lap.elevation_gain_m,
        "elevation_loss_m": lap.elevation_loss_m,
        "start_elevation_m": lap.start_elevation_m,
        "end_elevation_m": lap.end_elevation_m,
        "running_distance_m": lap.running_distance_m,
        "elapsed_s": lap.elapsed_s,
    })
}

/// One series of a run.
///
/// The reference: `api/activities.get_activity`.
pub fn stream_payload(stream: &StreamRow) -> Value {
    json!({
        "kind": stream.kind,
        "seconds": stream.seconds,
        "values": stream.values,
        "unit": stream.unit,
    })
}

/// One heart-rate zone of a run.
///
/// The reference: `api/activities.get_activity`.
pub fn zone_payload(zone: &ZoneRow) -> Value {
    json!({
        "zone_no": zone.zone_no,
        "name": zone.name,
        "low_bpm": zone.low_bpm,
        "high_bpm": zone.high_bpm,
        "time_s": zone.time_s,
        "time_label": duration_label(Some(zone.time_s)),
    })
}

/// One planned session.
///
/// The reference: `api/plan._session`.
pub fn plan_session_payload(session: &PlanSessionRow) -> Value {
    json!({
        "id": session.id,
        "week_id": session.week_id,
        "block_id": session.block_id,
        "date": session.date,
        "session_type": session.session_type,
        "title": session.title,
        "details": session.details,
        "target_distance_m": session.target_distance_m,
        "pace_low_s_km": session.pace_low_s_km,
        "pace_high_s_km": session.pace_high_s_km,
        "hr_zone_target": session.hr_zone_target,
        "status": session.status,
        "source": session.source,
        "notes": session.notes,
    })
}

/// One phase of a block.
///
/// The reference: `api/plan._phase`.
pub fn plan_phase_payload(phase: &PlanPhaseRow) -> Value {
    json!({
        "id": phase.id,
        "block_id": phase.block_id,
        "name": phase.name,
        "start_date": phase.start_date,
        "end_date": phase.end_date,
        "target_weekly_km_min": phase.target_weekly_km_min,
        "target_weekly_km_max": phase.target_weekly_km_max,
        "target_sessions": phase.target_sessions,
        "pace_easy_low": phase.pace_easy_low,
        "pace_easy_high": phase.pace_easy_high,
        "pace_tempo_low": phase.pace_tempo_low,
        "pace_tempo_high": phase.pace_tempo_high,
        "pace_interval_low": phase.pace_interval_low,
        "pace_interval_high": phase.pace_interval_high,
        "pace_long_low": phase.pace_long_low,
        "pace_long_high": phase.pace_long_high,
        "hr_zone_target": phase.hr_zone_target,
        "notes": phase.notes,
    })
}

/// One week of a phase.
///
/// The reference: `api/plan._week_row`.
pub fn plan_week_payload(week: &PlanWeekRow) -> Value {
    json!({
        "id": week.id,
        "phase_id": week.phase_id,
        "week_number": week.week_number,
        "start_date": week.start_date,
        "end_date": week.end_date,
        "target_km": week.target_km,
        "key_workout": week.key_workout,
        "notes": week.notes,
    })
}

/// One block, with the counts of what it contains.
///
/// The reference: `api/plan._block_row`.
pub fn plan_block_payload(
    block: &PlanBlockRow,
    phase_count: usize,
    week_count: usize,
    session_count: usize,
) -> Value {
    json!({
        "id": block.id,
        "name": block.name,
        "start_date": block.start_date,
        "end_date": block.end_date,
        "goal": block.goal,
        "status": block.status,
        "source": block.source,
        "notes": block.notes,
        "phase_count": phase_count,
        "week_count": week_count,
        "session_count": session_count,
    })
}

/// One physiology row, as the daily endpoint returns it.
///
/// The reference: `api/physiology._daily` — every column of the table.
pub fn physiology_payload(row: &PhysiologyRow) -> Value {
    json!({
        "date": row.date,
        "sleep_s": row.sleep_s,
        "sleep_score": row.sleep_score,
        "deep_sleep_s": row.deep_sleep_s,
        "rem_s": row.rem_s,
        "light_sleep_s": row.light_sleep_s,
        "awake_s": row.awake_s,
        "hrv_ms": row.hrv_ms,
        "hrv_status": row.hrv_status,
        "resting_hr": row.resting_hr,
        "max_hr": row.max_hr,
        "stress_avg": row.stress_avg,
        "stress_max": row.stress_max,
        "body_battery_min": row.body_battery_min,
        "body_battery_avg": row.body_battery_avg,
        "body_battery_max": row.body_battery_max,
        "steps": row.steps,
        "training_readiness": row.training_readiness,
        "acute_load": row.acute_load,
        "chronic_load": row.chronic_load,
        "vo2max": row.vo2max,
        "endurance_score": row.endurance_score,
        "running_tolerance": row.running_tolerance,
        "source": row.source,
        "updated_at": row.updated_at,
    })
}

/// The fields of a day that the Body screen draws.
///
/// The reference: `api/physiology.BODY_FIELDS`. Today's readiness, the sleep,
/// and the resting heart rate with its HRV — ten fields, in the column order
/// of the table.
///
/// The other Garmin columns — the body battery, the stress, the load, the
/// training status, the step count — are not among them. Nothing draws
/// them, and the plan engine does not read them.
pub const BODY_FIELDS: [&str; 10] = [
    "date",
    "sleep_s",
    "sleep_score",
    "deep_sleep_s",
    "rem_s",
    "light_sleep_s",
    "awake_s",
    "hrv_ms",
    "hrv_status",
    "resting_hr",
];

/// Keep only the requested fields of an item.
///
/// The reference: `api/physiology._pick`. Unknown names are ignored rather
/// than rejected, so a client asking for a field this version does not have
/// gets the rest of the row instead of an error.
///
/// `wanted` is the caller's list. `default` is the list to use when the
/// caller names none: `None` means the whole item, which is what the routes
/// that always answer in full do — and what the Body screen's routes do
/// not, because their default is [`BODY_FIELDS`].
pub fn pick(item: &Value, wanted: Option<&Vec<String>>, default: Option<&[&str]>) -> Value {
    let names: Vec<&str> = match wanted {
        Some(wanted) => wanted.iter().map(String::as_str).collect(),
        None => match default {
            Some(default) => default.to_vec(),
            None => return item.clone(),
        },
    };
    let Some(object) = item.as_object() else {
        return item.clone();
    };
    let mut out = Map::new();
    for (key, value) in object {
        if names.contains(&key.as_str()) {
            out.insert(key.clone(), value.clone());
        }
    }
    Value::Object(out)
}
