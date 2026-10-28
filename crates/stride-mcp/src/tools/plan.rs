//! `plan_context` — the plan, and what actually happened to it.
//!
//! The tool reports the plan as it is and the execution as it was: which
//! sessions were hit, which were missed, and — where the engine could say —
//! why. A cause without its confidence is not shown: the engine gives a
//! cause only when it has the evidence for it, and the confidence says how
//! sure it is.
//!
//! The day's recommendation is returned as a suggestion with the evidence
//! behind it. It is never a command: nothing here changes the plan.

use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::args::Args;
use crate::error::McpError;
use crate::output::Envelope;
use crate::schema;
use crate::tools::common::{self};
use crate::tools::Context;

/// The tool's name.
pub const NAME: &str = "plan_context";

/// What the tool does.
pub const DESCRIPTION: &str = "The training plan and how it was kept: the current block, the \
week that contains a date, the sessions of a block, or the results of a range. Each session \
carries the run that fulfilled it, the verdict, and the cause of a miss with the engine's \
confidence in it. The recommendation for the day is returned as a suggestion with the evidence \
behind it, never as an instruction.";

/// The views.
const VIEWS: &[&str] = &["current", "week", "block", "results"];

/// The largest number of sessions one week may show.
const MAX_SESSIONS: usize = 30;

/// The arguments.
pub fn schema() -> Value {
    schema::object(
        json!({
            "view": schema::enumeration(
                VIEWS,
                "Which part of the plan to answer with: \"current\", \"week\", \"block\" or \
\"results\". Defaults to \"current\".",
            ),
            "date": schema::date(
                "The day the week is about, for the \"week\" view. Defaults to today.",
            ),
            "block_id": schema::integer("The block to show, for the \"block\" view.", None, None),
            "from": schema::date(
                "The first day of the range, for the \"results\" view. Defaults to the start of \
the current block.",
            ),
            "to": schema::date(
                "The last day of the range, for the \"results\" view. Defaults to today.",
            ),
        }),
        &[],
    )
}

