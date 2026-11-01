//! The plan routes: the plan the athlete has, and what it says.
//!
//! The plan is **plain fact**: a session is *done*, *not done* or
//! *planned*. Nothing here grades the athlete, and nothing here says that
//! a session should have happened — the word "missed" does not exist in
//! this crate.
//!
//! ## The engine is the core's
//!
//! Every answer here is the core's: the baseline, the session of the day,
//! the sessions of the week, the count and the results. This crate reads
//! the store, hands the core the domain types it works on, and carries the
//! core's answers back. The status of a session is **computed on read**,
//! from the session's date, the runs of that day and the block's source —
//! a stored verdict would be a judgement the plan forbids, and a baseline
//! never yields "not done": an inferred baseline is not a prescription.
//!
//! ## The sources
//!
//! Three: a plan the athlete **imported** (`prescribed`), a plan the
//! athlete **typed** in (`typed`), and the **baseline** the app inferred
//! from the athlete's own history (`baseline`). The source is always named
//! in every answer, so the app never shows a plan without saying where it
//! came from.

use serde_json::Value;
use stride_core::plan::PlanBlock as CoreBlock;
use stride_store::model::{plan_source, plan_status};
use stride_store::{NewPlanBlock, NewPlanSession};

use crate::dto::{
    AdherenceWeek, CurrentPlan, PlanAdherence, PlanBlock, PlanBlocks, PlanImport, PlanResult,
    PlanResults, PlanSession, PlanSource, PlanSources, PlanToday, PlanWeek,
};
use crate::error::{ApiError, ApiResult};
use crate::plan;
use crate::planfile::{self, FileSession};
use crate::render;
use crate::router::Request;
use crate::state::Api;

use super::{core_date, ok, week_start};

