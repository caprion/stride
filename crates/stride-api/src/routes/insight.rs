//! The insight routes: Today, Progress, the chips of a run, and Compare.

use std::collections::BTreeMap;

use serde_json::Value;
use stride_core::date::Date;
use stride_core::eddington::{DailyDistance, EddingtonUnit, Period};
use stride_core::statement;
use stride_core::units;
use stride_core::ValueUnit;
use stride_store::ActivitySummary as StoredActivity;

use crate::dto::{
    Best, Chips, CompareCandidate, CompareMetric, CompareResponse, CompareRun, CompareSeries,
    Consistency, PaceAtHr, ProgressEddington, ProgressInsight, RunningDay, TodayInsight,
    WeekRunning, WeeklyVolume,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::{core_date, ok, personal_records};

/// How far apart two runs may be and still be "the same distance".
const SAME_DISTANCE_TOLERANCE: f64 = 0.1;

/// Today's readout.
///
/// The plan's own session for the day, and the plan's other sessions —
/// unranked, and never linked to the vitals. The plan comes from the
/// core's engine, so the block in force is the only block that is read.
pub fn today(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let active = crate::plan::active_or_empty(api)?;
    let facts = crate::plan::run_facts(api, "0000-01-01", &current.to_iso())?;
    let week_start = super::week_start(current);
    let week_end = week_start.add_days(6);

    let mut planned = None;
    let mut options = Vec::new();
    for result in stride_core::plan::results_of_at(&active.block, &facts, current).iter() {
        if active.is_baseline() {
            break;
        }
        if result.session.date < week_start || result.session.date > week_end {
            continue;
        }
        let rendered = crate::plan::session_of(&active, result, current);
        if result.session.date == current {
            if planned.is_none() {
                planned = Some(rendered);
            }
            continue;
        }
        // A rest day is not an alternative to run: the options are the
        // sessions the athlete could choose instead.
        if result.session.kind == stride_core::plan::SessionKind::Rest {
            continue;
        }
        options.push(rendered);
    }

    let runs = api
        .store()
        .activities_between("0000-01-01", &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let mut last_run = super::summaries(api, &runs[..runs.len().min(1)])?
        .into_iter()
        .next();
    if let Some(run) = &mut last_run {
        if let Some(decoded) = super::series::decoded(api, run.id) {
            run.geo_points = super::series::geo_points(&decoded, 64);
        }
    }

    let week_runs = api
        .store()
        .activities_between(&super::week_start(current).to_iso(), &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let week = WeekRunning {
        distance: render::distance_of(render::total_of(week_runs.iter().map(|run| run.distance_m))),
        sessions: week_runs.len(),
        days: (0..7)
            .map(|offset| {
                let date = week_start.add_days(offset).to_iso();
                RunningDay {
                    sessions: week_runs
                        .iter()
                        .filter(|run| run.local_date == date)
                        .count(),
                    distance: render::distance_of(render::total_of(
                        week_runs
                            .iter()
                            .filter(|run| run.local_date == date)
                            .map(|run| run.distance_m),
                    )),
                    date,
                }
            })
            .collect(),
    };

    let distance_comparison = match &planned {
        Some(session) => stride_core::plan::distance_comparison(
            session.actual.as_ref().and_then(|value| value.raw),
            session.target.as_ref().and_then(|value| value.raw),
        ),
        _ => None,
    };

    let response = TodayInsight {
        generated_at: api.generated_at(),
        date: current.to_iso(),
        planned,
        options,
        last_run,
        distance_comparison,
        week,
    };
    ok(&response)
}

/// The Progress screen's data.
pub fn progress(api: &Api, request: &Request) -> ApiResult<Value> {
    let weeks = request.query.integer("weeks")?.unwrap_or(12);
    if !(1..=104).contains(&weeks) {
        return Err(ApiError::bad_request(
            "the progress window is not between 1 and 104 weeks",
        ));
    }
    let current = super::today(api)?;
    let runs = api
        .store()
        .activities_between("0000-01-01", &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;

    let mut weekly_volume: Vec<WeeklyVolume> = Vec::new();
    let mut pace_at_hr: Vec<PaceAtHr> = Vec::new();
    let mut consistency: Vec<Consistency> = Vec::new();
    // One implementation of the week's numbers: the shared weekly summary,
    // the same one the pace page's training series reads.
    for week in super::week_summaries(&runs) {
        let week_start = week.week_start.to_iso();
        weekly_volume.push(WeeklyVolume {
            week_start: week_start.clone(),
            distance: render::distance_of(week.distance_m),
        });
        pace_at_hr.push(PaceAtHr {
            week_start: week_start.clone(),
            pace: render::pace(week.median_pace_s_per_km.map(units::round_pace_s_per_km)),
            heart_rate: render::heart_rate(week.median_hr_bpm),
        });
        consistency.push(Consistency {
            week_start,
            sessions: week.sessions,
        });
    }

    let typical_week = super::median(
        &weekly_volume
            .iter()
            .filter_map(|week| week.distance.raw)
            .collect::<Vec<f64>>(),
    );

    let records = personal_records(api, &runs)?;
    let bests: Vec<Best> = records
        .iter()
        .map(|record| Best {
            key: record.key.clone(),
            value: record
                .value
                .clone()
                .map(|value| value.text)
                .unwrap_or_default(),
            date: record.activity_date.clone(),
        })
        .collect();

    // The plan's adherence: the count of the block in force, and the
    // count is the engine's.
    let adherence = match crate::plan::active_or_empty(api) {
        Ok(active) if !active.is_baseline() => {
            let facts =
                crate::plan::run_facts(api, "0000-01-01", &current.to_iso()).unwrap_or_default();
            Some(crate::plan::adherence_of(stride_core::plan::adherence_of(
                &active.block,
                &facts,
            )))
        }
        _ => None,
    };

    // Independent weekly medians are descriptive, not matched-effort evidence.
    let headline = if weekly_volume.is_empty() {
        statement::progress_headline(None).headline
    } else {
        stride_core::strings::text("status.saved_running_history")
            .unwrap_or_default()
            .to_string()
    };

    // The sessions a week, as the core renders them.
    let sessions_per_week = if consistency.is_empty() {
        None
    } else {
        let total: usize = consistency.iter().map(|week| week.sessions).sum();
        let average = total as f64 / consistency.len() as f64;
        render::average(Some(average))
    };

    let window_start = super::week_start(current).add_days(-7 * (weeks - 1));
    let from = window_start.to_iso();
    let period_runs: Vec<_> = runs.iter().filter(|run| run.local_date >= from).collect();
    let period = crate::dto::ProgressPeriod {
        from: from.clone(),
        to: current.to_iso(),
        distance: render::distance_of(render::total_of(
            period_runs.iter().map(|run| run.distance_m),
        )),
        duration: render::duration_of(render::total_of(
            period_runs.iter().map(|run| run.moving_time_s),
        )),
        sessions: period_runs.len(),
        recorded_weeks: weekly_volume
            .iter()
            .filter(|week| week.week_start >= from)
            .count(),
    };
    let response = ProgressInsight {
        generated_at: api.generated_at(),
        date: current.to_iso(),
        period,
        headline,
        headline_note: None,
        weekly_volume,
        typical_week: render::distance(typical_week),
        pace_at_hr,
        bests,
        consistency,
        sessions_per_week,
        adherence,
        weeks_needed: None,
        // The number covers the same window the totals do: the card and
        // the chart cannot disagree about the period they describe.
        eddington: eddington_of(&period_runs, window_start, current),
    };
    ok(&response)
}

/// The chips of a run.
///
/// The run is named by the query — the app calls this route with the run's
/// id — and the path may name it too.
pub fn chips(api: &Api, request: &Request) -> ApiResult<Value> {
    let id = match request.query.integer("id")? {
        Some(id) => id,
        None => request.param_id("id")?,
    };
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
    let chips: Vec<crate::dto::Chip> = vec![
        crate::dto::Chip {
            key: "distance".to_string(),
            label: "Distance".to_string(),
            value: render::distance(Some(run.distance_m)),
        },
        crate::dto::Chip {
            key: "time".to_string(),
            label: "Time".to_string(),
            value: render::duration(Some(run.moving_time_s)),
        },
        crate::dto::Chip {
            key: "pace".to_string(),
            label: "Pace".to_string(),
            value: render::pace(super::pace_of(run.distance_m, run.moving_time_s)),
        },
        crate::dto::Chip {
            key: "heart_rate".to_string(),
            label: "Heart rate".to_string(),
            value: render::heart_rate(run.avg_hr),
        },
    ];
    let response = Chips {
        activity_id: id,
        chips,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The metric diff between two runs.
pub fn compare(api: &Api, request: &Request) -> ApiResult<Value> {
    let a = request.param_id("a").or_else(|_| {
        request
            .query
            .integer("a")?
            .ok_or_else(|| ApiError::bad_request("the comparison has no first run"))
    })?;
    let b = request.param_id("b").or_else(|_| {
        request
            .query
            .integer("b")?
            .ok_or_else(|| ApiError::bad_request("the comparison has no second run"))
    })?;
    let first = read_run(api, a)?;
    let second = read_run(api, b)?;

    let days = match (core_date(&first.local_date), core_date(&second.local_date)) {
        (Ok(first), Ok(second)) => Some(second.days_since(first).abs()),
        _ => None,
    };
    let same_distance = if first.distance_m > 0.0 {
        (second.distance_m - first.distance_m).abs() <= first.distance_m * SAME_DISTANCE_TOLERANCE
    } else {
        false
    };
    // The basis is the core's statement: what the two runs are being
    // compared on, in the core's words.
    let basis_text = statement::compare_basis(same_distance, false, days.unwrap_or(0)).sentence();

    let pace_a = super::pace_of(first.distance_m, first.moving_time_s);
    let pace_b = super::pace_of(second.distance_m, second.moving_time_s);
    let metrics = vec![
        metric(
            "distance",
            "vital.distance",
            render::distance(Some(first.distance_m)),
            render::distance(Some(second.distance_m)),
            ValueUnit::Distance,
        ),
        metric(
            "time",
            "vital.time",
            render::duration(Some(first.moving_time_s)),
            render::duration(Some(second.moving_time_s)),
            ValueUnit::Duration,
        ),
        metric(
            "pace",
            "vital.pace",
            render::pace(pace_a),
            render::pace(pace_b),
            ValueUnit::Pace,
        ),
        metric(
            "hr",
            "vital.heartRate",
            render::heart_rate(first.avg_hr),
            render::heart_rate(second.avg_hr),
            ValueUnit::HeartRate,
        ),
    ];

    let series = CompareSeries {
        a: super::series::samples_for(api, a),
        b: super::series::samples_for(api, b),
    };

    let response = CompareResponse {
        basis_text,
        metrics,
        a: CompareRun {
            id: first.id,
            label: first.name.clone(),
            date: first.local_date.clone(),
            distance: render::distance_of(first.distance_m),
            story: None,
        },
        b: CompareRun {
            id: second.id,
            label: second.name.clone(),
            date: second.local_date.clone(),
            distance: render::distance_of(second.distance_m),
            story: None,
        },
        series,
    };
    ok(&response)
}

/// The runs that can be compared against.
pub fn compare_candidates(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let runs = api
        .store()
        .activities_between("0000-01-01", &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let candidates: Vec<CompareCandidate> = runs
        .iter()
        .take(super::MAX_RUNS_IN_ANSWER)
        .map(|run| CompareCandidate {
            id: run.id,
            label: run.name.clone(),
            date: run.local_date.clone(),
            distance: render::distance_of(run.distance_m),
        })
        .collect();
    // The shape the app consumes is a list of candidates; the envelope
    // carries the time the answer was produced.
    serde_json::to_value(candidates)
        .map_err(|_| ApiError::internal("the answer could not be written"))
}

/// The Eddington number over the progress window.
///
/// The window is the one the screen is already showing — the calendar weeks
/// the volume chart draws — so the card and the chart describe the same
/// period. The days are the daily totals of the runs inside it, and the
/// unit is the contract's: kilometres.
///
/// A window with no distance has **no number**: the answer carries no
/// value at all, which is not the same as a number of zero.
fn eddington_of(runs: &[&StoredActivity], from: Date, to: Date) -> Option<ProgressEddington> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for run in runs {
        *totals.entry(run.local_date.clone()).or_insert(0.0) += run.distance_m;
    }
    let days: Vec<DailyDistance> = totals
        .iter()
        .filter_map(|(date, distance_m)| {
            // A day with no distance is absent from the list: absence is
            // not zero, and a day of nothing is not a day of running.
            if *distance_m <= 0.0 {
                return None;
            }
            let date = Date::parse(date)?;
            Some(DailyDistance::new(date, *distance_m))
        })
        .collect();
    let number =
        stride_core::eddington::eddington(&days, Period::new(from, to), EddingtonUnit::Kilometres);
    // No distance in the window, no number: the value is absent, not zero.
    let longest_day = number.longest_day_m?;
    Some(ProgressEddington {
        number: number.number,
        days_needed_for_next: number.days_needed_for_next,
        longest_day: render::distance(Some(longest_day)),
    })
}

// ------------------------------------------------------------------ helpers

/// One row of the diff.
///
/// The values are the core's rendering of them, with the numbers beside
/// them, and the **difference** is the core's rendering too: the sign is
/// the number's own, so the core's renderer reads a difference the way it
/// reads any other value ("-0.2 km", "-1 m", "-3 bpm").
///
fn metric(
    key: &str,
    label_key: &str,
    a: Option<crate::dto::RenderedValue>,
    b: Option<crate::dto::RenderedValue>,
    unit: ValueUnit,
) -> CompareMetric {
    let difference = match (a.as_ref(), b.as_ref()) {
        (Some(a), Some(b)) => match (a.raw, b.raw) {
            (Some(a), Some(b)) => Some(b - a),
            _ => None,
        },
        _ => None,
    };
    CompareMetric {
        key: key.to_string(),
        label_key: label_key.to_string(),
        a,
        b,
        delta: render::difference(difference, unit),
    }
}

/// One run, read.
fn read_run(api: &Api, id: i64) -> ApiResult<stride_store::Activity> {
    let run = api
        .store()
        .activity(id)
        .map_err(|error| api.store_error("the run could not be read", &error))?;
    match run {
        Some(run) => Ok(run),
        None => Err(ApiError::not_found(format!(
            "there is no run with the id {id}"
        ))),
    }
}
