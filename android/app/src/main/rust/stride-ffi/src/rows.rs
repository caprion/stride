//! The mapping between the rows the store keeps and the rows the router
//! reads.
//!
//! Two shapes, one meaning:
//!
//! * `stride_store::model::*` is the table. It is what the database holds.
//! * `stride_api::backend::*` is the seam. It is what the router asks for.
//! * `stride_core::model::*` is the domain. The derived numbers are computed
//!   over it.
//!
//! The mapping is deliberately mechanical: a column becomes a field, a NULL
//! becomes `None`, and nothing is invented. The one place where a decision is
//! made is the heart-rate zone model, which the reference derives from the
//! athlete's observed maximum heart rate.

use std::collections::BTreeMap;

use serde_json::Value;

use stride_api::backend::{
    ActivityRow, DailyMetricRow, LapRow, PhysiologyRow, PlanBlockRow, PlanPhaseRow,
    PlanSessionRow, PlanWeekRow, StreamRow, ZoneRow,
};
use stride_core::model as core_model;

/// The store's activity, as the router reads it.
pub fn activity_row(activity: &stride_store::model::Activity) -> ActivityRow {
    ActivityRow {
        id: activity.id.unwrap_or_default(),
        source: activity.source.clone(),
        external_id: activity.external_id.clone(),
        name: activity.name.clone(),
        sport_type: activity.sport_type.clone(),
        description: activity.description.clone(),
        start_time: Some(activity.start_time.to_rfc3339()),
        local_date: activity.local_date.clone(),
        day_of_week: activity.day_of_week.clone(),
        distance_m: activity.distance_m,
        moving_time_s: activity.moving_time_s,
        elapsed_time_s: activity.elapsed_time_s,
        elevation_gain_m: activity.elevation_gain_m,
        avg_hr: activity.avg_hr,
        max_hr: activity.max_hr,
        calories: activity.calories,
        avg_cadence: activity.avg_cadence,
        max_cadence: activity.max_cadence,
        avg_speed_ms: activity.avg_speed_ms,
        max_speed_ms: activity.max_speed_ms,
        perceived_exertion: activity.perceived_exertion,
        device: activity.device.clone(),
        gear: activity.gear.clone(),
        is_trail: activity.is_trail,
        is_commute: activity.is_commute,
        running_time_s: activity.running_time_s,
        walking_time_s: activity.walking_time_s,
        stopped_time_s: activity.stopped_time_s,
        running_distance_m: activity.running_distance_m,
        geo_points: geo_points(activity.geo_points.as_ref()),
        lap_count: 0,
        has_geo: None,
    }
}

