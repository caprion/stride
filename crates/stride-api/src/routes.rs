//! The handlers: one per route, and the helpers they share.
//!
//! Every handler answers with a value the router serialises, and every
//! handler is a **read** of the store or a **write** the store owns. No
//! handler opens a connection, writes SQL or asks what time it is.
//!
//! ## The helpers
//!
//! The helpers are where the arithmetic that more than one route needs
//! lives: the athlete's day, the week's first day, the pace of a run, the
//! sentence of a planned session. They all go through `stride-core` for
//! their rounding and their wording — the API never rounds a number on its
//! own and never writes a domain sentence of its own.

pub mod app;
pub mod data;
pub mod insight;
pub mod plan;
pub mod series;

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use serde_json::Value;
use stride_core::band;
use stride_core::date::Date;
use stride_core::statement;
use stride_core::types::Distance;
use stride_core::types::{Duration, Hrv, RestingHr, Sleep, Vitals, WellnessDay};
use stride_core::units;
use stride_store::{ActivitySummary as StoredActivity, WellnessDaily};

use crate::dto::{ActivitySummary, PersonalRecord};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::state::Api;

/// The most runs one answer may carry.
pub const MAX_RUNS_IN_ANSWER: usize = 500;

/// A value on its way out.
pub fn ok<T: Serialize>(value: &T) -> ApiResult<Value> {
    crate::router::success(value)
}

/// A date, as the core reads it.
pub fn core_date(iso: &str) -> ApiResult<Date> {
    Date::parse(iso).ok_or_else(|| ApiError::internal("a date could not be read"))
}

/// The athlete's day.
///
/// The day is the clock's day, moved by the offset the host says the
/// athlete's days are counted in. The clock is injected, so the day is
/// reproducible: the tests hold the clock still.
pub fn today(api: &Api) -> ApiResult<Date> {
    let offset = chrono::Duration::minutes(api.config().utc_offset_minutes as i64);
    let shifted = api.now() + offset;
    let naive = shifted.naive_utc();
    core_date(&naive.format("%Y-%m-%d").to_string())
}

/// A date, as chrono reads it.
pub fn naive_of(day: Date) -> NaiveDate {
    NaiveDate::from_ymd_opt(day.year(), day.month(), day.day()).unwrap_or(NaiveDate::MIN)
}

/// The day, as the app shows it: "11 May 2026".
pub fn day_label(day: Date) -> String {
    naive_of(day).format("%-d %b %Y").to_string()
}

/// The first day of the week a day is in.
pub fn week_start(day: Date) -> Date {
    let offset = naive_of(day).weekday().num_days_from_monday() as i64;
    day.add_days(-offset)
}

/// The week a day is in, as the app writes it: "2026-W19".
pub fn week_key(day: Date) -> String {
    let iso = week_start(day);
    format!("{}-W{:02}", iso.year(), iso_week(iso))
}

/// The week number of a date, in the ISO's reckoning.
fn iso_week(day: Date) -> u32 {
    let naive = naive_of(day);
    naive.iso_week().week()
}

/// The vitals of one day of wellness.
pub fn vitals_of(row: &WellnessDaily) -> Vitals {
    let sleep = row.sleep_s.and_then(Duration::from_seconds).map(|total| {
        Sleep::with_stages(
            total,
            row.deep_sleep_s.and_then(Duration::from_seconds),
            row.rem_s.and_then(Duration::from_seconds),
            row.light_sleep_s.and_then(Duration::from_seconds),
            row.awake_s.and_then(Duration::from_seconds),
        )
    });
    Vitals {
        sleep,
        hrv: row.hrv_ms.and_then(Hrv::from_ms),
        resting_hr: row.resting_hr.and_then(RestingHr::from_bpm),
    }
}

/// The wellness days, as the core reads them.
pub fn wellness_days(rows: &[WellnessDaily]) -> Vec<WellnessDay> {
    rows.iter()
        .filter_map(|row| Date::parse(&row.date).map(|date| WellnessDay::new(date, vitals_of(row))))
        .collect()
}

/// The history of the last `days` nights before a day, as the core reads it.
pub fn history_before(api: &Api, day: Date, days: i64) -> ApiResult<Vec<WellnessDay>> {
    let store = api.store();
    let rows = store
        .wellness_before(&day.to_iso(), days)
        .map_err(|_| ApiError::internal("the history could not be read"))?;
    Ok(wellness_days(&rows))
}

/// The pace of a run, in seconds per kilometre.
///
/// `None` when the run has no distance: a pace with no distance is not a
/// pace.
pub fn pace_of(distance_m: f64, moving_time_s: f64) -> Option<f64> {
    if distance_m <= 0.0 {
        return None;
    }
    let pace = moving_time_s / (distance_m / 1000.0);
    if !pace.is_finite() || pace <= 0.0 {
        return None;
    }
    Some(units::round_pace_s_per_km(pace))
}

/// One run, as the feed shows it.
///
/// Every measurement in the row is the core's rendering of it, with the
/// number beside it for the chart. Nothing here formats a value.
pub fn activity_summary(activity: &StoredActivity) -> ActivitySummary {
    ActivitySummary {
        id: activity.id,
        name: activity.name.clone(),
        sport_type: activity.sport_type.clone(),
        start_time: crate::state::format_instant(activity.start_time),
        local_date: activity.local_date.clone(),
        distance: render::distance(Some(activity.distance_m)),
        moving_time: render::duration(Some(activity.moving_time_s)),
        elapsed_time: render::duration(Some(activity.elapsed_time_s)),
        avg_hr: render::heart_rate(activity.avg_hr),
        max_hr: render::heart_rate(activity.max_hr),
        avg_cadence: render::cadence(None),
        elevation_gain: render::altitude(Some(activity.elevation_gain_m)),
        pace: render::pace(pace_of(activity.distance_m, activity.moving_time_s)),
        story: None,
        geo_points: Vec::new(),
    }
}

