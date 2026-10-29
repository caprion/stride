//! The store-backed backend.
//!
//! `stride-store` holds the rows; this file turns them into the rows the
//! router reads. The conversion is mechanical — same column names, same
//! types — and it is here, in the crate that depends on both, so that
//! neither `stride-store` nor `stride-core` has to know about the API.
//!
//! The derived numbers — the fingerprint, the readiness, the plan verdicts —
//! are **not** computed here. They belong to `stride-core`, and
//! [`crate::core::CoreBackend`] adds them on top of this.

use std::sync::Arc;

use chrono::Datelike;

use serde_json::{json, Value};

use stride_store::repo::activity::{ActivityFilter as StoreFilter, ActivityOrder};
use stride_store::{Store, StoreError};

use crate::backend::{
    ActivityFilter, ActivityPatch, ActivityRow, Backend, Counts, DailyMetricRow, ImportRequest,
    LapRow, PhysiologyRow, PlanBlockFields, PlanBlockRow, PlanPhaseFields, PlanPhaseRow,
    PlanSessionFields, PlanSessionRow, PlanWeekFields, PlanWeekRow, SettingsPatch, StatusView,
    StreamRow, SyncRunRow, Tile, ZoneRow,
};
use crate::error::BackendError;

/// A store error, with the context the router shows.
fn store_error(context: &str, err: StoreError) -> BackendError {
    BackendError::Store(format!("{context}: {err}"))
}

/// The ISO form Python's `isoformat()` produces.
///
/// The reference renders `2026-03-02T06:15:00+00:00`, with the microseconds
/// only when there are any. `chrono`'s own formatter writes a `Z`, which is
/// the same moment but not the same text — and the React app reads the text.
pub fn python_iso(value: chrono::DateTime<chrono::Utc>) -> String {
    if value.timestamp_subsec_micros() == 0 {
        value.format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
    } else {
        value.format("%Y-%m-%dT%H:%M:%S%.6f+00:00").to_string()
    }
}

/// The points of a run, as `[lat, lon]` pairs.
fn geo_points(value: Option<&Value>) -> Option<Vec<[f64; 2]>> {
    let points = value?.as_array()?;
    let mut out = Vec::with_capacity(points.len());
    for point in points {
        let pair = point.as_array()?;
        if pair.len() < 2 {
            continue;
        }
        let lat = pair[0].as_f64()?;
        let lon = pair[1].as_f64()?;
        out.push([lat, lon]);
    }
    Some(out)
}

/// One store activity, as the router reads it.
fn activity_row(activity: &stride_store::Activity) -> ActivityRow {
    ActivityRow {
        id: activity.id.unwrap_or_default(),
        athlete_id: activity.athlete_id,
        source: activity.source.clone(),
        external_id: activity.external_id.clone(),
        name: activity.name.clone(),
        sport_type: activity.sport_type.clone(),
        description: activity.description.clone(),
        start_time: Some(python_iso(activity.start_time)),
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
        running_time_s: activity.running_time_s,
        walking_time_s: activity.walking_time_s,
        stopped_time_s: activity.stopped_time_s,
        running_distance_m: activity.running_distance_m,
        perceived_exertion: activity.perceived_exertion,
        device: activity.device.clone(),
        gear: activity.gear.clone(),
        is_trail: activity.is_trail,
        is_commute: activity.is_commute,
        geo_points: geo_points(activity.geo_points.as_ref()),
        lap_count: 0,
        has_geo: None,
    }
}