/// The route, as `[lat, lon]` pairs.
///
/// The column is JSON; the seam wants numbers. A row that is not a list of
/// pairs has no route, which is what `None` means.
pub fn geo_points(raw: Option<&Value>) -> Option<Vec<[f64; 2]>> {
    let points = raw?;
    let array = points.as_array()?;
    let mut out = Vec::with_capacity(array.len());
    for point in array {
        let pair = point.as_array()?;
        if pair.len() < 2 {
            continue;
        }
        let lat = pair.first().and_then(Value::as_f64)?;
        let lon = pair.get(1).and_then(Value::as_f64)?;
        out.push([lat, lon]);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// The store's split, as the router reads it.
pub fn lap_row(lap: &stride_store::model::ActivityLap) -> LapRow {
    LapRow {
        activity_id: lap.activity_id,
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
        elapsed_s: lap.elapsed_s,
    }
}

/// The store's series, as the router reads it.
pub fn stream_row(stream: &stride_store::model::ActivityStream) -> StreamRow {
    StreamRow {
        activity_id: stream.activity_id,
        kind: stream.kind.clone(),
        seconds: number_list(Some(&stream.seconds)),
        values: optional_number_list(Some(&stream.values)),
        label: stream.label.clone(),
        unit: stream.unit.clone(),
    }
}

/// A JSON array of numbers.
pub fn number_list(raw: Option<&Value>) -> Vec<f64> {
    raw.and_then(Value::as_array)
        .map(|array| array.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

/// A JSON array of numbers, where a null is an absent sample.
pub fn optional_number_list(raw: Option<&Value>) -> Vec<Option<f64>> {
    raw.and_then(Value::as_array)
        .map(|array| {
            array
                .iter()
                .map(|value| value.as_f64())
                .collect::<Vec<Option<f64>>>()
        })
        .unwrap_or_default()
}

/// The store's heart-rate zone, as the router reads it.
pub fn zone_row(zone: &stride_store::model::HrZone) -> ZoneRow {
    ZoneRow {
        activity_id: zone.activity_id,
        zone_no: zone.zone_no,
        name: zone.name.clone(),
        low_bpm: zone.low_bpm,
        high_bpm: zone.high_bpm,
        time_s: zone.time_s,
    }
}

/// The store's day of physiology, as the router reads it.
pub fn physiology_row(row: &stride_store::model::DailyPhysiology) -> PhysiologyRow {
    PhysiologyRow {
        date: row.date.clone(),
        sleep_s: row.sleep_s,
        sleep_score: row.sleep_score,
        deep_sleep_s: row.deep_sleep_s,
        rem_s: row.rem_s,
        light_sleep_s: row.light_sleep_s,
        awake_s: row.awake_s,
        hrv_ms: row.hrv_ms,
        hrv_status: row.hrv_status.clone(),
        resting_hr: row.resting_hr,
        max_hr: row.max_hr,
        stress_avg: row.stress_avg,
        stress_max: row.stress_max,
        body_battery_min: row.body_battery_min,
        body_battery_avg: row.body_battery_avg,
        body_battery_max: row.body_battery_max,
        steps: row.steps,
        training_readiness: row.training_readiness,
        acute_load: row.acute_load,
        chronic_load: row.chronic_load,
        vo2max: None,
        endurance_score: None,
        running_tolerance: None,
        source: Some("garmin_personal".to_string()),
        updated_at: None,
    }
}

/// The store's day of daily metrics, as the router reads it.
pub fn daily_metric_row(row: &stride_store::model::DailyMetric) -> DailyMetricRow {
    DailyMetricRow {
        date: row.date.clone(),
        resting_hr: row.resting_hr,
        max_hr: row.max_hr,
        sleep_s: row.sleep_s,
        deep_sleep_s: row.deep_sleep_s,
        hrv_ms: row.hrv_ms,
        stress: row.stress,
        steps: row.steps,
        body_battery: row.body_battery,
        readiness: row.readiness,
        distance_m: row.distance_m,
    }
}

/// The store's block, as the router reads it.
pub fn plan_block_row(block: &stride_store::model::PlanBlock) -> PlanBlockRow {
    PlanBlockRow {
        id: block.id.unwrap_or_default(),
        name: block.name.clone(),
        start_date: block.start_date.clone(),
        end_date: block.end_date.clone(),
        goal: block.goal.clone(),
        status: block.status.clone(),
        source: block.source.clone(),
        notes: block.notes.clone(),
    }
}

/// The store's phase, as the router reads it.
pub fn plan_phase_row(phase: &stride_store::model::PlanPhase) -> PlanPhaseRow {
    PlanPhaseRow {
        id: phase.id.unwrap_or_default(),
        block_id: phase.block_id,
        name: phase.name.clone(),
        start_date: phase.start_date.clone(),
        end_date: phase.end_date.clone(),
        target_weekly_km_min: phase.target_weekly_km_min,
        target_weekly_km_max: phase.target_weekly_km_max,
        target_sessions: phase.target_sessions,
        pace_easy_low: phase.pace_easy_low,
        pace_easy_high: phase.pace_easy_high,
        pace_tempo_low: phase.pace_tempo_low,
        pace_tempo_high: phase.pace_tempo_high,
        pace_interval_low: phase.pace_interval_low,
        pace_interval_high: phase.pace_interval_high,
        pace_long_low: phase.pace_long_low,
        pace_long_high: phase.pace_long_high,
        hr_zone_target: phase.hr_zone_target,
        notes: phase.notes.clone(),
    }
}

/// The store's week, as the router reads it.
pub fn plan_week_row(week: &stride_store::model::PlanWeek) -> PlanWeekRow {
    PlanWeekRow {
        id: week.id.unwrap_or_default(),
        phase_id: week.phase_id,
        week_number: week.week_number,
        start_date: week.start_date.clone(),
        end_date: week.end_date.clone(),
        target_km: week.target_km,
        key_workout: week.key_workout.clone(),
        notes: week.notes.clone(),
    }
}

/// The store's planned session, as the router reads it.
pub fn plan_session_row(session: &stride_store::model::PlanSession) -> PlanSessionRow {
    PlanSessionRow {
        id: session.id.unwrap_or_default(),
        week_id: session.week_id,
        block_id: session.block_id,
        date: session.date.clone(),
        session_type: session.session_type.clone(),
        title: session.title.clone(),
        details: session.details.clone(),
        target_distance_m: session.target_distance_m,
        pace_low_s_km: session.pace_low_s_km,
        pace_high_s_km: session.pace_high_s_km,
        hr_zone_target: session.hr_zone_target,
        status: session.status.clone(),
        source: session.source.clone(),
        notes: session.notes.clone(),
    }
}

/// The store's run, as the domain sees it.
///
/// The domain reads a run whole: the summary, the splits, the zones and the
/// series. This is the join that the domain functions expect.
pub fn core_activity(
    activity: &stride_store::model::Activity,
    laps: &[stride_store::model::ActivityLap],
    zones: &[stride_store::model::HrZone],
    streams: &[stride_store::model::ActivityStream],
) -> core_model::Activity {
    let mut streams_by_kind: BTreeMap<String, core_model::Stream> = BTreeMap::new();
    for stream in streams {
        streams_by_kind.insert(
            stream.kind.clone(),
            core_model::Stream {
                seconds: number_list(Some(&stream.seconds)),
                values: optional_number_list(Some(&stream.values)),
                unit: stream.unit.clone(),
            },
        );
    }

    core_model::Activity {
        id: activity.id.unwrap_or_default(),
        name: activity.name.clone(),
        local_date: activity.local_date.clone(),
        start_date: activity.start_time.format("%Y-%m-%d").to_string(),
        day_of_week: activity.day_of_week.clone(),
        sport_type: activity.sport_type.clone(),
        source: activity.source.clone(),
        distance_m: activity.distance_m,
        moving_time_s: Some(activity.moving_time_s),
        elapsed_time_s: Some(activity.elapsed_time_s),
        elevation_gain_m: Some(activity.elevation_gain_m),
        avg_hr: activity.avg_hr,
        max_hr: activity.max_hr,
        calories: activity.calories,
        running_time_s: activity.running_time_s,
        walking_time_s: activity.walking_time_s,
        stopped_time_s: activity.stopped_time_s,
        laps: laps.iter().map(core_lap).collect(),
        zones: zones
            .iter()
            .map(|zone| core_model::ZoneTime {
                zone_no: zone.zone_no,
                time_s: zone.time_s,
            })
            .collect(),
        streams: streams_by_kind,
        fingerprint: None,
    }
}

/// The store's split, as the domain sees it.
pub fn core_lap(lap: &stride_store::model::ActivityLap) -> core_model::Lap {
    core_model::Lap {
        lap_index: lap.lap_index,
        distance_m: Some(lap.distance_m),
        duration_s: Some(lap.duration_s),
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
        elapsed_s: lap.elapsed_s,
        pace_s_per_km: None,
    }
}

/// The store's day of physiology, as the domain sees it.
pub fn core_physiology(row: &stride_store::model::DailyPhysiology) -> core_model::DailyPhysiology {
    core_model::DailyPhysiology {
        date: row.date.clone(),
        sleep_s: row.sleep_s,
        sleep_score: row.sleep_score,
        deep_sleep_s: row.deep_sleep_s,
        rem_s: row.rem_s,
        light_sleep_s: row.light_sleep_s,
        awake_s: row.awake_s,
        hrv_ms: row.hrv_ms,
        hrv_status: row.hrv_status.clone(),
        resting_hr: row.resting_hr,
        max_hr: row.max_hr,
        stress_avg: row.stress_avg,
        stress_max: row.stress_max,
        body_battery_min: row.body_battery_min,
        body_battery_avg: row.body_battery_avg,
        body_battery_max: row.body_battery_max,
        steps: row.steps,
        training_readiness: row.training_readiness,
        acute_load: row.acute_load,
        chronic_load: row.chronic_load,
        vo2max: None,
        endurance_score: None,
        running_tolerance: None,
        source: Some("garmin_personal".to_string()),
    }
}

/// The store's planned session, as the domain sees it.
pub fn core_plan_session(session: &stride_store::model::PlanSession) -> core_model::PlanSession {
    core_model::PlanSession {
        id: session.id.unwrap_or_default(),
        date: session.date.clone(),
        session_type: session.session_type.clone(),
        title: session.title.clone(),
        details: session.details.clone(),
        target_distance_m: session.target_distance_m,
        pace_low_s_km: session.pace_low_s_km,
        pace_high_s_km: session.pace_high_s_km,
    }
}