/// The block in force.
///
/// Its source, its name, its date range, and **whether it is a baseline** —
/// which changes what the app may say about it: a baseline is a source,
/// not a prescription.
pub fn current(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let active = plan::active(api)?;
    let response = CurrentPlan {
        block: match &active {
            Some(active) => Some(block_of(api, active)?),
            None => None,
        },
        source: active
            .as_ref()
            .map(|active| active.source_name().to_string())
            .unwrap_or_else(|| plan_source::BASELINE.to_string()),
        is_baseline: active
            .as_ref()
            .map(|active| active.is_baseline())
            .unwrap_or(false),
        baseline_weeks: active.as_ref().and_then(|active| active.baseline_weeks),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's day.
///
/// Today's session, with the core's label ("10 km · easy") and the core's
/// wording ("Today — 10 km · easy").
pub fn today(api: &Api, _request: &Request) -> ApiResult<Value> {
    let current = super::today(api)?;
    let active = plan::active_or_empty(api)?;
    let facts = plan::run_facts(api, "0000-01-01", &current.to_iso())?;
    let planned = match stride_core::plan::today_of(&active.block, current) {
        Some(one) => {
            let actual = plan::distance_on(api, &current.to_iso())?;
            Some(plan::session_of(
                one,
                current,
                active.source,
                current,
                &facts,
                actual,
                actual.is_some(),
            ))
        }
        None => None,
    };
    let response = PlanToday {
        date: current.to_iso(),
        session: planned,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's week.
///
/// The seven days: each with its date, its kind, its target — the core's
/// rendered text, not a number the app has to format — and whether a run
/// happened.
pub fn week(api: &Api, request: &Request) -> ApiResult<Value> {
    let day = match request.query.get("date") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?,
        _ => super::today(api)?,
    };
    let start = week_start(day);
    let end = start.add_days(6);
    let active = plan::active_or_empty(api)?;
    let source = active.source_name();
    let facts = plan::run_facts(api, "0000-01-01", &end.to_iso())?;

    let sessions = stride_core::plan::week_of(&active.block, day);
    let mut rendered: Vec<PlanSession> = Vec::with_capacity(sessions.len());
    for session in sessions.iter() {
        let actual = plan::distance_on(api, &session.date.to_iso())?;
        rendered.push(plan::session_of(
            session,
            session.date,
            active.source,
            day,
            &facts,
            actual,
            actual.is_some(),
        ));
    }

    // The week's target: the sum of the sessions' targets, as the core
    // renders it.
    let metres: f64 = rendered
        .iter()
        .filter_map(|session| session.target.as_ref().and_then(|value| value.raw))
        .sum();
    let target = render::distance(if metres > 0.0 { Some(metres) } else { None });

    // The week's count, from the core.
    let week_block = CoreBlock {
        source: active.source,
        name: active.name.clone(),
        sessions: sessions.clone(),
    };
    let adherence = plan::adherence_of(stride_core::plan::adherence_of(&week_block, &facts));

    let response = PlanWeek {
        block_name: active.name.clone(),
        source: source.to_string(),
        week_start: start.to_iso(),
        week_end: end.to_iso(),
        target,
        adherence: Some(adherence),
        sessions: rendered,
    };
    ok(&response)
}

/// The plan's blocks.
///
/// The blocks that were written down. The baseline is not one of them: it
/// is inferred, and it is never stored.
pub fn blocks(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let blocks = api
        .store()
        .plan_blocks()
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    let mut rendered: Vec<PlanBlock> = Vec::with_capacity(blocks.len());
    for block in blocks.iter() {
        rendered.push(PlanBlock {
            id: Some(block.id),
            name: block.name.clone(),
            source: block.source.clone(),
            start_date: block.start_date.clone(),
            end_date: block.end_date.clone(),
            goal: block.goal.clone(),
            status: block.status.clone(),
            session_count: api
                .store()
                .plan_sessions_for_block(block.id)
                .map_err(|error| api.store_error("the plan could not be read", &error))?
                .len(),
            is_baseline: block.source == plan_source::BASELINE,
        });
    }
    let response = PlanBlocks {
        blocks: rendered,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's sources.
///
/// The three sources and which one is in force. **The source is always
/// named** — there is no answer that leaves the app without saying where
/// the plan came from.
pub fn sources(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let blocks = api
        .store()
        .plan_blocks()
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    let active = plan::active(api)?;
    let mut rendered: Vec<PlanSource> = Vec::with_capacity(plan::SOURCES.len());
    for source in plan::SOURCES.iter() {
        let name = plan::source_name(*source);
        let of_source: Vec<&stride_store::PlanBlock> =
            blocks.iter().filter(|block| block.source == name).collect();
        let stored_active = of_source
            .iter()
            .any(|block| block.status == plan_status::ACTIVE);
        // The baseline is in force when the app inferred it — there is no
        // block of it in the store, and it is in force all the same.
        let inferred = active
            .as_ref()
            .map(|active| active.is_baseline())
            .unwrap_or(false);
        rendered.push(PlanSource {
            name: name.to_string(),
            blocks: of_source.len(),
            active: stored_active || (inferred && name == plan_source::BASELINE),
        });
    }
    let response = PlanSources {
        sources: rendered,
        active_source: active
            .as_ref()
            .map(|active| active.source_name().to_string())
            .unwrap_or_else(|| plan_source::BASELINE.to_string()),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The adherence: how many of the planned sessions were run.
///
/// A **count**, never a percentage: the contract drops the old adherence
/// percentage, and "7 of 8 sessions run" is a fact. While the plan is
/// paused the count is suspended, and the answer says so.
pub fn adherence(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let active = plan::active_or_empty(api)?;
    let facts = plan::run_facts(api, "0000-01-01", &current.to_iso())?;
    let count = plan::adherence_of(stride_core::plan::adherence_of(&active.block, &facts));

    // The weeks of the block, each counted by the core.
    let mut weeks: Vec<AdherenceWeek> = Vec::new();
    for session in active.block.sessions.iter() {
        let start = week_start(session.date);
        let key = start.to_iso();
        let in_week: Vec<stride_core::plan::PlanSession> = active
            .block
            .sessions
            .iter()
            .filter(|other| week_start(other.date) == session.date)
            .cloned()
            .collect();
        if weeks.iter().any(|week| week.week_start == key) {
            continue;
        }
        let week_block = CoreBlock {
            source: active.source,
            name: active.name.clone(),
            sessions: in_week,
        };
        let week_count = plan::adherence_of(stride_core::plan::adherence_of(&week_block, &facts));
        weeks.push(AdherenceWeek {
            week_start: key,
            done: week_count.done,
            planned: week_count.planned,
        });
    }

    let reason = crate::routes::settings::suspension_reason(api, current)?;
    let response = PlanAdherence {
        adherence: count,
        weeks,
        suspended: reason.is_some(),
        suspension_reason: reason,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's results: every session, and what happened.
///
/// The intent, the actual, and the difference between the two — stated
/// neutrally, and only when there is a difference to state.
pub fn results(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let active = plan::active_or_empty(api)?;
    let facts = plan::run_facts(api, "0000-01-01", &current.to_iso())?;
    let rendered: Vec<PlanResult> = plan::results_of(api, &active, &facts)?;
    let response = PlanResults {
        results: rendered,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Import a plan.
///
/// Two ways in:
///
/// * **a file** — markdown, CSV, JSON, or the Garmin workout schedule. The
///   plan is recorded as **prescribed**: it was written down somewhere
///   else.
/// * **a plan the athlete typed in** — the app's own form. It is recorded
///   as **typed**.
///
/// A file that cannot be read is **refused**: the import says which line it
/// could not read, and **nothing is written**. A session whose workout the
/// vocabulary does not know is **unclassified** — it is never given a kind
/// it did not have.
pub fn import(api: &Api, request: &Request) -> ApiResult<Value> {
    let body = match &request.body {
        Some(body) => body.clone(),
        None => return Err(ApiError::bad_request("the import has no plan in it")),
    };

    let (sessions, source, name, start_date, end_date, goal) = read_plan(api, &body)?;

    let block = NewPlanBlock {
        name: name.clone(),
        source: source.to_string(),
        start_date: start_date.clone(),
        end_date: end_date.clone(),
        goal: goal.clone(),
        status: plan_status::ACTIVE.to_string(),
        notes: None,
        created_at: api.now(),
    };
    let block_id = api
        .store()
        .upsert_plan_block(&block)
        .map_err(|error| api.store_error("the plan could not be written", &error))?;

    let mut written = 0usize;
    let mut unclassified = 0usize;
    for session in sessions.iter() {
        if session.session_type == "unclassified" {
            unclassified += 1;
        }
        let planned = NewPlanSession {
            block_id,
            date: session.date.to_iso(),
            session_type: session.session_type.clone(),
            title: session.title.clone(),
            details: session.details.clone(),
            target_distance_m: session.target_distance_m,
            pace_low_s_km: None,
            pace_high_s_km: None,
        };
        api.store()
            .upsert_plan_session(&planned)
            .map_err(|error| api.store_error("the plan could not be written", &error))?;
        written += 1;
    }

    let response = PlanImport {
        imported: true,
        block_id,
        sessions: written,
        source: source.to_string(),
        unclassified,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// One block, as it travels.
fn block_of(api: &Api, active: &plan::ActivePlan) -> ApiResult<PlanBlock> {
    let session_count = active.block.sessions.len();
    let _ = api;
    Ok(PlanBlock {
        id: active.stored.as_ref().map(|block| block.id),
        name: active.name.clone(),
        source: active.source_name().to_string(),
        start_date: active.start_date.clone(),
        end_date: active.end_date.clone(),
        goal: active.goal.clone(),
        status: match active.stored.as_ref() {
            Some(block) => block.status.clone(),
            None => plan_status::ACTIVE.to_string(),
        },
        session_count,
        is_baseline: active.is_baseline(),
    })
}

/// The plan in the request: the sessions, and where the plan came from.
///
/// The sessions are read whole before anything is written: a plan that
/// cannot be read changes nothing.
#[allow(clippy::type_complexity)]
fn read_plan(
    api: &Api,
    body: &serde_json::Value,
) -> ApiResult<(
    Vec<FileSession>,
    &'static str,
    String,
    String,
    Option<String>,
    Option<String>,
)> {
    let today = super::today(api)?;
    let content = body
        .get("content")
        .and_then(|value| value.as_str())
        .map(|text| text.to_string());

    match content {
        // A file the athlete imported.
        Some(content) => {
            let format = match body.get("format").and_then(|value| value.as_str()) {
                Some(name) => planfile::Format::from_name(name).ok_or_else(|| {
                    ApiError::bad_request("the format of the plan could not be read")
                })?,
                None => match body
                    .get("file_name")
                    .and_then(|value| value.as_str())
                    .and_then(planfile::Format::from_file_name)
                {
                    Some(format) => format,
                    None => {
                        return Err(ApiError::bad_request(
                            "the format of the plan could not be read",
                        ))
                    }
                },
            };
            let plan = planfile::read(format, &content)?;
            let name = body
                .get("name")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .or(plan.name)
                .or_else(|| {
                    body.get("file_name")
                        .and_then(|value| value.as_str())
                        .map(|text| text.trim().to_string())
                        .filter(|text| !text.is_empty())
                })
                .unwrap_or_else(|| "Imported plan".to_string());
            let start = plan
                .start_date
                .or_else(|| {
                    body.get("start_date")
                        .and_then(|value| value.as_str())
                        .map(|text| text.trim().to_string())
                        .filter(|text| !text.is_empty())
                })
                .unwrap_or_else(|| {
                    plan.sessions
                        .iter()
                        .map(|session| session.date)
                        .min()
                        .map(|date| date.to_iso())
                        .unwrap_or_else(|| today.to_iso())
                });
            let end = plan.end_date.or_else(|| {
                body.get("end_date")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
            });
            let goal = plan.goal.or_else(|| {
                body.get("goal")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
            });
            Ok((
                plan.sessions,
                plan_source::PRESCRIBED,
                name,
                start,
                end,
                goal,
            ))
        }
        // A plan the athlete typed in.
        None => {
            let sessions = body
                .get("sessions")
                .and_then(|value| value.as_array())
                .ok_or_else(|| ApiError::bad_request("the import has no plan in it"))?;
            let mut read: Vec<FileSession> = Vec::with_capacity(sessions.len());
            for (index, session) in sessions.iter().enumerate() {
                let date = session
                    .get("date")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_string())
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| {
                        ApiError::bad_request(format!(
                            "the plan could not be read: session {} has no date",
                            index + 1
                        ))
                    })?;
                let date = core_date(&date)?;
                let session_type = session
                    .get("session_type")
                    .and_then(|value| value.as_str())
                    .map(|text| text.trim().to_ascii_lowercase())
                    .filter(|text| !text.is_empty())
                    .unwrap_or_else(|| "unclassified".to_string());
                read.push(FileSession {
                    date,
                    session_type,
                    title: session
                        .get("title")
                        .and_then(|value| value.as_str())
                        .map(|text| text.to_string()),
                    target_distance_m: session
                        .get("target_distance_m")
                        .and_then(|value| value.as_f64())
                        .filter(|value| value.is_finite() && *value >= 0.0),
                    details: session
                        .get("details")
                        .and_then(|value| value.as_str())
                        .map(|text| text.to_string()),
                });
            }
            if read.is_empty() {
                return Err(ApiError::bad_request("the import has no plan in it"));
            }
            let name = body
                .get("name")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "Typed plan".to_string());
            let start = body
                .get("start_date")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| {
                    read.iter()
                        .map(|session| session.date)
                        .min()
                        .map(|date| date.to_iso())
                        .unwrap_or_else(|| today.to_iso())
                });
            let end = body
                .get("end_date")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty());
            let goal = body
                .get("goal")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty());
            Ok((read, plan_source::TYPED, name, start, end, goal))
        }
    }
}
