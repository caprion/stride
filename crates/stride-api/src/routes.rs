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
pub mod athlete;
pub mod challenges;
pub mod data;
pub mod export;
pub mod film;
pub mod heatmap;
pub mod insight;
pub mod load;
pub mod pace;
pub mod paces;
pub mod pacing_curve;
pub mod plan;
pub mod rolling;
pub mod series;
pub mod settings;
pub mod stimulus;
pub mod threshold;
pub mod year;

pub mod update;

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::Serialize;
use serde_json::Value;
use stride_core::band;
use stride_core::date::Date;
use stride_core::types::{Duration, Hrv, RestingHr, Sleep, Vitals, WellnessDay};
use stride_core::units;
use stride_store::{ActivitySummary as StoredActivity, WellnessDaily};

use crate::dto::{
    ActivitySummary, EquivalentRow, InsightWindow, PaceRow, PersonalRecord, RaceRow, RenderedValue,
    SourceLine,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

/// The most runs one answer may carry.
pub const MAX_RUNS_IN_ANSWER: usize = 500;

/// How many weeks a window-shaped answer may cover — the Progress route's
/// own bound.
pub const MAX_WINDOW_WEEKS: i64 = 104;

/// The source lines of an answer.
///
/// **The source line is hoisted**: the answer carries **one** line for the
/// screen to name — the zone model's own line when the bands were measured
/// against one — and every line the answer was built from travels behind a
/// click.
pub fn source_lines(
    facts: &crate::athlete::Facts,
    ladder: &stride_core::training_paces::Ladder,
) -> (Option<String>, Vec<SourceLine>) {
    let (model_line, model_day) = match facts.model.as_ref() {
        Some(model) => {
            let read_on = day_label(model.read_on);
            (
                stride_core::model_source(&model.training_method, model.max_heart_rate, &read_on),
                Some(read_on),
            )
        }
        None => (None, None),
    };
    let mut sources: Vec<SourceLine> = Vec::new();
    if let Some(race) = ladder.race.as_ref() {
        let date = day_label(race.date);
        sources.push(SourceLine {
            kind: "race".to_string(),
            line: ladder.source_line(&date),
            date: Some(race.date.to_iso()),
            date_label: Some(date),
        });
    }
    if let Some(line) = model_line.as_ref() {
        sources.push(SourceLine {
            kind: "zone_model".to_string(),
            line: Some(line.clone()),
            date: facts.model.as_ref().map(|model| model.read_on.to_iso()),
            date_label: model_day.clone(),
        });
    }
    let source = model_line
        .clone()
        .or_else(|| sources.first().and_then(|line| line.line.clone()));
    (source, sources)
}

/// A pace, or the reason there is none.
///
/// **A missing pace is `null` with a reason**, never a band of zero: the
/// method fixes no pace for some rungs, and an input the athlete has not
/// given makes the others no values.
pub fn pace_row(value: &stride_core::training_paces::PaceValue) -> PaceRow {
    use stride_core::training_paces::PaceValue;
    match value {
        PaceValue::Known(band) => pace_band_row(band),
        // The method states no pace for this rung: that is the method's
        // own answer, and the rule's line says so.
        PaceValue::NoneStated => PaceRow {
            value: None,
            fast_s_per_km: None,
            slow_s_per_km: None,
            missing: core_line("pace.none_stated"),
        },
        PaceValue::Missing(missing) => PaceRow {
            value: None,
            fast_s_per_km: None,
            slow_s_per_km: None,
            missing: missing.reason(),
        },
    }
}

/// A pace band, as the answer carries it.
pub fn pace_band_row(band: &stride_core::training_paces::PaceBand) -> PaceRow {
    PaceRow {
        value: band.render(),
        fast_s_per_km: Some(band.fast_s_per_km),
        slow_s_per_km: Some(band.slow_s_per_km),
        missing: None,
    }
}

/// The equivalent race times, and the two ways each was reached.
///
/// The two models are kept side by side and never merged: the gap between
/// them is a fact about two models, and the surface may print it.
pub fn equivalents_of(ladder: &stride_core::training_paces::Ladder) -> Vec<EquivalentRow> {
    ladder
        .equivalents
        .iter()
        .map(|equivalent| EquivalentRow {
            distance: stride_core::training_paces::race_label(equivalent.distance_m).to_string(),
            time: equivalent
                .riegel_time_s
                .map(stride_core::units::format_race_time),
            vdot_time: equivalent
                .vdot_time_s
                .map(stride_core::units::format_race_time),
            pace: equivalent
                .riegel_pace_s_per_km()
                .and_then(|pace| stride_core::units::render_pace(Some(pace))),
            difference_s: equivalent.difference_s.map(units::round_time_s),
        })
        .collect()
}

/// A threshold's value, as the core renders it: "165 bpm", "4:16–4:17 /km",
/// or both.
pub fn threshold_value(
    heart_rate_bpm: Option<f64>,
    pace: Option<&stride_core::training_paces::PaceBand>,
) -> Option<String> {
    let heart_rate = heart_rate_bpm.and_then(|bpm| units::render_heart_rate(Some(bpm)));
    let pace = pace.and_then(|band| band.render());
    match (heart_rate, pace) {
        (Some(heart_rate), Some(pace)) => Some(format!("{heart_rate} \u{b7} {pace}")),
        (Some(heart_rate), None) => Some(heart_rate),
        (None, Some(pace)) => Some(pace),
        (None, None) => None,
    }
}

/// A sentence from the core's catalogue.
///
/// The api never writes a sentence of its own: it asks the core's
/// catalogue for it and passes it on.
pub fn core_line(id: &str) -> Option<String> {
    stride_core::strings::render(id, &[])
}

/// A race the athlete typed, as the answer carries it.
///
/// A race is an **observation**, not a verdict: the athlete types it, and
/// the app never invents one and never infers one from the training. The
/// row carries the value, the day it was entered, the day it was last
/// changed and its source — the same discipline the wellness observations
/// have.
pub fn race_row(race: &stride_store::RaceObservation) -> RaceRow {
    let date = core_date(&race.race_date).ok();
    let pace = match race.distance_m > 0.0 && race.time_s > 0.0 {
        true => Some(race.time_s / (race.distance_m / 1_000.0)),
        false => None,
    };
    RaceRow {
        id: race.id,
        label: race.label.clone(),
        distance: render::distance_of(race.distance_m),
        time: RenderedValue {
            text: units::format_race_time(race.time_s),
            raw: Some(race.time_s),
        },
        pace: render::pace(pace),
        date: race.race_date.clone(),
        date_label: date.map(day_label).unwrap_or_default(),
        name: race.name.clone(),
        source: race.source.clone(),
        entered_on: race.entered_on.clone(),
        updated_on: race.updated_on.clone(),
        anchor: race.anchor,
    }
}

/// The Progress window: the last `weeks` calendar weeks through today.
///
/// **One window for the whole wall.** The bound is the Progress route's
/// own — one to a hundred and four weeks, twelve by default — and the
/// window is the calendar weeks, the current one in it and partial. Every
/// window-shaped answer names the window it covers, so that a number is
/// never read apart from the period it describes.
pub fn insight_window(api: &Api, request: &Request) -> ApiResult<(InsightWindow, Date, Date)> {
    let weeks = request.query.integer("weeks")?.unwrap_or(12);
    if !(1..=MAX_WINDOW_WEEKS).contains(&weeks) {
        return Err(ApiError::bad_request(
            "the window is not between 1 and 104 weeks",
        ));
    }
    let today = today(api)?;
    let from = week_start(today).add_days(-7 * (weeks - 1));
    Ok((
        InsightWindow {
            from: from.to_iso(),
            from_label: day_label(from),
            to: today.to_iso(),
            to_label: day_label(today),
            weeks,
        },
        from,
        today,
    ))
}

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
    day_of(api, api.now())
}

