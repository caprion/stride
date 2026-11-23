//! The challenges: the athlete's own, measured week by week.
//!
//! A challenge is something the athlete set themselves — a run of at
//! least so many kilometres, or so many kilometres across the week —
//! and the grid says, week by week, whether the week reached it.
//!
//! **The weekly goal is a challenge already.** It is the same shape —
//! so many kilometres across the week — so it is in the grid without
//! anything being set up, and the athlete's own challenges sit beside
//! it.
//!
//! ## A week is met or it is not
//!
//! The answer is a fact: the week's distance was what it was, and the
//! longest run in it was what it was. **A week that did not reach the
//! target is not a failure** — it is a week, and the reader draws the
//! conclusion. Nothing here is coloured as a problem, because nothing
//! here is a problem.

use serde_json::Value;
use stride_core::challenges::{Challenge, ChallengeKind, WeekMeasure};

use crate::dto::{ChallengeInsight, ChallengeRow, ChallengeWeekRow, ChallengeWindow};
use crate::error::{ApiError, ApiResult};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// The challenges of the athlete, over a window of weeks.
///
/// The weeks are the calendar weeks the Progress window covers, oldest
/// first.
pub fn challenges(api: &Api, request: &Request) -> ApiResult<Value> {
    let current = super::today(api)?;
    let weeks = request.query.integer("weeks")?.unwrap_or(12);
    if !(1..=104).contains(&weeks) {
        return Err(ApiError::bad_request(
            "the challenge window is not between 1 and 104 weeks",
        ));
    }

    let runs = api
        .store()
        .activities_between("0001-01-01", &current.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;

    // The weeks, and what each of them measured.
    let window_start = super::week_start(current).add_days(-7 * (weeks - 1));
    let measures: Vec<WeekMeasure> = super::week_summaries(&runs)
        .into_iter()
        .filter(|week| week.week_start >= window_start)
        .map(|week| WeekMeasure {
            week_start: week.week_start,
            distance_m: week.distance_m,
            longest_run_m: week.longest.map(|longest| longest.distance_m),
        })
        .collect();

    let challenges = challenges_of(api)?;
    let grid = stride_core::challenges::challenge_grid(&challenges, &measures);

    let rows: Vec<ChallengeRow> = challenges
        .iter()
        .zip(grid.iter())
        .map(|(challenge, weeks_of)| ChallengeRow {
            key: format!("{}:{}", challenge.kind.as_str(), challenge.target_m),
            kind: challenge.kind.as_str().to_string(),
            label: challenge
                .label
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| challenge.name()),
            target: render::distance_of(challenge.target_m),
            streak: render::whole(Some(f64::from(stride_core::challenges::streak_of(
                weeks_of,
            )))),
            weeks: weeks_of
                .iter()
                .zip(measures.iter())
                .map(|(week, measure)| ChallengeWeekRow {
                    week_start: measure.week_start.to_iso(),
                    label: super::day_label(measure.week_start),
                    met: week.met,
                    measured: render::distance_of(week.measured_m),
                })
                .collect(),
        })
        .collect();

    let response = ChallengeInsight {
        generated_at: api.generated_at(),
        date: current.to_iso(),
        window: ChallengeWindow {
            from: window_start.to_iso(),
            from_label: super::day_label(window_start),
            to: current.to_iso(),
            to_label: super::day_label(current),
            weeks,
        },
        challenges: rows,
        unavailable: None,
    };
    ok(&response)
}

/// The challenges the athlete has set, with the weekly goal among them.
///
/// The goal is a challenge of the same shape, so it needs no setting up
/// and it is in the grid from the first day. The athlete's own
/// challenges come after it.
fn challenges_of(api: &Api) -> ApiResult<Vec<Challenge>> {
    let mut challenges: Vec<Challenge> = Vec::new();

    // The weekly goal, which is a challenge of the week-distance kind.
    let goal = api
        .store()
        .goal_km()
        .map_err(|error| api.store_error("the goal could not be read", &error))?;
    if let Some(goal) = goal {
        if goal > 0.0 {
            challenges.push(Challenge::new(ChallengeKind::WeekDistance, goal * 1000.0));
        }
    }

    // The athlete's own challenges, in the order they were written.
    for kind in [ChallengeKind::WeekDistance, ChallengeKind::RunDistance] {
        let notes = api
            .store()
            .notes_of_kind(note_kind_of(kind))
            .map_err(|error| api.store_error("the challenges could not be read", &error))?;
        for note in notes {
            let Some(value) = note.value else { continue };
            if value <= 0.0 {
                continue;
            }
            challenges.push(Challenge {
                kind,
                target_m: value * 1000.0,
                label: note.text.clone(),
            });
        }
    }

    Ok(challenges)
}

/// The note kind a challenge of this kind is written as.
fn note_kind_of(kind: ChallengeKind) -> &'static str {
    match kind {
        ChallengeKind::WeekDistance => stride_store::model::note_kind::CHALLENGE_WEEK,
        ChallengeKind::RunDistance => stride_store::model::note_kind::CHALLENGE_RUN,
    }
}
