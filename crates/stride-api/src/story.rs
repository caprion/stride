//! The seam to the core's story engine.
//!
//! **The story has one owner: `stride-core`.** This module reads the store,
//! hands the core the run and the athlete's history, and carries the
//! sentence back. It writes no sentence of its own — the story is the
//! core's, like every other word the app says.

use stride_core::date::Date;
use stride_core::story::{RunFacts, Split};
use stride_store::ActivitySummary as StoredActivity;

use crate::error::ApiResult;
use crate::state::Api;

/// The run, as the story engine reads it.
///
/// `splits` is empty where the caller has not decoded the file — the feed —
/// and the run's own laps where it has — the detail.
pub fn run_facts_of(activity: &StoredActivity, splits: Vec<Split>) -> Option<RunFacts> {
    let date = Date::parse(&activity.local_date)?;
    Some(RunFacts {
        date,
        distance_m: Some(activity.distance_m),
        moving_time_s: Some(activity.moving_time_s),
        avg_hr: activity.avg_hr,
        splits,
    })
}

/// The athlete's history: every run the story may compare against.
///
/// The story engine ignores runs on the run's own day and runs after it, so
/// the whole history is safe to pass.
pub fn history_facts(api: &Api) -> ApiResult<Vec<RunFacts>> {
    let today = crate::routes::today(api)?;
    let runs = api
        .store()
        .activities_between("0000-01-01", &today.to_iso())
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    Ok(runs
        .iter()
        .filter_map(|run| run_facts_of(run, Vec::new()))
        .collect())
}

/// The story of one run: one sentence, never empty.
pub fn story_of(run: &RunFacts, history: &[RunFacts]) -> String {
    stride_core::story::story(run, history)
}

/// The sentence for a run whose facts cannot be read.
///
/// The core's own wording — the api never writes a sentence of its own.
pub fn no_story() -> String {
    stride_core::strings::text("story.nothing_unusual")
        .unwrap_or("nothing unusual about this one")
        .to_string()
}

/// The splits of a decoded run, as the story engine reads them.
pub fn story_splits(activity: &stride_ingest::FitActivity) -> Vec<Split> {
    activity
        .laps
        .iter()
        .map(|lap| Split {
            distance_m: lap.distance_m,
            moving_time_s: lap.duration_s,
        })
        .collect()
}