/// Run the tool.
pub fn run(api: &dyn StrideApi, ctx: &Context, args: &Value) -> Result<Value, McpError> {
    let args = Args::new(NAME, args)?;
    args.reject_unknown(&["view", "date", "block_id", "from", "to"])?;
    let view = args
        .enum_str("view", VIEWS)?
        .unwrap_or_else(|| "current".to_string());
    let date = args.date("date")?.unwrap_or(ctx.today);
    let block_id = args.opt_i64("block_id")?;
    let from = args.date("from")?;
    let to = args.date("to")?;

    let mut limitations: Vec<String> = Vec::new();
    let mut data = json!({ "view": view.clone(), "has_plan": false });

    match view.as_str() {
        "current" => {
            let current = common::get(api, NAME, "/api/v1/plan/current")?;
            // The route sends the block's detail, which holds the block
            // itself one level down.
            let block = current
                .get("block")
                .map(block_of)
                .unwrap_or(Value::Null);
            let week = current.get("week").cloned().unwrap_or(Value::Null);
            let has_plan = !block.is_null();
            let today = common::get(api, NAME, "/api/v1/plan/today")?;
            data["has_plan"] = json!(has_plan);
            data["block"] = block_value(&block);
            data["week"] = week_value(&week, &mut limitations);
            data["today"] = today_value(&today);
            if !has_plan {
                limitations.push("there is no plan in the database".to_string());
            }
        }
        "week" => {
            let path = format!(
                "/api/v1/plan/week?{}",
                common::query(&[("date", Some(common::date_text(date)))])
            );
            let week = common::get(api, NAME, &path)?;
            let has_week = week
                .get("week")
                .map(|value| !value.is_null())
                .unwrap_or(false);
            data["has_plan"] = json!(has_week);
            data["block"] = Value::Null;
            data["week"] = week_value(&week, &mut limitations);
            data["phase"] = phase_value(week.get("phase"));
            data["today"] = Value::Null;
        }
        "block" => {
            let Some(block_id) = block_id else {
                return Err(McpError::invalid(
                    NAME,
                    "block_id",
                    "is required for the \"block\" view",
                ));
            };
            let block = common::get(api, NAME, &format!("/api/v1/plan/blocks/{block_id}"))?;
            data["has_plan"] = json!(true);
            data["block"] = block_value(block.get("block").unwrap_or(&Value::Null));
            data["phases"] = phases_value(
                block.get("phases").and_then(Value::as_array),
                &mut limitations,
            );
            data["week"] = Value::Null;
            data["today"] = Value::Null;
        }
        "results" => {
            let path = format!(
                "/api/v1/plan/results?{}",
                common::query(&[
                    ("from", from.map(common::date_text)),
                    ("to", to.map(common::date_text)),
                ])
            );
            let results = common::get(api, NAME, &path)?;
            let list = common::items(&results);
            let mut hits = 0usize;
            let mut misses = 0usize;
            let mut rendered: Vec<Value> = Vec::new();
            for item in list.iter().take(crate::output::MAX_ITEMS) {
                let verdict = common::str_of(item, "verdict").unwrap_or_default();
                if verdict == "hit" || verdict == "missed_pace" {
                    hits += 1;
                } else {
                    misses += 1;
                }
                rendered.push(json!({
                    "plan_session_id": common::i64_of(item, "plan_session_id"),
                    "activity_id": common::i64_of(item, "activity_id"),
                    "verdict": verdict,
                    "cause": common::str_of(item, "cause"),
                    "cause_confidence": common::f64_of(item, "cause_confidence"),
                    "distance_delta_m": common::f64_of(item, "distance_delta_m"),
                    "pace_delta_s_per_km": common::f64_of(item, "pace_delta_s_km"),
                    "adherence_score": common::f64_of(item, "adherence_score"),
                }));
            }
            if list.len() > crate::output::MAX_ITEMS {
                limitations.push(format!(
                    "the range holds {} results and only the first {} are returned.",
                    list.len(),
                    crate::output::MAX_ITEMS
                ));
            }
            data["has_plan"] = json!(!list.is_empty());
            data["results"] = Value::Array(rendered);
            data["hits"] = json!(hits);
            data["misses"] = json!(misses);
            data["week"] = Value::Null;
            data["today"] = Value::Null;
        }
        other => {
            return Err(McpError::invalid(
                NAME,
                "view",
                format!("must be one of {VIEWS:?}, not {other:?}"),
            ))
        }
    }

    // The adherence of the range is the same number the plan page shows.
    if view == "current" || view == "week" {
        let path = format!(
            "/api/v1/plan/adherence?{}",
            common::query(&[
                ("from", from.map(common::date_text)),
                ("to", to.map(common::date_text)),
            ])
        );
        if let Ok(adherence) = common::get(api, NAME, &path) {
            data["adherence"] = json!({
                "overall_pct": common::f64_of(&adherence, "overall_pct"),
                "planned_km": common::f64_of(&adherence, "planned_km"),
                "actual_km": common::f64_of(&adherence, "actual_km"),
                "planned_sessions": common::i64_of(&adherence, "planned_sessions"),
                "completed_sessions": common::i64_of(&adherence, "completed_sessions"),
                "hit_sessions": common::i64_of(&adherence, "hit_sessions"),
                "missed_sessions": common::i64_of(&adherence, "missed_sessions"),
                "extra_sessions": common::i64_of(&adherence, "extra_sessions"),
                "by_verdict": adherence.get("by_verdict").cloned().unwrap_or(Value::Null),
            });
        }
    }

    let mut envelope = Envelope::new(data)
        .coverage("view", view)
        .coverage("date", common::date_text(date))
        .coverage("attributed_to", "the plan tables of the athlete's own database");
    for limitation in limitations {
        envelope = envelope.limitation(limitation);
    }
    Ok(envelope.finish(NAME))
}

/// The block inside a block's detail.
fn block_of(detail: &Value) -> Value {
    detail.get("block").cloned().unwrap_or_else(|| detail.clone())
}

/// The block, without the notes.
fn block_value(block: &Value) -> Value {
    if block.is_null() {
        return Value::Null;
    }
    json!({
        "id": common::i64_of(block, "id"),
        "name": common::str_of(block, "name"),
        "start_date": common::str_of(block, "start_date"),
        "end_date": common::str_of(block, "end_date"),
        "goal": common::str_of(block, "goal"),
        "status": common::str_of(block, "status"),
        "phase_count": common::i64_of(block, "phase_count"),
        "week_count": common::i64_of(block, "week_count"),
        "session_count": common::i64_of(block, "session_count"),
    })
}

/// The phase, without the notes.
fn phase_value(phase: Option<&Value>) -> Value {
    let Some(phase) = phase else {
        return Value::Null;
    };
    if phase.is_null() {
        return Value::Null;
    }
    json!({
        "id": common::i64_of(phase, "id"),
        "name": common::str_of(phase, "name"),
        "start_date": common::str_of(phase, "start_date"),
        "end_date": common::str_of(phase, "end_date"),
        "target_weekly_km_min": common::f64_of(phase, "target_weekly_km_min"),
        "target_weekly_km_max": common::f64_of(phase, "target_weekly_km_max"),
        "target_sessions": common::i64_of(phase, "target_sessions"),
    })
}

