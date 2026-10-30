//! The insight routes: today's call, each run's story, and progress.
//!
//! These routes have no Python reference. They read the same rows every
//! other route reads and hand them to `stride_core::insights`, which does all
//! the deciding. Nothing here leaves the device.

use serde_json::{json, Map, Value};

use stride_core::insights::{self, Recovery, RecoveryRead, Run};

use crate::analytics;
use crate::backend::ActivityFilter;
use crate::error::ApiError;
use crate::query::Query;
use crate::routes::Ctx;

/// Every run, oldest first, as the insights read them.
fn load_runs(ctx: &Ctx<'_>) -> Result<Vec<Run>, ApiError> {
    let demo_visible = ctx.backend.public_status()?.demo_visible;
    let rows = ctx.backend.activities(&ActivityFilter {
        demo_visible,
        all_sports: true,
        ..ActivityFilter::default()
    })?;
    let mut runs: Vec<Run> = analytics::runs(&rows, demo_visible)
        .into_iter()
        .filter(|row| row.distance_m > 0.0)
        .map(|row| Run {
            id: row.id,
            name: row.name.clone(),
            date: row
                .local_date
                .get(..10)
                .unwrap_or(&row.local_date)
                .to_string(),
            distance_m: row.distance_m,
            moving_time_s: row.moving_time_s,
            avg_hr: row.avg_hr.filter(|hr| *hr > 0.0),
        })
        .collect();
    runs.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));
    Ok(runs)
}

/// Sleep, HRV and resting HR for the 31 days ending on `day`.
fn load_recovery(ctx: &Ctx<'_>, day: &str) -> Vec<Recovery> {
    let Ok(date) = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d") else {
        return Vec::new();
    };
    let from = (date - chrono::Duration::days(31))
        .format("%Y-%m-%d")
        .to_string();
    ctx.backend
        .physiology(&from, day)
        .unwrap_or_default()
        .into_iter()
        .map(|row| Recovery {
            date: row.date.get(..10).unwrap_or(&row.date).to_string(),
            sleep_s: row.sleep_s,
            hrv_ms: row.hrv_ms,
            resting_hr: row.resting_hr,
        })
        .collect()
}

fn to_value<T: serde::Serialize>(value: &T) -> Result<Value, ApiError> {
    serde_json::to_value(value)
        .map_err(|error| ApiError::internal(format!("cannot shape the insight: {error}")))
}

/// `GET /api/v1/insights/today` — one call, one reason, one guardrail.
pub fn today(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let day = ctx.today.format("%Y-%m-%d").to_string();
    let runs = load_runs(ctx)?;
    let recovery = load_recovery(ctx, &day);
    let read = RecoveryRead::from_history(&recovery, &day);
    to_value(&insights::today(&day, &runs, read.as_ref()))
}

/// `GET /api/v1/insights/activity/{id}` — what one run meant.
pub fn activity(ctx: &Ctx<'_>, id: i64) -> Result<Value, ApiError> {
    let runs = load_runs(ctx)?;
    let Some(run) = runs.iter().find(|run| run.id == id) else {
        return Err(ApiError::not_found("Run not found."));
    };
    let recovery = load_recovery(ctx, &run.date);
    let read = RecoveryRead::from_history(&recovery, &run.date);
    let story = insights::run_story(run, &runs, read.as_ref());
    let mut value = to_value(&story)?;
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "recovery".into(),
            match &read {
                Some(read) => json!({
                    "flags": read.flags(),
                    "normal": read.normal_count().0,
                    "read": read.normal_count().1,
                }),
                None => Value::Null,
            },
        );
    }
    Ok(value)
}

/// `GET /api/v1/insights/chips?days=365` — the chip of every run in the
/// window, keyed by activity id, for the run list.
pub fn chips(ctx: &Ctx<'_>, query: &Query) -> Result<Value, ApiError> {
    let days = query.int("days", 365, Some(1), Some(3650))?;
    let runs = load_runs(ctx)?;
    let from = (ctx.today - chrono::Duration::days(days))
        .format("%Y-%m-%d")
        .to_string();
    let mut out = Map::new();
    for run in runs.iter().filter(|run| run.date >= from) {
        let story = insights::run_story(run, &runs, None);
        out.insert(
            run.id.to_string(),
            json!({ "chip": story.chip, "tone": story.tone }),
        );
    }
    Ok(json!({ "items": out }))
}

/// `GET /api/v1/insights/progress` — fitness, consistency, bests,
/// milestones and race ranges.
pub fn progress(ctx: &Ctx<'_>) -> Result<Value, ApiError> {
    let day = ctx.today.format("%Y-%m-%d").to_string();
    let runs = load_runs(ctx)?;
    to_value(&insights::progress(&runs, &day))
}