/// One store lap, as the router reads it.
fn lap_row(lap: &stride_store::ActivityLap) -> LapRow {
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

/// One store stream, as the router reads it.
fn stream_row(stream: &stride_store::ActivityStream) -> StreamRow {
    let seconds = stream
        .seconds
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(|value| value.as_f64().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    let values = stream
        .values
        .as_array()
        .map(|values| values.iter().map(|value| value.as_f64()).collect())
        .unwrap_or_default();
    StreamRow {
        activity_id: stream.activity_id,
        kind: stream.kind.clone(),
        seconds,
        values,
        label: stream.label.clone(),
        unit: stream.unit.clone(),
    }
}

/// One store zone, as the router reads it.
fn zone_row(zone: &stride_store::HrZone) -> ZoneRow {
    ZoneRow {
        activity_id: zone.activity_id,
        zone_no: zone.zone_no,
        name: zone.name.clone(),
        low_bpm: zone.low_bpm,
        high_bpm: zone.high_bpm,
        time_s: zone.time_s,
    }
}

/// One store physiology row, as the router reads it.
fn physiology_row(row: &stride_store::DailyPhysiology) -> PhysiologyRow {
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
        vo2max: row.vo2max,
        endurance_score: row.endurance_score,
        running_tolerance: row.running_tolerance,
        source: Some(row.source.clone()),
        updated_at: Some(python_iso(row.updated_at)),
    }
}

/// The store, seen as a backend.
impl Backend for Store {
    fn activities(&self, filter: &ActivityFilter) -> Result<Vec<ActivityRow>, BackendError> {
        let mut store_filter = StoreFilter {
            from: filter.date_from.clone(),
            to: filter.date_to.clone(),
            sport: filter.sport.clone(),
            sports: None,
            exclude_source: if filter.demo_visible {
                None
            } else {
                Some("demo".to_string())
            },
            search: filter.query.clone(),
            order: None,
            limit: None,
            offset: None,
        };
        if filter.sport.is_none() && !filter.all_sports {
            store_filter.sports = Some(
                crate::backend::RUN_SPORTS
                    .iter()
                    .map(|sport| sport.to_string())
                    .collect(),
            );
        }
        // The elevation order is not in the store's order enum, so the rows
        // are sorted here instead.
        let sort_in_rust = filter.sort == "elevation";
        if !sort_in_rust {
            store_filter.order = Some(match filter.sort.as_str() {
                "distance" => ActivityOrder::Distance,
                "pace" => ActivityOrder::Pace,
                _ => ActivityOrder::Date,
            });
            // A per-page of zero means "no limit" in the router's filter;
            // SQLite's LIMIT 0 would mean "no rows at all".
            store_filter.limit = if filter.per_page > 0 {
                Some(filter.per_page)
            } else {
                None
            };
            store_filter.offset = if filter.per_page > 0 {
                Some(((filter.page - 1).max(0)) * filter.per_page)
            } else {
                None
            };
        }
        let mut rows: Vec<ActivityRow> = self
            .activities_filtered(&store_filter)
            .map_err(|err| store_error("could not read the activities", err))?
            .iter()
            .map(activity_row)
            .collect();
        if sort_in_rust {
            rows.sort_by(|a, b| {
                b.elevation_gain_m
                    .partial_cmp(&a.elevation_gain_m)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if filter.per_page > 0 {
                let offset = ((filter.page - 1).max(0) * filter.per_page) as usize;
                rows = rows
                    .into_iter()
                    .skip(offset)
                    .take(filter.per_page as usize)
                    .collect();
            }
        }
        for row in &mut rows {
            let laps = self
                .activity_laps(row.id)
                .map_err(|err| store_error("could not read the splits", err))?;
            row.lap_count = laps.len() as i64;
        }
        Ok(rows)
    }

    fn activity_count(&self, filter: &ActivityFilter) -> Result<i64, BackendError> {
        let store_filter = StoreFilter {
            from: filter.date_from.clone(),
            to: filter.date_to.clone(),
            sport: filter.sport.clone(),
            sports: if filter.sport.is_none() && !filter.all_sports {
                Some(
                    crate::backend::RUN_SPORTS
                        .iter()
                        .map(|sport| sport.to_string())
                        .collect(),
                )
            } else {
                None
            },
            exclude_source: if filter.demo_visible {
                None
            } else {
                Some("demo".to_string())
            },
            search: filter.query.clone(),
            order: None,
            limit: None,
            offset: None,
        };
        self.count_activities(&store_filter)
            .map_err(|err| store_error("could not count the activities", err))
    }

    fn activity(&self, id: i64) -> Result<Option<ActivityRow>, BackendError> {
        let Some(activity) = self
            .activity(id)
            .map_err(|err| store_error("could not read the activity", err))?
        else {
            return Ok(None);
        };
        let mut row = activity_row(&activity);
        let laps = self
            .activity_laps(id)
            .map_err(|err| store_error("could not read the splits", err))?;
        row.lap_count = laps.len() as i64;
        Ok(Some(row))
    }

    fn activity_laps(&self, id: i64) -> Result<Vec<LapRow>, BackendError> {
        let laps = self
            .activity_laps(id)
            .map_err(|err| store_error("could not read the splits", err))?;
        Ok(laps.iter().map(lap_row).collect())
    }

    fn activity_streams(&self, id: i64) -> Result<Vec<StreamRow>, BackendError> {
        let streams = self
            .activity_streams(id)
            .map_err(|err| store_error("could not read the series", err))?;
        Ok(streams.iter().map(stream_row).collect())
    }

    fn activity_zones(&self, id: i64) -> Result<Vec<ZoneRow>, BackendError> {
        let zones = self
            .hr_zones(id)
            .map_err(|err| store_error("could not read the zones", err))?;
        Ok(zones.iter().map(zone_row).collect())
    }

    fn activity_geo(&self, id: i64) -> Result<Option<Vec<[f64; 2]>>, BackendError> {
        Ok(self
            .activity(id)
            .map_err(|err| store_error("could not read the activity", err))?
            .and_then(|activity| geo_points(activity.geo_points.as_ref())))
    }

    fn patch_activity(
        &self,
        id: i64,
        patch: &ActivityPatch,
    ) -> Result<Option<ActivityRow>, BackendError> {
        let Some(mut activity) = self
            .activity(id)
            .map_err(|err| store_error("could not read the activity", err))?
        else {
            return Ok(None);
        };
        if let Some(name) = &patch.name {
            activity.name = name.clone();
        }
        if let Some(description) = &patch.description {
            activity.description = Some(description.clone());
        }
        if let Some(exertion) = patch.perceived_exertion {
            activity.perceived_exertion = Some(exertion);
        }
        if let Some(is_trail) = patch.is_trail {
            activity.is_trail = is_trail;
        }
        if let Some(is_commute) = patch.is_commute {
            activity.is_commute = is_commute;
        }
        self.upsert_activity(&activity)
            .map_err(|err| store_error("could not write the activity", err))?;
        Ok(Some(activity_row(&activity)))
    }

    fn counts(&self) -> Result<Counts, BackendError> {
        let all = StoreFilter::default();
        let activities = self
            .count_activities(&all)
            .map_err(|err| store_error("could not count the activities", err))?;
        let real = self
            .count_activities(&StoreFilter {
                exclude_source: Some("demo".to_string()),
                ..StoreFilter::default()
            })
            .map_err(|err| store_error("could not count the activities", err))?;
        let physiology = self
            .physiology("0000-01-01", "9999-12-31")
            .map_err(|err| store_error("could not count the physiology", err))?
            .len() as i64;
        let sessions = self
            .plan_sessions("0000-01-01", "9999-12-31")
            .map_err(|err| store_error("could not count the sessions", err))?
            .len() as i64;
        Ok(Counts {
            activities,
            from_garmin: real,
            physiology_days: physiology,
            plan_sessions: sessions,
        })
    }

    fn activity_by_external(
        &self,
        source: &str,
        external_id: &str,
    ) -> Result<Option<ActivityRow>, BackendError> {
        let Some(activity) = self
            .activity_by_external(source, external_id)
            .map_err(|err| store_error("could not read the activity", err))?
        else {
            return Ok(None);
        };
        Ok(Some(activity_row(&activity)))
    }

    fn activity_raw(&self, id: i64) -> Result<Option<Value>, BackendError> {
        Ok(self
            .activity(id)
            .map_err(|err| store_error("could not read the activity", err))?
            .and_then(|activity| activity.raw))
    }

    fn upsert_synced_activity(
        &self,
        activity: &ActivityRow,
        laps: &[LapRow],
        streams: &[StreamRow],
    ) -> Result<i64, BackendError> {
        // The marker: an activity that has been rebuilt from its original FIT
        // file cannot change, so the next sync must not download it again.
        // The marker is the reference's `raw["fit_imported"]`.
        let existing_raw = self
            .activity_by_external(&activity.source, &activity.external_id)
            .ok()
            .flatten()
            .and_then(|row| row.raw);
        let raw = if laps.is_empty() && streams.is_empty() {
            // Nothing came from a FIT file, so the marker is left as it was.
            existing_raw
        } else {
            let mut object = existing_raw
                .as_ref()
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            object.insert("fit_imported".to_string(), json!(true));
            Some(Value::Object(object))
        };
        let row = stride_store::Activity {
            id: None,
            athlete_id: activity.athlete_id,
            source: activity.source.clone(),
            external_id: activity.external_id.clone(),
            name: activity.name.clone(),
            sport_type: activity.sport_type.clone(),
            description: activity.description.clone(),
            start_time: activity
                .start_time
                .as_deref()
                .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
                .map(|raw| raw.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
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
            running_time_s: activity.running_time_s,
            walking_time_s: activity.walking_time_s,
            stopped_time_s: activity.stopped_time_s,
            running_distance_m: activity.running_distance_m,
            perceived_exertion: activity.perceived_exertion,
            device: activity.device.clone(),
            gear: activity.gear.clone(),
            is_trail: activity.is_trail,
            is_commute: activity.is_commute,
            geo_points: activity.geo_points.as_ref().map(|points| json!(points)),
            raw,
            ..stride_store::Activity::default()
        };
        let id = self
            .upsert_activity(&row)
            .map_err(|err| store_error("could not write the activity", err))?;
        if !laps.is_empty() {
            let store_laps: Vec<stride_store::ActivityLap> = laps
                .iter()
                .map(|lap| stride_store::ActivityLap {
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
                    elapsed_s: lap.elapsed_s,
                    ..stride_store::ActivityLap::default()
                })
                .collect();
            self.replace_activity_laps(id, &store_laps)
                .map_err(|err| store_error("could not write the splits", err))?;
        }
        if !streams.is_empty() {
            let store_streams: Vec<stride_store::ActivityStream> = streams
                .iter()
                .map(|stream| stride_store::ActivityStream {
                    activity_id: id,
                    kind: stream.kind.clone(),
                    seconds: json!(stream.seconds),
                    values: json!(stream.values),
                    label: stream.label.clone(),
                    unit: stream.unit.clone(),
                    ..stride_store::ActivityStream::default()
                })
                .collect();
            self.replace_activity_streams(id, &store_streams)
                .map_err(|err| store_error("could not write the series", err))?;
        }
        Ok(id)
    }

    fn upsert_physiology(&self, row: &PhysiologyRow) -> Result<(), BackendError> {
        let physiology = stride_store::DailyPhysiology {
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
            vo2max: row.vo2max,
            endurance_score: row.endurance_score,
            running_tolerance: row.running_tolerance,
            source: row
                .source
                .clone()
                .unwrap_or_else(|| "garmin_personal".to_string()),
            updated_at: chrono::Utc::now(),
        };
        self.upsert_physiology(&physiology)
            .map_err(|err| store_error("could not write the physiology", err))
    }

    fn upsert_daily_metric(&self, row: &DailyMetricRow) -> Result<(), BackendError> {
        let metric = stride_store::DailyMetric {
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
        };
        self.upsert_daily_metric(&metric)
            .map_err(|err| store_error("could not write the daily metric", err))
    }

    fn upsert_sync_run(&self, run: &SyncRunRow) -> Result<i64, BackendError> {
        let record = stride_store::SyncRun {
            id: run.id,
            source: run.source.clone(),
            started_at: run
                .started_at
                .as_deref()
                .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
                .map(|raw| raw.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            finished_at: run
                .finished_at
                .as_deref()
                .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
                .map(|raw| raw.with_timezone(&chrono::Utc)),
            status: run.status.clone(),
            days: run.days,
            activities_found: run.activities_found,
            activities_new: run.activities_new,
            activities_updated: run.activities_updated,
            message: run.message.clone(),
            error: run.error.clone(),
        };
        self.upsert_sync_run(&record)
            .map_err(|err| store_error("could not write the sync log", err))
    }

    fn garmin_personal_key(&self) -> Result<Option<String>, BackendError> {
        // The key is in the state, not in the database.
        Ok(None)
    }

    fn physiology(&self, from: &str, to: &str) -> Result<Vec<PhysiologyRow>, BackendError> {
        let rows = self
            .physiology(from, to)
            .map_err(|err| store_error("could not read the physiology", err))?;
        Ok(rows.iter().map(physiology_row).collect())
    }

    fn physiology_latest(&self) -> Result<Option<PhysiologyRow>, BackendError> {
        Ok(self
            .latest_physiology()
            .map_err(|err| store_error("could not read the physiology", err))?
            .as_ref()
            .map(physiology_row))
    }

    fn daily_metrics(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<DailyMetricRow>, BackendError> {
        let rows = self
            .daily_metrics(from.unwrap_or("0000-01-01"), to.unwrap_or("9999-12-31"))
            .map_err(|err| store_error("could not read the daily metrics", err))?;
        Ok(rows
            .iter()
            .map(|row| DailyMetricRow {
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
            })
            .collect())
    }

    fn plan_blocks(&self) -> Result<Vec<PlanBlockRow>, BackendError> {
        let blocks = self
            .plan_blocks()
            .map_err(|err| store_error("could not read the plan blocks", err))?;
        Ok(blocks
            .iter()
            .map(|block| PlanBlockRow {
                id: block.id.unwrap_or_default(),
                name: block.name.clone(),
                start_date: block.start_date.clone(),
                end_date: block.end_date.clone(),
                goal: block.goal.clone(),
                status: block.status.clone(),
                source: block.source.clone(),
                notes: block.notes.clone(),
            })
            .collect())
    }

    fn plan_block(&self, id: i64) -> Result<Option<PlanBlockRow>, BackendError> {
        Ok(self
            .plan_block(id)
            .map_err(|err| store_error("could not read the plan block", err))?
            .as_ref()
            .map(|block| PlanBlockRow {
                id: block.id.unwrap_or_default(),
                name: block.name.clone(),
                start_date: block.start_date.clone(),
                end_date: block.end_date.clone(),
                goal: block.goal.clone(),
                status: block.status.clone(),
                source: block.source.clone(),
                notes: block.notes.clone(),
            }))
    }

    fn plan_phases(&self, block_id: i64) -> Result<Vec<PlanPhaseRow>, BackendError> {
        let phases = self
            .plan_phases(block_id)
            .map_err(|err| store_error("could not read the plan phases", err))?;
        Ok(phases
            .iter()
            .map(|phase| PlanPhaseRow {
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
            })
            .collect())
    }

    fn plan_weeks(&self, phase_id: i64) -> Result<Vec<PlanWeekRow>, BackendError> {
        let weeks = self
            .plan_weeks(phase_id)
            .map_err(|err| store_error("could not read the plan weeks", err))?;
        Ok(weeks
            .iter()
            .map(|week| PlanWeekRow {
                id: week.id.unwrap_or_default(),
                phase_id: week.phase_id,
                week_number: week.week_number,
                start_date: week.start_date.clone(),
                end_date: week.end_date.clone(),
                target_km: week.target_km,
                key_workout: week.key_workout.clone(),
                notes: week.notes.clone(),
            })
            .collect())
    }

    fn plan_sessions(&self, week_id: i64) -> Result<Vec<PlanSessionRow>, BackendError> {
        let sessions = self
            .plan_sessions("0000-01-01", "9999-12-31")
            .map_err(|err| store_error("could not read the plan sessions", err))?;
        Ok(sessions
            .iter()
            .map(session_row)
            .filter(|session| session.week_id == Some(week_id))
            .collect())
    }

    fn plan_sessions_in_range(
        &self,
        from: &str,
        to: &str,
    ) -> Result<Vec<PlanSessionRow>, BackendError> {
        let sessions = self
            .plan_sessions(from, to)
            .map_err(|err| store_error("could not read the plan sessions", err))?;
        Ok(sessions.iter().map(session_row).collect())
    }

    fn create_plan_block(&self, fields: &PlanBlockFields) -> Result<PlanBlockRow, BackendError> {
        let block = stride_store::PlanBlock {
            id: None,
            name: fields.name.clone(),
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            goal: fields.goal.clone(),
            status: fields
                .status
                .clone()
                .unwrap_or_else(|| "active".to_string()),
            source: fields
                .source
                .clone()
                .unwrap_or_else(|| "manual".to_string()),
            notes: fields.notes.clone(),
            created_at: chrono::Utc::now(),
        };
        let id = self
            .upsert_plan_block(&block)
            .map_err(|err| store_error("could not write the plan block", err))?;
        Ok(PlanBlockRow {
            id,
            name: block.name,
            start_date: block.start_date,
            end_date: block.end_date,
            goal: block.goal,
            status: block.status,
            source: block.source,
            notes: block.notes,
        })
    }

    fn create_plan_phase(
        &self,
        block_id: i64,
        fields: &PlanPhaseFields,
    ) -> Result<PlanPhaseRow, BackendError> {
        let phase = stride_store::PlanPhase {
            id: None,
            block_id,
            name: fields.name.clone(),
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            target_weekly_km_min: fields.target_weekly_km_min,
            target_weekly_km_max: fields.target_weekly_km_max,
            target_sessions: fields.target_sessions,
            pace_easy_low: fields.pace_easy_low,
            pace_easy_high: fields.pace_easy_high,
            pace_tempo_low: fields.pace_tempo_low,
            pace_tempo_high: fields.pace_tempo_high,
            pace_interval_low: fields.pace_interval_low,
            pace_interval_high: fields.pace_interval_high,
            pace_long_low: fields.pace_long_low,
            pace_long_high: fields.pace_long_high,
            hr_zone_target: fields.hr_zone_target,
            notes: fields.notes.clone(),
        };
        let id = self
            .upsert_plan_phase(&phase)
            .map_err(|err| store_error("could not write the plan phase", err))?;
        Ok(PlanPhaseRow {
            id,
            block_id,
            name: phase.name,
            start_date: phase.start_date,
            end_date: phase.end_date,
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
            notes: phase.notes,
        })
    }

    fn create_plan_week(
        &self,
        phase_id: i64,
        fields: &PlanWeekFields,
    ) -> Result<PlanWeekRow, BackendError> {
        let week = stride_store::PlanWeek {
            id: None,
            phase_id,
            week_number: fields.week_number,
            start_date: fields.start_date.clone(),
            end_date: fields.end_date.clone(),
            target_km: fields.target_km,
            key_workout: fields.key_workout.clone(),
            notes: fields.notes.clone(),
        };
        let id = self
            .upsert_plan_week(&week)
            .map_err(|err| store_error("could not write the plan week", err))?;
        Ok(PlanWeekRow {
            id,
            phase_id,
            week_number: week.week_number,
            start_date: week.start_date,
            end_date: week.end_date,
            target_km: week.target_km,
            key_workout: week.key_workout,
            notes: week.notes,
        })
    }

    fn create_plan_session(
        &self,
        week_id: i64,
        fields: &PlanSessionFields,
    ) -> Result<PlanSessionRow, BackendError> {
        let block_id = block_of_week(self, week_id).unwrap_or_default();
        let session = stride_store::PlanSession {
            id: None,
            week_id: Some(week_id),
            block_id: Some(block_id),
            date: fields.date.clone(),
            session_type: fields.session_type.clone(),
            title: fields.title.clone(),
            details: fields.details.clone(),
            target_distance_m: fields.target_distance_m,
            pace_low_s_km: fields.pace_low_s_km,
            pace_high_s_km: fields.pace_high_s_km,
            hr_zone_target: fields.hr_zone_target,
            status: fields
                .status
                .clone()
                .unwrap_or_else(|| "planned".to_string()),
            source: fields
                .source
                .clone()
                .unwrap_or_else(|| "manual".to_string()),
            notes: fields.notes.clone(),
        };
        let id = self
            .upsert_plan_session(&session)
            .map_err(|err| store_error("could not write the plan session", err))?;
        Ok(PlanSessionRow {
            id,
            week_id: Some(week_id),
            block_id: Some(block_id),
            date: session.date,
            session_type: session.session_type,
            title: session.title,
            details: session.details,
            target_distance_m: session.target_distance_m,
            pace_low_s_km: session.pace_low_s_km,
            pace_high_s_km: session.pace_high_s_km,
            hr_zone_target: session.hr_zone_target,
            status: session.status,
            source: session.source,
            notes: session.notes,
        })
    }

    fn import_plan(&self, request: &ImportRequest) -> Result<Value, BackendError> {
        // The document is parsed by the importer and written by the store.
        let year = chrono::Datelike::year(&chrono::Utc::now());
        let plan = match crate::plan_import::parse_plan(
            &request.content,
            &request.format,
            request.name.as_deref(),
            year,
        ) {
            Ok(plan) => plan,
            Err(message) => {
                // A document that cannot be read is an answer, not a crash.
                return Ok(json!({
                    "ok": false,
                    "block_id": Value::Null,
                    "name": request.name,
                    "phases": 0,
                    "weeks": 0,
                    "sessions": 0,
                    "warnings": [],
                    "error": message,
                }));
            }
        };
        let answer = crate::plan_import::write_plan(self, &plan)?;
        // The anchoring: a plan that does not overlap the runs it governs
        // reports every session as missed.
        if request.anchor == "activities" {
            if let Some(block_id) = answer["block_id"].as_i64() {
                anchor_plan_to_history(self, block_id)?;
            }
        }
        Ok(answer)
    }

    fn fingerprint(&self, _activity_id: i64) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the fingerprint is computed by stride-core".to_string(),
        ))
    }

    fn fingerprints(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the fingerprint is computed by stride-core".to_string(),
        ))
    }

    fn readiness(&self, _day: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the readiness is computed by stride-core".to_string(),
        ))
    }

    fn readiness_range(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the readiness is computed by stride-core".to_string(),
        ))
    }

    fn baselines(&self, _day: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the baselines are computed by stride-core".to_string(),
        ))
    }

    fn compare(&self, _ids: &[i64]) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the comparison is computed by stride-core".to_string(),
        ))
    }

    fn compare_candidates(&self, _limit: usize) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the comparison is computed by stride-core".to_string(),
        ))
    }

    fn plan_results(&self, _from: &str, _to: &str) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the plan verdicts are computed by stride-core".to_string(),
        ))
    }

    fn plan_adherence(&self, _from: &str, _to: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the plan verdicts are computed by stride-core".to_string(),
        ))
    }

    fn plan_sessions_with_results(
        &self,
        _from: &str,
        _to: &str,
    ) -> Result<Vec<crate::backend::SessionWithResult>, BackendError> {
        Err(BackendError::Unsupported(
            "the plan verdicts are computed by stride-core".to_string(),
        ))
    }

    fn athlete_zone_model(&self, _activity_id: i64) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the zone model is computed by stride-core".to_string(),
        ))
    }

    fn max_hr_reference(&self, _activity_id: i64) -> Result<f64, BackendError> {
        Err(BackendError::Unsupported(
            "the maximum heart rate is computed by stride-core".to_string(),
        ))
    }

    fn public_status(&self) -> Result<StatusView, BackendError> {
        // The application state — the Garmin credentials, the pairing code,
        // the sync settings — lives beside the database, not in it: the
        // Python keeps it in `data/state.json`, and the phone keeps it in
        // the Android preferences. The store answers with the defaults the
        // reference starts from; a backend that owns the state overrides
        // this (see the workstream report for the proposed signature).
        //
        // The one thing the store *does* know is when the last sync
        // finished: the sync log is in the database. The dashboard's
        // "Synced 2 days ago" reads it, so it must not be null just because
        // the state layer is not in the stack.
        let last_sync_at = self.sync_runs(20).ok().and_then(|runs| {
            runs.iter()
                .find(|run| run.status == "ok")
                .and_then(|run| run.finished_at)
                .map(python_iso)
        });
        Ok(StatusView {
            athlete: json!({}),
            network_route: "auto".to_string(),
            sync_days: 90,
            history_months: Some(12),
            health_history_days: Some(90),
            auto_sync: true,
            sync_interval_minutes: 180,
            demo_visible: true,
            last_sync_at,
            ..StatusView::default()
        })
    }

    fn network_status(&self) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the network status is read from the machine, not from the database".to_string(),
        ))
    }

    fn public_identity(&self) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the public identity is read from the machine, not from the database".to_string(),
        ))
    }

    fn exit_nodes(&self) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the exit nodes are read from the machine, not from the database".to_string(),
        ))
    }

    fn set_network_route(&self, _route: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the network route is set on the machine, not in the database".to_string(),
        ))
    }

    fn sync_status(&self) -> Result<Value, BackendError> {
        // The sync log is in the database, so the status can be read out of
        // it: the newest run says whether a sync is running, when it started
        // and what it said.
        //
        // The reference keeps the start time and the message after the sync
        // has finished — `started_at` is when the last sync began, and the
        // message is what it said — so the newest run is the answer, not the
        // newest *finished* run.
        let runs = self
            .sync_runs(10)
            .map_err(|err| store_error("could not read the sync log", err))?;
        let newest = runs.first();
        let running = newest.map(|run| run.status == "running").unwrap_or(false);
        let started_at = newest
            .map(|run| json!(python_iso(run.started_at)))
            .unwrap_or(Value::Null);
        let message = if running {
            // A sync that is running has no message yet.
            Value::Null
        } else {
            newest
                .and_then(|run| run.message.clone())
                .map(Value::String)
                .unwrap_or(Value::Null)
        };
        Ok(json!({
            "running": running,
            "started_at": started_at,
            "message": message,
        }))
    }

    fn sync_runs(&self, limit: usize) -> Result<Vec<Value>, BackendError> {
        let runs = self
            .sync_runs(limit as i64)
            .map_err(|err| store_error("could not read the sync log", err))?;
        Ok(runs
            .iter()
            .map(|run| {
                json!({
                    "id": run.id,
                    "source": run.source,
                    "started_at": python_iso(run.started_at),
                    "finished_at": run.finished_at.map(python_iso),
                    "status": run.status,
                    "days": run.days,
                    "activities_found": run.activities_found,
                    "activities_new": run.activities_new,
                    "activities_updated": run.activities_updated,
                    "message": run.message,
                    "error": run.error,
                })
            })
            .collect())
    }

    fn run_sync(&self, _days: Option<i64>, _background: bool) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the sync is driven by the Garmin client, not by the store".to_string(),
        ))
    }

    fn load_history(
        &self,
        _activities: Option<i64>,
        _health_days: Option<i64>,
        _background: bool,
    ) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the sync is driven by the Garmin client, not by the store".to_string(),
        ))
    }

    fn update_settings(&self, _patch: &SettingsPatch) -> Result<StatusView, BackendError> {
        Err(BackendError::Unsupported(
            "the application state is not in the database".to_string(),
        ))
    }

    fn gear(&self) -> Result<Vec<Value>, BackendError> {
        let gear = self
            .gear()
            .map_err(|err| store_error("could not read the shoes", err))?;
        Ok(gear
            .iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "uuid": row.uuid,
                    "name": row.name,
                    "brand": row.brand,
                    "model": row.model,
                    "status": row.status,
                    "distance_m": row.distance_m,
                    "activity_count": row.activity_count,
                    "first_used": row.first_used,
                    "last_used": row.last_used,
                    "retire_at_m": row.retire_at_m,
                    "retired": row.distance_m >= row.retire_at_m,
                })
            })
            .collect())
    }

    fn pair_code(&self) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the pairing code is minted by the application state".to_string(),
        ))
    }

    fn pair(&self, _code: &str) -> Result<Option<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the pairing code is minted by the application state".to_string(),
        ))
    }

    fn devices(&self) -> Result<Vec<Value>, BackendError> {
        Err(BackendError::Unsupported(
            "the paired devices live in the application state".to_string(),
        ))
    }

    fn revoke_device(&self, _device_id: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the paired devices live in the application state".to_string(),
        ))
    }

    fn connect_personal_key(&self, _api_key: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the Garmin key is validated by the Garmin client".to_string(),
        ))
    }

    fn test_personal_key(&self, _api_key: &str) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the Garmin key is validated by the Garmin client".to_string(),
        ))
    }

    fn tile_policy(&self) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the tile policy is a property of the machine".to_string(),
        ))
    }

    fn tile(&self, _z: i64, _x: i64, _y: i64) -> Result<Tile, BackendError> {
        Err(BackendError::Unsupported(
            "the map tiles are fetched by the machine".to_string(),
        ))
    }

    fn garmin_oauth(&self, _action: &str, _body: &Value) -> Result<Value, BackendError> {
        Err(BackendError::Unsupported(
            "the Garmin account flow is driven by the Garmin client".to_string(),
        ))
    }

    fn import_fit(&self, filename: &str, bytes: &[u8]) -> Result<Value, BackendError> {
        // The file is parsed by `stride-fit` and written by the store: the
        // same two steps the Python's ingest layer takes.
        let parsed = stride_fit::parse_fit(bytes, filename).map_err(|err| {
            BackendError::Domain(format!(
                "the FIT file {filename} could not be parsed: {err}"
            ))
        })?;
        let existed = self
            .activity_by_external(&parsed.source, &parsed.external_id)
            .map_err(|err| store_error("could not read the activity", err))?
            .is_some();

        let activity = stride_store::Activity {
            source: parsed.source.clone(),
            external_id: parsed.external_id.clone(),
            name: parsed.name.clone(),
            sport_type: parsed.sport_type.clone(),
            start_time: parsed.start_time,
            local_date: parsed.local_date.clone(),
            day_of_week: Some(parsed.day_of_week.clone()),
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
            perceived_exertion: parsed.perceived_exertion.map(|value| value as i64),
            device: parsed.device.clone(),
            gear: parsed.gear.clone(),
            is_trail: parsed.is_trail,
            is_commute: parsed.is_commute,
            geo_points: parsed.geo_points.as_ref().map(|points| json!(points)),
            ..stride_store::Activity::default()
        };
        let id = self
            .upsert_activity(&activity)
            .map_err(|err| store_error("could not write the activity", err))?;

        let laps: Vec<stride_store::ActivityLap> = parsed
            .laps
            .iter()
            .map(|lap| stride_store::ActivityLap {
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
                ..stride_store::ActivityLap::default()
            })
            .collect();
        self.replace_activity_laps(id, &laps)
            .map_err(|err| store_error("could not write the splits", err))?;

        let streams: Vec<stride_store::ActivityStream> = parsed
            .streams
            .iter()
            .map(|(kind, stream)| stride_store::ActivityStream {
                activity_id: id,
                kind: kind.clone(),
                seconds: json!(stream.seconds),
                values: json!(stream.values),
                label: None,
                unit: Some(stream.unit.clone()),
                ..stride_store::ActivityStream::default()
            })
            .collect();
        self.replace_activity_streams(id, &streams)
            .map_err(|err| store_error("could not write the series", err))?;

        Ok(json!({
            "id": id,
            "name": activity.name,
            "date": activity.local_date,
            "distance_m": activity.distance_m,
            "created": !existed,
        }))
    }
}