/// The athlete's day of an instant.
///
/// The same rule as `today`, applied to a moment that has passed: the
/// instant's own day, moved by the offset the host says the athlete's days
/// are counted in. A model read at 23:30 UTC belongs to the athlete's day,
/// not to the UTC one.
pub fn day_of(api: &Api, instant: DateTime<Utc>) -> ApiResult<Date> {
    let offset = chrono::Duration::minutes(api.utc_offset_minutes() as i64);
    let naive = (instant + offset).naive_utc();
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

/// The month a day is in, as the app shows it: "May 2026".
pub fn month_label(day: Date) -> String {
    naive_of(day).format("%B %Y").to_string()
}

/// The first day of the week a day is in.
pub fn week_start(day: Date) -> Date {
    let offset = naive_of(day).weekday().num_days_from_monday() as i64;
    day.add_days(-offset)
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
/// number beside it for the chart. The story is the core's own sentence
/// about the run.
pub fn activity_summary(
    activity: &StoredActivity,
    group: crate::dto::ActivityGroupTotals,
    story: String,
) -> ActivitySummary {
    ActivitySummary {
        id: activity.id,
        name: activity.name.clone(),
        sport_type: activity.sport_type.clone(),
        start_time: crate::state::format_instant(activity.start_time),
        local_date: activity.local_date.clone(),
        distance: render::distance_of(activity.distance_m),
        duration: render::duration_of(activity.moving_time_s),
        heart_rate: render::heart_rate(activity.avg_hr),
        elevation_gain: render::elevation(Some(activity.elevation_gain_m)),
        pace: render::pace(pace_of(activity.distance_m, activity.moving_time_s)),
        group,
        story,
        geo_points: Vec::new(),
    }
}

/// The group a run belongs to, and that group's totals.
///
/// The feed is grouped by the API — the API is the one that knows the
/// totals — and the app labels the groups. A group is a week: the key is
/// the Monday the week starts on.
pub fn group_totals(api: &Api, key: &str) -> ApiResult<crate::dto::ActivityGroupTotals> {
    let start = core_date(key)?;
    let end = start.add_days(6);
    let runs = api
        .store()
        .activities_between(key, &end.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let distance_m: f64 = render::total_of(runs.iter().map(|run| run.distance_m));
    let moving_time_s: f64 = render::total_of(runs.iter().map(|run| run.moving_time_s));
    Ok(crate::dto::ActivityGroupTotals {
        key: key.to_string(),
        distance: render::distance_of(distance_m),
        duration: render::duration_of(moving_time_s),
    })
}

/// The runs as the feed shows them, each with the group it belongs to.
pub fn summaries(api: &Api, runs: &[StoredActivity]) -> ApiResult<Vec<ActivitySummary>> {
    let history = crate::story::history_facts(api)?;
    let mut groups: BTreeMap<String, crate::dto::ActivityGroupTotals> = BTreeMap::new();
    let mut out: Vec<ActivitySummary> = Vec::with_capacity(runs.len());
    for run in runs {
        let key = group_key(run)?;
        let group = match groups.get(&key) {
            Some(group) => group.clone(),
            None => {
                let group = group_totals(api, &key)?;
                groups.insert(key.clone(), group.clone());
                group
            }
        };
        let story = match crate::story::run_facts_of(run, Vec::new()) {
            Some(facts) => crate::story::story_of(&facts, &history),
            None => crate::story::no_story(),
        };
        out.push(activity_summary(run, group, story));
    }
    Ok(out)
}

/// The group a run belongs to: the week it falls in.
pub fn group_key(activity: &StoredActivity) -> ApiResult<String> {
    let date = core_date(&activity.local_date)?;
    Ok(week_start(date).to_iso())
}

/// The distance of one day, in metres.
pub fn distance_on(api: &Api, date: &str) -> ApiResult<f64> {
    let runs = api
        .store()
        .activities_for_date(date)
        .map_err(|_| ApiError::internal("the runs could not be read"))?;
    Ok(render::total_of(runs.iter().map(|run| run.distance_m)))
}

/// The median of a sample, as the core reads it.
pub fn median(values: &[f64]) -> Option<f64> {
    band::median(values)
}

/// The sentence of a planned session.
///
/// The wording is the catalogue's: "Today — 10 km · easy" for the day
/// itself, "11 May 2026 — 8 km · easy" for another day. The summary is the
/// core's own label of the session, and the template is the core's
/// catalogue: there is no wording of this crate's own here.
pub fn session_sentence(day: Date, today: Date, summary: &str) -> String {
    if day == today {
        return stride_core::strings::render("option.today_session", &[("summary", summary)])
            .unwrap_or_else(|| summary.to_string());
    }
    let label = day_label(day);
    stride_core::strings::render(
        "option.other_session",
        &[("day", &label), ("summary", summary)],
    )
    .unwrap_or_else(|| summary.to_string())
}

/// The runs of every week, oldest first.
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

/// One week of the runs, as the app's weekly series reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeekSummary {
    /// The Monday the week starts on.
    pub week_start: Date,
    /// The week's distance, in metres.
    pub distance_m: f64,
    /// How many runs the week holds.
    pub sessions: usize,
    /// The week's median pace, in seconds per kilometre.
    ///
    /// The median of the **runs' own paces**: it is the pace of a run in
    /// that week, not a pace the athlete held at one heart rate.
    pub median_pace_s_per_km: Option<f64>,
    /// The week's median heart rate.
    ///
    /// A separate median from the pace, and never a matched effort with
    /// it: the two are descriptive.
    pub median_hr_bpm: Option<f64>,
    /// The week's longest run.
    pub longest: Option<WeekLongest>,
}

/// The longest run of a week.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeekLongest {
    /// The run's distance, in metres.
    pub distance_m: f64,
    /// The run's pace, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
}

/// The weeks of the runs, oldest first.
///
/// **One implementation of the week's numbers.** The Progress screen's
/// weekly volume, its weekly medians and the pace page's training series
/// all read these, so they cannot disagree about a week.
pub fn week_summaries(runs: &[StoredActivity]) -> Vec<WeekSummary> {
    weeks(runs)
        .into_iter()
        .map(|(week_start, week_runs)| {
            let distance_m = render::total_of(week_runs.iter().map(|run| run.distance_m));
            let paces: Vec<f64> = week_runs
                .iter()
                .filter_map(|run| pace_of(run.distance_m, run.moving_time_s))
                .collect();
            let hearts: Vec<f64> = week_runs.iter().filter_map(|run| run.avg_hr).collect();
            WeekSummary {
                week_start,
                distance_m,
                sessions: week_runs.len(),
                median_pace_s_per_km: median(&paces),
                median_hr_bpm: median(&hearts),
                longest: longest_of(&week_runs),
            }
        })
        .collect()
}

/// The longest run of a week: the first of the runs that are as long as the
/// longest, so the answer does not depend on the order the runs came in.
fn longest_of(runs: &[&StoredActivity]) -> Option<WeekLongest> {
    let mut longest: Option<&StoredActivity> = None;
    for run in runs {
        let is_longer = match longest {
            Some(current) => run.distance_m > current.distance_m,
            None => true,
        };
        if is_longer {
            longest = Some(run);
        }
    }
    let run = longest?;
    Some(WeekLongest {
        distance_m: run.distance_m,
        pace_s_per_km: pace_of(run.distance_m, run.moving_time_s),
    })
}