/// The runs of one day.
pub fn runs_on(api: &Api, date: &str) -> ApiResult<Vec<ActivitySummary>> {
    let runs = api
        .store()
        .activities_for_date(date)
        .map_err(|_| ApiError::internal("the runs could not be read"))?;
    Ok(runs.iter().map(activity_summary).collect())
}

/// The distance of one day, in metres.
pub fn distance_on(api: &Api, date: &str) -> ApiResult<f64> {
    let runs = api
        .store()
        .activities_for_date(date)
        .map_err(|_| ApiError::internal("the runs could not be read"))?;
    Ok(runs.iter().map(|run| run.distance_m).sum())
}

/// The median of a sample, as the core reads it.
pub fn median(values: &[f64]) -> Option<f64> {
    band::median(values)
}

/// The distance comparison, when there is one.
///
/// **The core renders it.** The comparison is a statement — "2.4 km more
/// than the 10 km target" — and the core states it only when the gap from
/// the target is beyond 10% (the contract §5). Within 10% it is absent:
/// 10.2 km against a 10 km target is not news.
pub fn distance_comparison(actual_m: Option<f64>, target_m: Option<f64>) -> Option<String> {
    let actual = Distance::from_metres(actual_m?)?;
    let target = Distance::from_metres(target_m?)?;
    statement::distance_comparison(actual, target)
}

/// The summary of a planned session: "10 km · easy".
///
/// The distance is the core's own rendering of it, and the session type is
/// the plan's own vocabulary.
pub fn session_summary(target_distance_m: Option<f64>, session_type: &str) -> String {
    match target_distance_m.and_then(Distance::from_metres) {
        Some(distance) => format!("{} · {session_type}", distance.to_human()),
        None => session_type.to_string(),
    }
}

/// The sentence of a planned session.
///
/// The wording is the catalogue's: "Today — 10 km · easy" for the day
/// itself, "11 May 2026 — 8 km · easy" for another day. The catalogue's
/// templates take the summary as an argument, and the summary is the
/// distance the core's units render and the plan's own vocabulary.
pub fn session_sentence(
    day: Date,
    today: Date,
    target_distance_m: Option<f64>,
    session_type: &str,
) -> String {
    let summary = session_summary(target_distance_m, session_type);
    if day == today {
        return stride_core::strings::render("option.today_session", &[("summary", &summary)])
            .unwrap_or(summary);
    }
    let label = day_label(day);
    stride_core::strings::render(
        "option.other_session",
        &[("day", &label), ("summary", &summary)],
    )
    .unwrap_or(summary)
}

/// Whether a planned session was run.
///
/// The status is computed on read: a stored verdict would be a judgement
/// the plan forbids. A **baseline** never yields "not done" — an inferred
/// baseline is not a prescription, and the plan says so.
pub fn session_status(date: Date, today: Date, source: &str, ran: bool) -> &'static str {
    if ran {
        return "done";
    }
    if date < today && source != stride_store::model::plan_source::BASELINE {
        return "not_done";
    }
    "planned"
}

/// The runs of every week, oldest first.
pub fn weeks(runs: &[StoredActivity]) -> Vec<(Date, Vec<&StoredActivity>)> {
    let mut grouped: BTreeMap<String, Vec<&StoredActivity>> = BTreeMap::new();
    for run in runs {
        let date = match Date::parse(&run.local_date) {
            Some(date) => week_start(date),
            None => continue,
        };
        grouped.entry(date.to_iso()).or_default().push(run);
    }
    grouped
        .into_iter()
        .filter_map(|(key, runs)| Date::parse(&key).map(|date| (date, runs)))
        .collect()
}

/// The personal records of a set of runs.
///
/// The records are facts: the longest run, and the fastest 5 km and 10 km
/// the file itself carries. The values are rendered by the core's units,
/// so the app reads "16.1 km" and "20 m" rather than a number it rounds
/// itself.
pub fn personal_records(api: &Api, runs: &[StoredActivity]) -> ApiResult<Vec<PersonalRecord>> {
    let mut records: Vec<PersonalRecord> = Vec::new();

    if let Some(longest) = runs.iter().max_by(|left, right| {
        left.distance_m
            .partial_cmp(&right.distance_m)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        records.push(PersonalRecord {
            key: "progress.best.longest".to_string(),
            label: "Longest run".to_string(),
            value: render::distance(Some(longest.distance_m)),
            activity_id: longest.id,
            activity_date: longest.local_date.clone(),
        });
    }

    for (key, label, target_m) in [
        ("progress.best.5k", "Fastest 5 km", 5_000.0),
        ("progress.best.10k", "Fastest 10 km", 10_000.0),
    ] {
        let mut best: Option<(f64, &StoredActivity)> = None;
        for run in runs.iter() {
            if run.distance_m < target_m {
                continue;
            }
            let decoded = match series::decoded(api, run.id) {
                Some(decoded) => decoded,
                None => continue,
            };
            if let Some(seconds) = series::fastest_split(&decoded, target_m) {
                best = Some(match best {
                    Some((current, current_run)) if current <= seconds => (current, current_run),
                    _ => (seconds, run),
                });
            }
        }
        if let Some((seconds, run)) = best {
            records.push(PersonalRecord {
                key: key.to_string(),
                label: label.to_string(),
                value: render::duration(Some(seconds)),
                activity_id: run.id,
                activity_date: run.local_date.clone(),
            });
        }
    }

    Ok(records)
}