/// One store session, as the router reads it.
fn session_row(session: &stride_store::PlanSession) -> PlanSessionRow {
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

/// Shift a plan so that its last week ends with the athlete's latest run.
///
/// Without this, a plan written for a past season never overlaps the runs it
/// is supposed to govern, and every session reads as "missed". The whole
/// block is shifted by whole weeks, so every internal relationship survives.
fn anchor_plan_to_history(store: &Store, block_id: i64) -> Result<(), BackendError> {
    let demo_visible = store
        .public_status()
        .map(|status| status.demo_visible)
        .unwrap_or(true);
    let mut filter = StoreFilter::default();
    if !demo_visible {
        filter.exclude_source = Some("demo".to_string());
    }
    let activities = store
        .activities_filtered(&filter)
        .map_err(|err| store_error("could not read the activities", err))?;
    let Some(latest) = activities
        .iter()
        .filter_map(|activity| activity.local_date.parse::<chrono::NaiveDate>().ok())
        .max()
    else {
        return Ok(());
    };
    let Some(block) = store
        .plan_block(block_id)
        .map_err(|err| store_error("could not read the plan block", err))?
    else {
        return Ok(());
    };
    let Some(plan_end) = block
        .end_date
        .as_deref()
        .and_then(|date| date.parse::<chrono::NaiveDate>().ok())
    else {
        return Ok(());
    };
    // The week the last run falls in ends on its Sunday.
    let latest_week_end =
        latest + chrono::Duration::days(6 - latest.weekday().num_days_from_monday() as i64);
    let shift_days = (latest_week_end - plan_end).num_days();
    let shift = crate::num::round_to(shift_days as f64 / 7.0, 0) as i64 * 7;
    if shift == 0 {
        return Ok(());
    }

    let move_date = |value: &str| -> Option<String> {
        if value.is_empty() {
            return None;
        }
        value.parse::<chrono::NaiveDate>().ok().map(|date| {
            (date + chrono::Duration::days(shift))
                .format("%Y-%m-%d")
                .to_string()
        })
    };

    // The block.
    if let Some(mut block) = store
        .plan_block(block_id)
        .map_err(|err| store_error("could not read the plan block", err))?
    {
        if let Some(start) = move_date(&block.start_date) {
            block.start_date = start;
        }
        let end = block.end_date.as_deref().and_then(move_date);
        block.end_date = end.or(block.end_date);
        store
            .upsert_plan_block(&block)
            .map_err(|err| store_error("could not write the plan block", err))?;
    }
    // The phases.
    for phase in store
        .plan_phases(block_id)
        .map_err(|err| store_error("could not read the plan phases", err))?
    {
        let mut phase = phase;
        let start = phase.start_date.as_deref().and_then(move_date);
        let end = phase.end_date.as_deref().and_then(move_date);
        phase.start_date = start.or(phase.start_date);
        phase.end_date = end.or(phase.end_date);
        store
            .upsert_plan_phase(&phase)
            .map_err(|err| store_error("could not write the plan phase", err))?;
    }
    // The weeks and the sessions.
    let mut weeks = Vec::new();
    for phase in store
        .plan_phases(block_id)
        .map_err(|err| store_error("could not read the plan phases", err))?
    {
        for week in store
            .plan_weeks(phase.id.unwrap_or_default())
            .map_err(|err| store_error("could not read the plan weeks", err))?
        {
            weeks.push(week);
        }
    }
    for week in &weeks {
        // The sessions are read with the week's *old* dates, before the week
        // moves: otherwise the week and its sessions fall apart.
        let (from, to) = (
            week.start_date.as_deref().unwrap_or("0000-01-01"),
            week.end_date.as_deref().unwrap_or("9999-12-31"),
        );
        let sessions = store
            .plan_sessions(from, to)
            .map_err(|err| store_error("could not read the plan sessions", err))?;
        let mut week = week.clone();
        let start = week.start_date.as_deref().and_then(move_date);
        let end = week.end_date.as_deref().and_then(move_date);
        week.start_date = start.or(week.start_date);
        week.end_date = end.or(week.end_date);
        store
            .upsert_plan_week(&week)
            .map_err(|err| store_error("could not write the plan week", err))?;
        for session in sessions {
            let mut session = session;
            session.date = move_date(&session.date).unwrap_or(session.date);
            store
                .upsert_plan_session(&session)
                .map_err(|err| store_error("could not write the plan session", err))?;
        }
    }
    Ok(())
}

/// The block a week belongs to.
fn block_of_week(store: &Store, week_id: i64) -> Option<i64> {
    for block in store.plan_blocks().ok()? {
        for phase in store.plan_phases(block.id.unwrap_or_default()).ok()? {
            for week in store.plan_weeks(phase.id.unwrap_or_default()).ok()? {
                if week.id == Some(week_id) {
                    return Some(block.id.unwrap_or_default());
                }
            }
        }
    }
    None
}

/// A store, wrapped so it can be handed to the router.
pub fn backend(store: Arc<Store>) -> Arc<dyn Backend> {
    store
}
