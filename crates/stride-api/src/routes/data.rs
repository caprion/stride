//! The data routes: the runs, the wellness, the fitness and the statistics.

use std::collections::BTreeMap;

use serde_json::Value;

use stride_core::band;
use stride_core::direction::day_directions;
use stride_core::pattern::{patterns, HistoryDay, PatternKind};
use stride_core::types::VitalKind;
use stride_core::units;
use stride_store::{ActivitySummary as StoredActivity, WellnessDaily};

use crate::dto::{
    ActivityDetail, ActivityListResponse, ActivitySummary, ElevationStats, ElevationWeek,
    Fingerprint, Fingerprints, HrvDay, HrvDays, LatestPhysiology, PaceTrends, PaceWeek,
    PersonalRecords, PhysiologyDay, PhysiologyDays, SleepDay, SleepDays, SourceInfo, Sources,
    StatsSummary, Totals, Trend, TrendPoint, Trends, VitalBand, VitalReading, VitalsHistory,
    VitalsSnapshot, WeekTotals, WeeklyStats, Zones,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::series;
use super::{activity_summary, core_date, ok, pace_of, today, week_start};

/// The most runs one answer may carry.
const MAX_RUNS_IN_ANSWER: usize = 500;

// ------------------------------------------------------------------- the feed

/// The feed: the runs, newest first, in pages.
pub fn activities(api: &Api, request: &Request) -> ApiResult<Value> {
    let limit = match request.query.integer("limit")? {
        Some(value) if value > 0 => value.min(api.config().max_feed_page as i64) as u32,
        Some(_) => return Err(ApiError::bad_request("the limit is not a number")),
        None => api.config().feed_page_size,
    };
    let cursor = match request.query.get("cursor") {
        Some(text) if !text.trim().is_empty() => Some(
            stride_store::FeedCursor::decode(text.trim())
                .map_err(|_| ApiError::bad_request("the cursor is not one this build wrote"))?,
        ),
        _ => None,
    };
    let since = match request.query.get("since") {
        Some(text) if !text.trim().is_empty() => Some(core_date(text.trim())?.to_iso()),
        _ => None,
    };

    let (runs, next_cursor) = feed(api, cursor, limit, since)?;
    let response = ActivityListResponse {
        activities: runs,
        next_cursor,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// One page of the feed.
///
/// The store pages the feed itself; a window (`since`) is cut here, over
/// the same order the store uses — the local date, then the id, both
/// descending.
fn feed(
    api: &Api,
    cursor: Option<stride_store::FeedCursor>,
    limit: u32,
    since: Option<String>,
) -> ApiResult<(Vec<ActivitySummary>, Option<String>)> {
    let store = api.store();
    let limit = limit.clamp(1, api.config().max_feed_page);
    match since {
        None => {
            let page = store
                .activity_feed(cursor.as_ref(), limit)
                .map_err(|error| api.store_error("the feed could not be read", &error))?;
            let next = page.next_cursor.map(|cursor| cursor.encode());
            let runs = page
                .activities
                .iter()
                .map(activity_summary)
                .collect::<Vec<ActivitySummary>>();
            Ok((runs, next))
        }
        Some(from) => {
            let to = today(api)?.to_iso();
            let rows = store
                .activities_between(&from, &to)
                .map_err(|error| api.store_error("the feed could not be read", &error))?;
            let kept: Vec<&StoredActivity> = rows
                .iter()
                .filter(|row| match &cursor {
                    None => true,
                    Some(cursor) => {
                        row.local_date.as_str() < cursor.local_date()
                            || (row.local_date == cursor.local_date() && row.id < cursor.id())
                    }
                })
                .collect();
            let more = kept.len() > limit as usize;
            let page: Vec<&StoredActivity> = kept.into_iter().take(limit as usize).collect();
            let next = match (more, page.last()) {
                (true, Some(last)) => {
                    Some(stride_store::FeedCursor::new(last.local_date.clone(), last.id).encode())
                }
                _ => None,
            };
            let runs = page
                .iter()
                .map(|row| activity_summary(row))
                .collect::<Vec<ActivitySummary>>();
            Ok((runs, next))
        }
    }
}

/// One run, in full.
pub fn activity(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = request.param_id("id")?;
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    let run = match run {
        Some(run) => run,
        None => {
            return Err(ApiError::not_found(format!(
                "there is no run with the id {id}"
            )))
        }
    };

    let mut summary = activity_summary(&run.summary());
    let mut splits = Vec::new();
    let mut elevation = Vec::new();
    let mut samples = Vec::new();
    if let Some(decoded) = series::decoded(api, id) {
        summary.geo_points = series::geo_points(&decoded);
        splits = series::splits(&decoded);
        elevation = series::elevation(&decoded, api.config().max_samples_in_response);
        samples = series::samples(&decoded, api.config().max_samples_in_response);
    }

    let plan_line = plan_line(api, &run.local_date, run.distance_m)?;
    let note = api
        .store()
        .notes_for_activity(id)
        .ok()
        .and_then(|notes| notes.into_iter().find_map(|note| note.text));

    let detail = ActivityDetail {
        summary,
        splits,
        elevation,
        samples,
        plan_line,
        note,
    };
    ok(&detail)
}

/// The plan's line under a run: what was planned and what was run.
fn plan_line(api: &Api, date: &str, actual_m: f64) -> ApiResult<Option<crate::dto::PlanLine>> {
    let sessions = api
        .store()
        .plan_sessions_for_day(date)
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    Ok(sessions
        .iter()
        .find_map(|session| session.target_distance_m)
        .map(|target| crate::dto::PlanLine {
            actual: render::distance(Some(actual_m)),
            target: render::distance(Some(target)),
        }))
}

// --------------------------------------------------------------- physiology

/// The window the wellness routes read.
fn window(api: &Api, request: &Request) -> ApiResult<(String, String)> {
    let current = today(api)?;
    let to = match request.query.get("to") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.to_iso(),
    };
    let from = match request.query.get("from") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.add_days(-api.config().health_history_days).to_iso(),
    };
    Ok((from, to))
}

/// The days of wellness.
pub fn physiology_daily(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let days: Vec<PhysiologyDay> = rows.iter().map(physiology_day).collect();
    let response = PhysiologyDays {
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The nights of sleep.
pub fn physiology_sleep(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let days: Vec<SleepDay> = rows
        .iter()
        .map(|row| SleepDay {
            date: row.date.clone(),
            sleep: render::duration(row.sleep_s),
            deep_sleep: render::duration(row.deep_sleep_s),
            rem_sleep: render::duration(row.rem_s),
            light_sleep: render::duration(row.light_sleep_s),
            awake: render::duration(row.awake_s),
        })
        .collect();
    let response = SleepDays {
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The mornings.
pub fn physiology_hrv(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let days: Vec<HrvDay> = rows
        .iter()
        .map(|row| HrvDay {
            date: row.date.clone(),
            hrv: render::hrv(row.hrv_ms),
            resting_hr: render::heart_rate(row.resting_hr),
        })
        .collect();
    let response = HrvDays {
        days,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The vitals and their directions — the snapshot Today and Body read.
pub fn physiology_vitals(api: &Api, request: &Request) -> ApiResult<Value> {
    let day = match request.query.get("date") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?,
        _ => today(api)?,
    };
    let rows = wellness_rows(api, day)?;
    let history = super::wellness_days(&rows);
    let directions = day_directions(&history, day);

    let vitals: Vec<VitalReading> = directions
        .vitals
        .iter()
        .map(|vital| VitalReading {
            key: vital_key(vital.vital).to_string(),
            label: vital.vital.label().to_string(),
            value_text: vital.value_text(),
            direction_text: vital.direction_text().map(|text| text.to_string()),
            deviation_text: vital.deviation_text(),
            travel_text: vital.travel_text(),
            status_text: vital.status_text(),
            band: vital.band.map(|band| VitalBand {
                lower: band.lower(),
                upper: band.upper(),
                nights: band.samples(),
            }),
        })
        .collect();

    let rest_line = {
        let line = directions.rest.line();
        if line.is_empty() {
            directions.rest.status_text()
        } else {
            Some(line)
        }
    };

    let history_rows = api
        .store()
        .wellness_before(&day.to_iso(), band::BAND_WINDOW_NIGHTS)
        .map_err(|error| api.store_error("the history could not be read", &error))?;
    let charts = VitalsHistory {
        sleep: history_rows
            .iter()
            .map(|row| crate::dto::HistoryPoint {
                date: row.date.clone(),
                value: row.sleep_s,
            })
            .collect(),
        hrv: history_rows
            .iter()
            .map(|row| crate::dto::HistoryPoint {
                date: row.date.clone(),
                value: row.hrv_ms,
            })
            .collect(),
        resting_hr: history_rows
            .iter()
            .map(|row| crate::dto::HistoryPoint {
                date: row.date.clone(),
                value: row.resting_hr,
            })
            .collect(),
    };

    let pattern_rows = history_days(api, &rows, day)?;
    let found = patterns(&pattern_rows, day);
    let pattern_readings: Vec<crate::dto::PatternReading> = found
        .iter()
        .map(|pattern| crate::dto::PatternReading {
            id: pattern_id(&pattern.kind),
            text: pattern.text.clone(),
            sample_size: pattern.sample_size,
        })
        .collect();

    let response = VitalsSnapshot {
        date: day.to_iso(),
        generated_at: api.generated_at(),
        rest: crate::dto::RestReading {
            label: stride_core::strings::text("vital.rest.label")
                .unwrap_or("Rest")
                .to_string(),
            line: rest_line,
        },
        vitals,
        history: charts,
        patterns: pattern_readings,
    };
    ok(&response)
}

/// The most recent day of wellness.
pub fn physiology_latest(api: &Api, _request: &Request) -> ApiResult<Value> {
    let current = today(api)?;
    let row = api
        .store()
        .latest_wellness(&current.to_iso())
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let response = LatestPhysiology {
        day: row.as_ref().map(physiology_day),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Where the wellness came from.
pub fn physiology_sources(api: &Api, request: &Request) -> ApiResult<Value> {
    let (from, to) = window(api, request)?;
    let rows = api
        .store()
        .wellness_between(&from, &to)
        .map_err(|error| api.store_error("the wellness could not be read", &error))?;
    let mut sources = Vec::new();
    for (name, count) in [
        (
            "sleep",
            rows.iter().filter(|row| row.sleep_s.is_some()).count(),
        ),
        (
            "hrv",
            rows.iter().filter(|row| row.hrv_ms.is_some()).count(),
        ),
        (
            "resting_hr",
            rows.iter().filter(|row| row.resting_hr.is_some()).count(),
        ),
    ] {
        let latest = rows
            .iter()
            .rev()
            .find(|row| match name {
                "sleep" => row.sleep_s.is_some(),
                "hrv" => row.hrv_ms.is_some(),
                _ => row.resting_hr.is_some(),
            })
            .map(|row| row.date.clone());
        sources.push(SourceInfo {
            name: name.to_string(),
            days: count as i64,
            latest,
        });
    }
    let response = Sources {
        sources,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// One day of wellness, as it travels.
fn physiology_day(row: &WellnessDaily) -> PhysiologyDay {
    PhysiologyDay {
        date: row.date.clone(),
        sleep: render::duration(row.sleep_s),
        deep_sleep: render::duration(row.deep_sleep_s),
        rem_sleep: render::duration(row.rem_s),
        light_sleep: render::duration(row.light_sleep_s),
        awake: render::duration(row.awake_s),
        hrv: render::hrv(row.hrv_ms),
        resting_hr: render::heart_rate(row.resting_hr),
        stress: render::whole(row.stress_avg),
        source: row.source.clone(),
    }
}

/// The wellness of the window that ends with the day.
fn wellness_rows(api: &Api, day: stride_core::date::Date) -> ApiResult<Vec<WellnessDaily>> {
    let from = day.add_days(-band::BAND_WINDOW_NIGHTS).to_iso();
    api.store()
        .wellness_between(&from, &day.to_iso())
        .map_err(|error| api.store_error("the wellness could not be read", &error))
}

/// The history the pattern engine reads: the wellness and the runs.
fn history_days(
    api: &Api,
    rows: &[WellnessDaily],
    day: stride_core::date::Date,
) -> ApiResult<Vec<HistoryDay>> {
    let from = day.add_days(-band::BAND_WINDOW_NIGHTS).to_iso();
    let runs = api
        .store()
        .activities_between(&from, &day.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let mut distances: BTreeMap<String, f64> = BTreeMap::new();
    for run in runs {
        *distances.entry(run.local_date.clone()).or_insert(0.0) += run.distance_m;
    }
    let mut days = Vec::new();
    for row in rows {
        let date = match core_date(&row.date) {
            Ok(date) => date,
            Err(_) => continue,
        };
        days.push(HistoryDay::new(
            date,
            super::vitals_of(row),
            distances.get(&row.date).copied(),
        ));
    }
    Ok(days)
}

/// The key a vital travels under.
fn vital_key(vital: VitalKind) -> &'static str {
    match vital {
        VitalKind::Sleep => "sleep",
        VitalKind::Hrv => "hrv",
        VitalKind::RestingHr => "resting_hr",
    }
}

/// The id a pattern travels under.
fn pattern_id(kind: &PatternKind) -> String {
    match kind {
        PatternKind::UsualStreak { vital, side } => {
            format!("usual_streak.{}.{}", vital_key(*vital), side.word())
        }
        PatternKind::ComparableOutcome { vital, side } => {
            format!("comparable_outcome.{}.{}", vital_key(*vital), side.word())
        }
    }
}

// ------------------------------------------------------------------ fitness

/// The fingerprints of the runs that have been synced.
///
/// **Endpoint only.** The data is already synced; the route has no screen,
/// and the plan says so in the OpenAPI document so that nobody builds one
/// by accident.
pub fn fitness_fingerprint(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let mut items = Vec::new();
    for run in runs.iter().take(MAX_RUNS_IN_ANSWER) {
        let decoded = match series::decoded(api, run.id) {
            Some(decoded) => decoded,
            None => continue,
        };
        items.push(Fingerprint {
            activity_id: run.id,
            local_date: run.local_date.clone(),
            distance: render::distance(Some(run.distance_m)),
            moving_time: render::duration(Some(run.moving_time_s)),
            avg_hr: render::heart_rate(run.avg_hr),
            max_hr: render::heart_rate(run.max_hr),
            avg_cadence: render::cadence(decoded.avg_cadence),
            pace: render::pace(pace_of(run.distance_m, run.moving_time_s)),
            elevation_gain: render::elevation(Some(run.elevation_gain_m)),
        });
    }
    let response = Fingerprints {
        items,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The measured trends.
pub fn fitness_trends(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let mut pace_points: Vec<TrendPoint> = Vec::new();
    let mut heart_points: Vec<TrendPoint> = Vec::new();
    for (week, week_runs) in super::weeks(&runs) {
        let paces: Vec<f64> = week_runs
            .iter()
            .filter_map(|run| pace_of(run.distance_m, run.moving_time_s))
            .collect();
        let hearts: Vec<f64> = week_runs.iter().filter_map(|run| run.avg_hr).collect();
        if let Some(pace) = super::median(&paces) {
            pace_points.push(TrendPoint {
                date: week.to_iso(),
                value: render::pace(Some(units::round_pace_s_per_km(pace))),
            });
        }
        if !hearts.is_empty() {
            let mean = hearts.iter().sum::<f64>() / hearts.len() as f64;
            heart_points.push(TrendPoint {
                date: week.to_iso(),
                value: render::heart_rate(Some(units::round_half_even(mean, 1))),
            });
        }
    }
    let trends = vec![
        Trend {
            metric: "median_pace".to_string(),
            label: "Median pace".to_string(),
            unit: "s/km".to_string(),
            baseline: render::pace(super::median(
                &pace_points
                    .iter()
                    .filter_map(|point| point.value.as_ref().and_then(|value| value.raw))
                    .collect::<Vec<f64>>(),
            )),
            points: pace_points,
        },
        Trend {
            metric: "avg_hr".to_string(),
            label: "Average heart rate".to_string(),
            unit: "bpm".to_string(),
            baseline: render::heart_rate(super::median(
                &heart_points
                    .iter()
                    .filter_map(|point| point.value.as_ref().and_then(|value| value.raw))
                    .collect::<Vec<f64>>(),
            )),
            points: heart_points,
        },
    ];
    let response = Trends {
        trends,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The heart-rate zones.
///
/// The frozen schema has no zones table, so the route answers with the
/// zones that are stored — none, until a sync writes them — rather than
/// with boundaries this crate would have invented.
pub fn fitness_zones(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let response = Zones {
        zones: Vec::new(),
        total_time_s: 0.0,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// -------------------------------------------------------------------- stats

/// The totals, over four windows.
pub fn stats_summary(api: &Api, request: &Request) -> ApiResult<Value> {
    let current = today(api)?;
    let year_start = current.add_days(-day_of_year(current) + 1).to_iso();
    let month_start = current.add_days(-(current.day() as i64) + 1).to_iso();
    let week = week_start(current).to_iso();

    let all = stored_runs(api, request)?;
    let this_year: Vec<&StoredActivity> = all
        .iter()
        .filter(|run| run.local_date >= year_start)
        .collect();
    let this_month: Vec<&StoredActivity> = all
        .iter()
        .filter(|run| run.local_date >= month_start)
        .collect();
    let this_week: Vec<&StoredActivity> = all.iter().filter(|run| run.local_date >= week).collect();

    let response = StatsSummary {
        all_time: totals(all.iter()),
        this_year: totals(this_year),
        this_month: totals(this_month),
        this_week: totals(this_week),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The totals of a set of runs.
fn totals<'a>(runs: impl IntoIterator<Item = &'a StoredActivity>) -> Totals {
    let runs: Vec<&StoredActivity> = runs.into_iter().collect();
    let distance_m: f64 = runs.iter().map(|run| run.distance_m).sum();
    let moving_time_s: f64 = runs.iter().map(|run| run.moving_time_s).sum();
    let hearts: Vec<f64> = runs.iter().filter_map(|run| run.avg_hr).collect();
    let avg_pace_s_per_km = if distance_m > 0.0 {
        Some(units::round_pace_s_per_km(
            moving_time_s / (distance_m / 1000.0),
        ))
    } else {
        None
    };
    Totals {
        activity_count: runs.len() as i64,
        distance: render::distance(Some(distance_m)),
        elevation_gain: render::elevation(Some(runs.iter().map(|run| run.elevation_gain_m).sum())),
        moving_time: render::duration(Some(moving_time_s)),
        avg_pace: render::pace(avg_pace_s_per_km),
        avg_hr: render::heart_rate(if hearts.is_empty() {
            None
        } else {
            Some(units::round_half_even(
                hearts.iter().sum::<f64>() / hearts.len() as f64,
                1,
            ))
        }),
    }
}

/// The weeks of running.
pub fn stats_weekly(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let weeks: Vec<WeekTotals> = super::weeks(&runs)
        .into_iter()
        .map(|(week, week_runs)| {
            let distance_m: f64 = week_runs.iter().map(|run| run.distance_m).sum();
            let moving_time_s: f64 = week_runs.iter().map(|run| run.moving_time_s).sum();
            WeekTotals {
                week_start: week.to_iso(),
                activity_count: week_runs.len() as i64,
                distance: render::distance(Some(distance_m)),
                moving_time: render::duration(Some(moving_time_s)),
                elevation_gain: render::elevation(Some(
                    week_runs.iter().map(|run| run.elevation_gain_m).sum(),
                )),
                avg_pace: render::pace(if distance_m > 0.0 {
                    Some(units::round_pace_s_per_km(
                        moving_time_s / (distance_m / 1000.0),
                    ))
                } else {
                    None
                }),
            }
        })
        .collect();
    let response = WeeklyStats {
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The pace, week by week.
pub fn stats_pace_trends(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let weeks: Vec<PaceWeek> = super::weeks(&runs)
        .into_iter()
        .map(|(week, week_runs)| {
            let paces: Vec<f64> = week_runs
                .iter()
                .filter_map(|run| pace_of(run.distance_m, run.moving_time_s))
                .collect();
            PaceWeek {
                week_start: week.to_iso(),
                run_count: week_runs.len() as i64,
                median_pace: render::pace(super::median(&paces).map(units::round_pace_s_per_km)),
                best_pace: render::pace(
                    paces
                        .iter()
                        .copied()
                        .fold(None, |best: Option<f64>, pace| {
                            Some(match best {
                                Some(current) => current.min(pace),
                                None => pace,
                            })
                        })
                        .map(units::round_pace_s_per_km),
                ),
            }
        })
        .collect();
    let response = PaceTrends {
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The climbing, week by week.
pub fn stats_elevation(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let weeks: Vec<ElevationWeek> = super::weeks(&runs)
        .into_iter()
        .map(|(week, week_runs)| ElevationWeek {
            week_start: week.to_iso(),
            elevation_gain: render::elevation(Some(
                week_runs.iter().map(|run| run.elevation_gain_m).sum(),
            )),
        })
        .collect();
    let response = ElevationStats {
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The personal records.
pub fn stats_personal_records(api: &Api, request: &Request) -> ApiResult<Value> {
    let runs = stored_runs(api, request)?;
    let records = super::personal_records(api, &runs)?;
    let response = PersonalRecords {
        records,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The runs the statistics routes read.
fn stored_runs(api: &Api, request: &Request) -> ApiResult<Vec<StoredActivity>> {
    let current = today(api)?;
    let to = match request.query.get("to") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.to_iso(),
    };
    let from = match request.query.get("from") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?.to_iso(),
        _ => current.add_days(-api.config().history_months * 30).to_iso(),
    };
    let runs = api
        .store()
        .activities_between(&from, &to)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    Ok(runs)
}

/// The day of the year, as the days of a year are counted.
fn day_of_year(day: stride_core::date::Date) -> i64 {
    let start = stride_core::date::Date::from_ymd(day.year(), 1, 1);
    match start {
        Some(start) => day.days_since(start) + 1,
        None => 1,
    }
}
