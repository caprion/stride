//! `training_digest` — the one call that answers "how is my training going?".
//!
//! The digest is the first call of a session: it brings the load, the change
//! against the period before it, the plan and how it was kept, the readiness
//! of the period, the trend of the mechanics and the records into one
//! answer.
//!
//! Every section is a section of the answer or it is null. A section that
//! the database cannot fill is null with a limitation, never a zero — a
//! zero distance is a claim about the athlete's training, and only the
//! athlete's own runs can make that claim.

use chrono::{Datelike, NaiveDate};
use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::Envelope;
use crate::schema;
use crate::tools::common::{self, items};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "training_digest";

/// What the tool does.
pub const DESCRIPTION: &str = "An overview of a period: the distance, the number of runs and the \
climb; the change against the period before it; the plan and how much of it was kept, with the \
missed sessions and the engine's reasons; the readiness of the period; the trend of the \
mechanics; and the personal records. A section that the database cannot fill is null.";

/// The periods.
const PERIODS: &[&str] = &["7d", "28d", "block", "custom"];

/// The largest number of weeks the digest reports.
const MAX_WEEKS: usize = 16;

/// The largest number of missed sessions the digest reports.
const MAX_MISSES: usize = 20;

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "period": schema::enumeration(
                PERIODS,
                "The period to report: \"7d\", \"28d\", \"block\" for the current plan block, or \
\"custom\". Defaults to \"28d\".",
            ),
            "from": schema::date("The first day of a custom period, inclusive."),
            "to": schema::date("The last day of a custom period, inclusive."),
        }),
        &[],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["period", "from", "to"])?;
    let period = args
        .enum_str("period", PERIODS)?
        .unwrap_or_else(|| "28d".to_string());
    let custom_from = args.date("from")?;
    let custom_to = args.date("to")?;

    let mut limitations: Vec<String> = Vec::new();

    let (from, to) = match period.as_str() {
        "7d" => (
            ctx.today - chrono::Duration::days(6),
            ctx.today,
        ),
        "28d" => (
            ctx.today - chrono::Duration::days(27),
            ctx.today,
        ),
        "block" => {
            let current = common::get(api, NAME, "/api/v1/plan/current")?;
            let block = current.get("block").cloned().unwrap_or(Value::Null);
            let start = common::str_of(&block, "start_date")
                .and_then(|text| chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").ok());
            let end = common::str_of(&block, "end_date")
                .and_then(|text| chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").ok());
            match (start, end) {
                (Some(start), end) => {
                    if end.is_none() {
                        limitations.push(
                            "the current block has no end date, so the period ends today"
                                .to_string(),
                        );
                    }
                    (start, end.unwrap_or(ctx.today))
                }
                _ => {
                    limitations.push(
                        "there is no plan block in the database, so the period is the last \
twenty-eight days"
                            .to_string(),
                    );
                    (
                        ctx.today - chrono::Duration::days(27),
                        ctx.today,
                    )
                }
            }
        }
        _ => {
            let (Some(from), Some(to)) = (custom_from, custom_to) else {
                return Err(McpError::invalid(
                    NAME,
                    "from",
                    "a custom period needs both from and to",
                ));
            };
            (from, to)
        }
    };
    if from > to {
        return Err(McpError::invalid(NAME, "from", "must not be after to"));
    }

    let length = (to - from).num_days() + 1;
    let previous_to = from - chrono::Duration::days(1);
    let previous_from = previous_to - chrono::Duration::days(length - 1);

    // ------------------------------------------------------------------ load
    let load = load_of(api, from, to)?;
    let previous = load_of(api, previous_from, previous_to)?;
    if !load.complete || !previous.complete {
        limitations.push(
            "the period holds more runs than one page of the activity list; the load is measured \
from the weekly aggregation, which is rounded to the week"
                .to_string(),
        );
    }

    // ----------------------------------------------------------- the weeks
    let weeks = weeks_of(api, from, to)?;
    let weekly_series: Vec<Value> = weeks
        .iter()
        .rev()
        .take(MAX_WEEKS)
        .rev()
        .map(|week| {
            json!({
                "week": common::str_of(week, "week_key"),
                "start_date": common::str_of(week, "start_date"),
                "runs": common::i64_of(week, "activity_count"),
                "distance_m": common::f64_of(week, "distance_m"),
                "elevation_gain_m": common::f64_of(week, "elevation_gain_m"),
                "avg_pace_s_per_km": common::f64_of(week, "avg_pace_s_per_km"),
            })
        })
        .collect();

    let change = json!({
        "distance_m": change(load.distance_m, previous.distance_m),
        "distance_pct": percent(load.distance_m, previous.distance_m),
        "runs": change(load.runs as f64, previous.runs as f64),
        "elevation_gain_m": change(load.elevation_m, previous.elevation_m),
        "elevation_pct": percent(load.elevation_m, previous.elevation_m),
        "pace_s_per_km": match (load.pace_s_per_km, previous.pace_s_per_km) {
            (Some(now), Some(before)) => Some(common::round_to(now - before, 2)),
            _ => None,
        },
    });

    // --------------------------------------------------------------- the plan
    let plan = plan_of(api, from, to, &mut limitations)?;

    // ---------------------------------------------------------- the physiology
    let physiology = match physiology_of(api, from, to) {
        Ok(value) => value,
        Err(_) => {
            limitations.push(
                "the readiness of the period could not be read; the section is null".to_string(),
            );
            Value::Null
        }
    };

    // -------------------------------------------------------- the mechanics
    let mut trends = serde_json::Map::new();
    for metric in ["decoupling_pct", "cadence_cv", "form_score"] {
        let path = format!(
            "/api/v1/fitness/trends?{}",
            common::query(&[
                ("from", Some(common::date_text(from))),
                ("to", Some(common::date_text(to))),
                ("metric", Some(metric.to_string())),
            ])
        );
        if let Ok(value) = common::get(api, NAME, &path) {
            trends.insert(
                metric.to_string(),
                json!({
                    "baseline": common::f64_of(&value, "baseline"),
                    "trend_pct": common::f64_of(&value, "trend_pct"),
                    "points": value
                        .get("points")
                        .and_then(Value::as_array)
                        .map(|points| points.len())
                        .unwrap_or(0),
                }),
            );
        }
    }

    // ------------------------------------------------------------ the records
    let records = match common::get(api, NAME, "/api/v1/stats/personal-records") {
        Ok(value) => items(&value)
            .iter()
            .take(5)
            .map(|record| {
                json!({
                    "label": common::str_of(record, "label"),
                    "value": common::f64_of(record, "value"),
                    "unit": common::str_of(record, "unit"),
                    "activity_id": common::i64_of(record, "activity_id"),
                    "activity_date": common::str_of(record, "activity_date"),
                })
            })
            .collect::<Vec<Value>>(),
        Err(_) => Vec::new(),
    };

    let data = json!({
        "period": {
            "name": period,
            "from": common::date_text(from),
            "to": common::date_text(to),
            "days": length,
        },
        "load": {
            "distance_m": load.distance_m,
            "runs": load.runs,
            "elevation_gain_m": load.elevation_m,
            "moving_time_s": load.moving_time_s,
            "pace_s_per_km": load.pace_s_per_km,
            "weekly_series": weekly_series,
        },
        "change_vs_previous": change,
        "plan": plan,
        "physiology": physiology,
        "fingerprint_trends": Value::Object(trends),
        "records": records,
    });

    let mut envelope = Envelope::new(data)
        .coverage("from", common::date_text(from))
        .coverage("to", common::date_text(to))
        .coverage("previous_from", common::date_text(previous_from))
        .coverage("previous_to", common::date_text(previous_to))
        .coverage("runs", load.runs)
        .coverage("weeks", weeks.len())
        .coverage("attributed_to", "the athlete's own activities, plan and physiology tables");
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}

/// The load of a period.
#[derive(Debug, Clone, Copy, Default)]
struct Load {
    distance_m: f64,
    runs: i64,
    elevation_m: f64,
    moving_time_s: f64,
    pace_s_per_km: Option<f64>,
    /// Whether the totals came from every run, or from the weekly
    /// aggregation of a period that holds more runs than one page.
    complete: bool,
}

/// The load of a period, from the runs themselves.
///
/// A period with more runs than one page holds is measured from the weekly
/// aggregation instead, and the answer says so — a total that silently
/// covered only the first two hundred runs would be a wrong number.
fn load_of(api: &dyn StrideApi, from: NaiveDate, to: NaiveDate) -> Result<Load, McpError> {
    let path = format!(
        "/api/v1/activities?{}",
        common::query(&[
            ("from", Some(common::date_text(from))),
            ("to", Some(common::date_text(to))),
            ("per_page", Some("200".to_string())),
        ])
    );
    let value = common::get(api, NAME, &path)?;
    let total = common::i64_of(&value, "total").unwrap_or(0);
    let runs = items(&value);
    if (total as usize) <= runs.len() {
        let distance: f64 = runs
            .iter()
            .map(|item| common::f64_of(item, "distance_m").unwrap_or(0.0))
            .sum();
        let elevation: f64 = runs
            .iter()
            .map(|item| common::f64_of(item, "elevation_gain_m").unwrap_or(0.0))
            .sum();
        let moving: f64 = runs
            .iter()
            .map(|item| common::f64_of(item, "moving_time_s").unwrap_or(0.0))
            .sum();
        return Ok(Load {
            distance_m: common::round_to(distance, 1),
            runs: total,
            elevation_m: common::round_to(elevation, 1),
            moving_time_s: common::round_to(moving, 1),
            pace_s_per_km: if distance > 0.0 {
                Some(common::round_to(moving / (distance / 1000.0), 2))
            } else {
                None
            },
            complete: true,
        });
    }
    let weeks = weeks_of(api, from, to)?;
    let distance: f64 = weeks
        .iter()
        .map(|week| common::f64_of(week, "distance_m").unwrap_or(0.0))
        .sum();
    let elevation: f64 = weeks
        .iter()
        .map(|week| common::f64_of(week, "elevation_gain_m").unwrap_or(0.0))
        .sum();
    let moving: f64 = weeks
        .iter()
        .map(|week| common::f64_of(week, "moving_time_s").unwrap_or(0.0))
        .sum();
    let runs: i64 = weeks
        .iter()
        .map(|week| common::i64_of(week, "activity_count").unwrap_or(0))
        .sum();
    Ok(Load {
        distance_m: common::round_to(distance, 1),
        runs,
        elevation_m: common::round_to(elevation, 1),
        moving_time_s: common::round_to(moving, 1),
        pace_s_per_km: if distance > 0.0 {
            Some(common::round_to(moving / (distance / 1000.0), 2))
        } else {
            None
        },
        complete: false,
    })
}

/// The weeks that fall in a range.
fn weeks_of(api: &dyn StrideApi, from: NaiveDate, to: NaiveDate) -> Result<Vec<Value>, McpError> {
    let weeks = ((to - from).num_days() / 7 + 2).clamp(1, 200);
    let path = format!("/api/v1/stats/weekly?weeks={weeks}");
    let all = items(&common::get(api, NAME, &path)?);
    Ok(all
        .into_iter()
        .filter(|week| {
            common::str_of(week, "start_date")
                .and_then(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
                .is_some_and(|date| date >= from && date <= to)
        })
        .collect())
}

/// The plan of the period, and how it was kept.
fn plan_of(
    api: &dyn StrideApi,
    from: NaiveDate,
    to: NaiveDate,
    limitations: &mut Vec<String>,
) -> Result<Value, McpError> {
    let path = format!(
        "/api/v1/plan/adherence?{}",
        common::query(&[
            ("from", Some(common::date_text(from))),
            ("to", Some(common::date_text(to))),
        ])
    );
    let adherence = match common::get(api, NAME, &path) {
        Ok(value) => value,
        Err(_) => {
            limitations.push("the plan could not be read for this period".to_string());
            return Ok(Value::Null);
        }
    };
    let planned_sessions = common::i64_of(&adherence, "planned_sessions").unwrap_or(0);
    if planned_sessions == 0 {
        limitations.push("no session was planned in this period".to_string());
    }

    // The missed sessions, week by week.
    let mut misses: Vec<Value> = Vec::new();
    let mut monday = from - chrono::Duration::days(from.weekday().num_days_from_monday() as i64);
    let mut weeks = 0;
    while monday <= to && weeks < 8 {
        let path = format!(
            "/api/v1/plan/week?{}",
            common::query(&[("date", Some(common::date_text(monday)))])
        );
        if let Ok(week) = common::get(api, NAME, &path) {
            for session in week
                .get("sessions")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
            {
                let result = session.get("result").cloned().unwrap_or(Value::Null);
                let verdict = common::str_of(&result, "verdict").unwrap_or_default();
                if verdict == "hit" || verdict == "missed_pace" || verdict.is_empty() {
                    continue;
                }
                misses.push(json!({
                    "session_id": common::i64_of(&session, "id"),
                    "date": common::str_of(&session, "date"),
                    "session_type": common::str_of(&session, "session_type"),
                    "title": common::str_of(&session, "title"),
                    "verdict": verdict,
                    "cause": common::str_of(&result, "cause"),
                    "cause_confidence": common::f64_of(&result, "cause_confidence"),
                }));
            }
        }
        monday += chrono::Duration::days(7);
        weeks += 1;
    }
    if weeks == 8 && monday <= to {
        limitations.push(
            "the period is longer than eight weeks; only the first eight are searched for missed \
sessions"
                .to_string(),
        );
    }
    let missed = misses.len();
    let misses: Vec<Value> = misses.into_iter().take(MAX_MISSES).collect();

    Ok(json!({
        "planned_km": common::f64_of(&adherence, "planned_km"),
        "actual_km": common::f64_of(&adherence, "actual_km"),
        "adherence_pct": common::f64_of(&adherence, "overall_pct"),
        "planned_sessions": planned_sessions,
        "completed_sessions": common::i64_of(&adherence, "completed_sessions"),
        "missed_sessions": common::i64_of(&adherence, "missed_sessions"),
        "misses": misses,
        "misses_found": missed,
    }))
}

/// The readiness of the period.
fn physiology_of(api: &dyn StrideApi, from: NaiveDate, to: NaiveDate) -> Result<Value, McpError> {
    let path = format!(
        "/api/v1/physiology/readiness?{}",
        common::query(&[
            ("from", Some(common::date_text(from))),
            ("to", Some(common::date_text(to))),
        ])
    );
    let days = items(&common::get(api, NAME, &path)?);
    if days.is_empty() {
        return Ok(Value::Null);
    }
    let scores: Vec<f64> = days
        .iter()
        .filter_map(|day| common::f64_of(day, "score"))
        .collect();
    let latest = days.last();
    Ok(json!({
        "days_with_readiness": scores.len(),
        "mean_readiness": common::mean(&scores).map(|value| common::round_to(value, 1)),
        "latest": latest.map(|day| json!({
            "date": common::str_of(day, "date"),
            "score": common::f64_of(day, "score"),
            "grade": common::str_of(day, "grade"),
            "rationale": common::str_of(day, "rationale"),
        })),
    }))
}

/// The difference between two numbers.
fn change(now: f64, before: f64) -> f64 {
    common::round_to(now - before, 1)
}

/// The difference between two numbers, in percent.
fn percent(now: f64, before: f64) -> Option<f64> {
    if before == 0.0 {
        None
    } else {
        Some(common::round_to((now - before) / before.abs() * 100.0, 1))
    }
}