/// The week, with its sessions.
///
/// The router sends the week itself one level down (`{week, phase,
/// sessions, planned_km, ...}`), so the week payload and the sessions are
/// read from their own places.
fn week_value(week: &Value, limitations: &mut Vec<String>) -> Value {
    if week.is_null() {
        return Value::Null;
    }
    let base = week.get("week").unwrap_or(week);
    let sessions = week
        .get("sessions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if sessions.len() > MAX_SESSIONS {
        limitations.push(format!(
            "the week holds {} sessions and only the first {MAX_SESSIONS} are shown.",
            sessions.len()
        ));
    }
    json!({
        "id": common::i64_of(base, "id"),
        "week_number": common::i64_of(base, "week_number"),
        "start_date": common::str_of(base, "start_date"),
        "end_date": common::str_of(base, "end_date"),
        "target_km": common::f64_of(base, "target_km"),
        "planned_km": common::f64_of(week, "planned_km"),
        "actual_km": common::f64_of(week, "actual_km"),
        "planned_count": common::i64_of(week, "planned_count"),
        "completed_count": common::i64_of(week, "completed_count"),
        "adherence_pct": common::f64_of(week, "adherence_pct"),
        "sessions": sessions
            .iter()
            .take(MAX_SESSIONS)
            .map(session_value)
            .collect::<Vec<Value>>(),
    })
}

/// One planned session and how it went.
fn session_value(session: &Value) -> Value {
    let result = session.get("result").cloned().unwrap_or(Value::Null);
    json!({
        "id": common::i64_of(session, "id"),
        "date": common::str_of(session, "date"),
        "session_type": common::str_of(session, "session_type"),
        "title": common::str_of(session, "title"),
        "target": {
            "distance_m": common::f64_of(session, "target_distance_m"),
            "pace_low_s_per_km": common::f64_of(session, "pace_low_s_km"),
            "pace_high_s_per_km": common::f64_of(session, "pace_high_s_km"),
            "hr_zone_target": common::i64_of(session, "hr_zone_target"),
        },
        "activity": activity_value(session.get("activity")),
        "verdict": common::str_of(&result, "verdict"),
        "cause": common::str_of(&result, "cause"),
        "cause_confidence": common::f64_of(&result, "cause_confidence"),
        "adherence_score": common::f64_of(&result, "adherence_score"),
        "distance_delta_m": common::f64_of(&result, "distance_delta_m"),
        "pace_delta_s_per_km": common::f64_of(&result, "pace_delta_s_km"),
    })
}

/// The run that fulfilled a session.
fn activity_value(activity: Option<&Value>) -> Value {
    let Some(activity) = activity else {
        return Value::Null;
    };
    if activity.is_null() {
        return Value::Null;
    }
    json!({
        "id": common::i64_of(activity, "id"),
        "name": common::str_of(activity, "name"),
        "local_date": common::str_of(activity, "local_date"),
        "distance_m": common::f64_of(activity, "distance_m"),
        "pace_s_per_km": common::f64_of(activity, "pace_s_per_km"),
        "avg_hr": common::f64_of(activity, "avg_hr"),
        "elevation_gain_m": common::f64_of(activity, "elevation_gain_m"),
    })
}

/// The phases of a block, with their weeks.
fn phases_value(phases: Option<&Vec<Value>>, limitations: &mut Vec<String>) -> Value {
    let Some(phases) = phases else {
        return Value::Null;
    };
    let mut rendered: Vec<Value> = Vec::new();
    for phase in phases.iter().take(10) {
        let weeks = phase
            .get("weeks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if weeks.len() > 20 {
            limitations.push(format!(
                "a phase holds {} weeks and only the first 20 are shown.",
                weeks.len()
            ));
        }
        let mut value = phase_value(Some(phase));
        value["weeks"] = Value::Array(
            weeks
                .iter()
                .take(20)
                .map(|week| week_value(week, limitations))
                .collect(),
        );
        rendered.push(value);
    }
    Value::Array(rendered)
}

/// Today's plan and the suggestion that goes with it.
fn today_value(today: &Value) -> Value {
    let session = today.get("session").cloned().unwrap_or(Value::Null);
    let readiness = today.get("readiness").cloned().unwrap_or(Value::Null);
    json!({
        "date": common::str_of(today, "date"),
        "has_session": common::bool_of(today, "has_session").unwrap_or(false),
        "session": if session.is_null() {
            Value::Null
        } else {
            session_value(&session)
        },
        "suggestion": common::str_of(today, "recommendation"),
        "evidence": common::str_of(today, "rationale"),
        "readiness": if readiness.is_null() {
            Value::Null
        } else {
            json!({
                "date": common::str_of(&readiness, "date"),
                "score": common::f64_of(&readiness, "score"),
                "grade": common::str_of(&readiness, "grade"),
            })
        },
    })
}
