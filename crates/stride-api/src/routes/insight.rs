//! The insight routes: Today, Progress, the chips of a run, and Compare.

use serde_json::Value;
use stride_core::statement;
use stride_core::units;

use crate::dto::{
    Adherence, Best, Chips, CompareCandidate, CompareMetric, CompareResponse, CompareRun,
    CompareSeries, Consistency, PaceAtHr, ProgressInsight, TodayInsight, WeekRunning, WeeklyVolume,
};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::{core_date, ok, personal_records, runs_on, session_sentence, week_start};

/// How far apart two runs may be and still be "the same distance".
const SAME_DISTANCE_TOLERANCE: f64 = 0.1;

/// How many weeks back the fitness comparison looks.
const WEEKS_APART: usize = 8;

/// How far apart two heart rates may be and still be the same heart rate.
const SAME_HEART_RATE_TOLERANCE: f64 = 3.0;

/// Today's readout.
pub fn today(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let sessions = api
        .store()
        .plan_sessions_for_day(&current.to_iso())
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    let source = plan_source_of(api, &sessions);

    let mut planned = None;
    let mut options = Vec::new();
    let week_end = week_start(current).add_days(6);
    let week_sessions = api
        .store()
        .plan_sessions_between(&week_start(current).to_iso(), &week_end.to_iso())
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    for session in week_sessions.iter() {
        if session.date == current.to_iso() {
            if planned.is_none() {
                planned = Some(render_session(api, session, &source, current)?);
            }
            continue;
        }
        // A rest day is not an alternative to run: the options are the
        // sessions the athlete could choose instead.
        if session.session_type == "rest" {
            continue;
        }
        options.push(render_session(api, session, &source, current)?);
    }

    let runs = api
        .store()
        .activities_between("0000-01-01", &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let last_run = runs.first().map(super::activity_summary);

    let week_runs = api
        .store()
        .activities_between(&week_start(current).to_iso(), &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let week = WeekRunning {
        distance: render::distance(Some(week_runs.iter().map(|run| run.distance_m).sum())),
        sessions: week_runs.len(),
    };

    let distance_comparison = match (&planned, &last_run) {
        (Some(session), Some(run)) => super::distance_comparison(
            run.distance.as_ref().and_then(|value| value.raw),
            session.target_distance.as_ref().and_then(|value| value.raw),
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
    let _ = request;
    let current = super::today(api)?;
    let runs = api
        .store()
        .activities_between("0000-01-01", &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;

    let mut weekly_volume: Vec<WeeklyVolume> = Vec::new();
    let mut pace_at_hr: Vec<PaceAtHr> = Vec::new();
    let mut consistency: Vec<Consistency> = Vec::new();
    for (week, week_runs) in super::weeks(&runs) {
        let distance_m: f64 = week_runs.iter().map(|run| run.distance_m).sum();
        weekly_volume.push(WeeklyVolume {
            week_start: week.to_iso(),
            distance: render::distance(Some(distance_m)),
        });
        let paces: Vec<f64> = week_runs
            .iter()
            .filter_map(|run| super::pace_of(run.distance_m, run.moving_time_s))
            .collect();
        let hearts: Vec<f64> = week_runs.iter().filter_map(|run| run.avg_hr).collect();
        pace_at_hr.push(PaceAtHr {
            week_start: week.to_iso(),
            pace: render::pace(super::median(&paces).map(units::round_pace_s_per_km)),
            hr: render::heart_rate(super::median(&hearts)),
        });
        consistency.push(Consistency {
            week_start: week.to_iso(),
            sessions: week_runs.len(),
        });
    }

    let typical_week = super::median(
        &weekly_volume
            .iter()
            .filter_map(|week| week.distance.as_ref().and_then(|value| value.raw))
            .collect::<Vec<f64>>(),
    );

    let records = personal_records(api, &runs)?;
    let bests: Vec<Best> = records
        .iter()
        .map(|record| Best {
            key: record.key.clone(),
            value: record.value.clone(),
            date: record.activity_date.clone(),
        })
        .collect();

    let adherence = api
        .store()
        .active_plan_block(stride_store::model::plan_source::PRESCRIBED)
        .ok()
        .flatten()
        .map(|block| {
            let sessions = api
                .store()
                .plan_sessions_for_block(block.id)
                .unwrap_or_default();
            let mut done = 0usize;
            for session in sessions.iter() {
                if !runs_on(api, &session.date).unwrap_or_default().is_empty() {
                    done += 1;
                }
            }
            Adherence {
                done,
                planned: sessions.len(),
            }
        });

    // The headline is the core's statement: it compares the pace at the
    // same heart rate now and then, and it says so plainly when there is
    // not enough history to compare.
    let headline = statement::progress_headline(pace_at_hr_comparison(&pace_at_hr));
    let headline_note = headline.evidence.clone();

    let response = ProgressInsight {
        generated_at: api.generated_at(),
        headline: headline.headline,
        headline_note,
        weekly_volume,
        typical_week: render::distance(typical_week),
        pace_at_hr,
        bests,
        consistency,
        adherence,
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
        ),
        metric(
            "time",
            "vital.time",
            render::duration(Some(first.moving_time_s)),
            render::duration(Some(second.moving_time_s)),
        ),
        metric(
            "pace",
            "vital.pace",
            render::pace(pace_a),
            render::pace(pace_b),
        ),
        metric(
            "hr",
            "vital.heartRate",
            render::heart_rate(first.avg_hr),
            render::heart_rate(second.avg_hr),
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
            distance: render::distance(Some(first.distance_m)),
            story: None,
        },
        b: CompareRun {
            id: second.id,
            label: second.name.clone(),
            date: second.local_date.clone(),
            distance: render::distance(Some(second.distance_m)),
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
            distance: render::distance(Some(run.distance_m)),
        })
        .collect();
    // The shape the app consumes is a list of candidates; the envelope
    // carries the time the answer was produced.
    serde_json::to_value(candidates)
        .map_err(|_| ApiError::internal("the answer could not be written"))
}

// ------------------------------------------------------------------ helpers

/// The pace at the same heart rate, now and then.
///
/// The core renders the Progress headline from this: a lower pace at the
/// same heart rate is a fitter athlete, and a higher one is not. The
/// comparison needs two weeks at the same heart rate, so it is made only
/// when the history has them.
fn pace_at_hr_comparison(weeks: &[PaceAtHr]) -> Option<statement::PaceAtHr> {
    let mut now_index = None;
    for (index, week) in weeks.iter().enumerate().rev() {
        let has_pace = week.pace.as_ref().and_then(|value| value.raw).is_some();
        let has_heart_rate = week.hr.as_ref().and_then(|value| value.raw).is_some();
        if has_pace && has_heart_rate {
            now_index = Some(index);
            break;
        }
    }
    let now_index = now_index?;
    let now = weeks.get(now_index)?;
    let now_hr = now.hr.as_ref().and_then(|value| value.raw)?;
    let now_pace = now.pace.as_ref().and_then(|value| value.raw)?;

    // Eight weeks back, if there were eight weeks and the heart rate was
    // the same then.
    let then_index = now_index.checked_sub(WEEKS_APART)?;
    let then = weeks.get(then_index)?;
    let then_hr = then.hr.as_ref().and_then(|value| value.raw)?;
    let then_pace = then.pace.as_ref().and_then(|value| value.raw)?;
    if (then_hr - now_hr).abs() > SAME_HEART_RATE_TOLERANCE {
        return None;
    }
    Some(statement::PaceAtHr {
        now_s_per_km: now_pace,
        then_s_per_km: then_pace,
        weeks: (now_index - then_index) as u32,
    })
}

/// One row of the diff.
///
/// The values are the core's rendering of them, with the numbers beside
/// them. The **difference** is the core's to render as well, and the core
/// does not render a difference yet — so the row carries no difference
/// until it does.
fn metric(
    key: &str,
    label_key: &str,
    a: Option<crate::dto::RenderedValue>,
    b: Option<crate::dto::RenderedValue>,
) -> CompareMetric {
    let delta = match (a.as_ref(), b.as_ref()) {
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
        delta: render::difference(delta),
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

/// The source the plan's sessions came from.
fn plan_source_of(api: &Api, sessions: &[stride_store::PlanSession]) -> String {
    if sessions.is_empty() {
        return stride_store::model::plan_source::TYPED.to_string();
    }
    let block_id = sessions[0].block_id;
    match api.store().plan_blocks() {
        Ok(blocks) => blocks
            .iter()
            .find(|block| block.id == block_id)
            .map(|block| block.source.clone())
            .unwrap_or_else(|| stride_store::model::plan_source::TYPED.to_string()),
        Err(_) => stride_store::model::plan_source::TYPED.to_string(),
    }
}

/// One session, as it travels.
fn render_session(
    api: &Api,
    session: &stride_store::PlanSession,
    source: &str,
    current: stride_core::date::Date,
) -> ApiResult<crate::dto::PlanSession> {
    let date = core_date(&session.date)?;
    let runs = runs_on(api, &session.date)?;
    let actual_distance_m = if runs.is_empty() {
        None
    } else {
        Some(
            runs.iter()
                .filter_map(|run| run.distance.as_ref().and_then(|value| value.raw))
                .sum(),
        )
    };
    let status = super::session_status(date, current, source, !runs.is_empty());
    Ok(crate::dto::PlanSession {
        date: session.date.clone(),
        session_type: session.session_type.clone(),
        title: session.title.clone(),
        sentence: session_sentence(
            date,
            current,
            session.target_distance_m,
            &session.session_type,
        ),
        target_distance: render::distance(session.target_distance_m),
        actual_distance: render::distance(actual_distance_m),
        status: status.to_string(),
        source: source.to_string(),
    })
}
