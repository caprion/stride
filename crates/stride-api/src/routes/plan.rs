//! The plan routes: the plan the athlete has, and what it says.
//!
//! The plan is **plain fact**: a session is *done*, *not done* or
//! *planned*. Nothing here grades the athlete, and nothing here says that
//! a session should have happened — the word "missed" does not exist in
//! this crate.
//!
//! The status of a session is **computed on read**, from the session's
//! date, the runs of that day and the block's source. A stored verdict
//! would be a judgement the plan forbids, and a baseline never yields
//! "not done": an inferred baseline is not a prescription.

use serde_json::Value;
use stride_core::date::Date;
use stride_core::units;
use stride_store::model::{plan_source, plan_status};
use stride_store::{NewPlanBlock, NewPlanSession, PlanBlock as StoredBlock, PlanSession as StoredSession};

use crate::dto::{
    Adherence, AdherenceWeek, CurrentPlan, PlanAdherence, PlanBlock, PlanBlocks, PlanImport,
    PlanResult, PlanResults, PlanSession, PlanSource, PlanSources, PlanToday, PlanWeek,
};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;

use super::{core_date, ok, runs_on, session_sentence, session_status, week_start};

/// The sources a plan may come from, in the order the app prefers them.
///
/// The athlete's own plan comes first: a plan the athlete wrote or imported
/// is the plan in force; a baseline is what the app inferred, and it is
/// the last thing the app offers.
const SOURCES: [&str; 3] = [
    plan_source::PRESCRIBED,
    plan_source::TYPED,
    plan_source::BASELINE,
];

/// The block in force.
pub fn current(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let block = active_block(api)?;
    let response = CurrentPlan {
        block: match block {
            Some(block) => Some(block_of(api, &block)?),
            None => None,
        },
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's day.
pub fn today(api: &Api, _request: &Request) -> ApiResult<Value> {
    let current = super::today(api)?;
    let sessions = api
        .store()
        .plan_sessions_for_day(&current.to_iso())
        .map_err(|_| ApiError::internal("the plan could not be read"))?;
    let source = source_of_sessions(api, &sessions);
    let mut rendered: Vec<PlanSession> = Vec::new();
    for session in sessions.iter() {
        rendered.push(session_of(api, session, &source, current)?);
    }
    let response = PlanToday {
        date: current.to_iso(),
        session: rendered.into_iter().next(),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's week.
pub fn week(api: &Api, request: &Request) -> ApiResult<Value> {
    let day = match request.query.get("date") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?,
        _ => super::today(api)?,
    };
    let start = week_start(day);
    let end = start.add_days(6);
    let sessions = api
        .store()
        .plan_sessions_between(&start.to_iso(), &end.to_iso())
        .map_err(|_| ApiError::internal("the plan could not be read"))?;
    let source = source_of_sessions(api, &sessions);
    let mut rendered: Vec<PlanSession> = Vec::new();
    for session in sessions.iter() {
        rendered.push(session_of(api, session, &source, day)?);
    }
    let done = rendered
        .iter()
        .filter(|session| session.status == "done")
        .count();
    let target_km = {
        let metres: f64 = rendered
            .iter()
            .filter_map(|session| session.target_distance_m)
            .sum();
        if metres > 0.0 {
            Some(units::round_km(metres / 1000.0))
        } else {
            None
        }
    };
    let block_name = block_name_of(api, &sessions)?;
    let response = PlanWeek {
        block_name,
        source,
        week_start: start.to_iso(),
        week_end: end.to_iso(),
        target_km,
        adherence: Some(Adherence {
            done,
            planned: rendered.len(),
        }),
        sessions: rendered,
    };
    ok(&response)
}

/// The plan's blocks.
pub fn blocks(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let blocks = api
        .store()
        .plan_blocks()
        .map_err(|_| ApiError::internal("the plan could not be read"))?;
    let mut rendered: Vec<PlanBlock> = Vec::new();
    for block in blocks.iter() {
        rendered.push(block_of(api, block)?);
    }
    let response = PlanBlocks {
        blocks: rendered,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's sources.
pub fn sources(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let blocks = api
        .store()
        .plan_blocks()
        .map_err(|_| ApiError::internal("the plan could not be read"))?;
    let mut rendered: Vec<PlanSource> = Vec::new();
    for source in SOURCES.iter() {
        let of_source: Vec<&StoredBlock> = blocks
            .iter()
            .filter(|block| block.source == *source)
            .collect();
        rendered.push(PlanSource {
            name: source.to_string(),
            blocks: of_source.len(),
            active: of_source
                .iter()
                .any(|block| block.status == plan_status::ACTIVE),
        });
    }
    let response = PlanSources {
        sources: rendered,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The adherence: how many of the planned sessions were run.
///
/// A **count**, never a percentage: the contract drops the old adherence
/// percentage, and "7 of 8 sessions run" is a fact.
pub fn adherence(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let block = active_block(api)?;
    let sessions = match &block {
        Some(block) => api
            .store()
            .plan_sessions_for_block(block.id)
            .map_err(|_| ApiError::internal("the plan could not be read"))?,
        None => Vec::new(),
    };
    let source = block
        .as_ref()
        .map(|block| block.source.clone())
        .unwrap_or_else(|| plan_source::TYPED.to_string());
    let mut rendered: Vec<PlanSession> = Vec::new();
    for session in sessions.iter() {
        rendered.push(session_of(api, session, &source, current)?);
    }
    let done = rendered
        .iter()
        .filter(|session| session.status == "done")
        .count();

    let mut weeks: Vec<AdherenceWeek> = Vec::new();
    for session in rendered.iter() {
        let start = match core_date(&session.date) {
            Ok(date) => week_start(date),
            Err(_) => continue,
        };
        let key = start.to_iso();
        match weeks.iter_mut().find(|week| week.week_start == key) {
            Some(week) => {
                week.planned += 1;
                if session.status == "done" {
                    week.done += 1;
                }
            }
            None => weeks.push(AdherenceWeek {
                week_start: key,
                done: usize::from(session.status == "done"),
                planned: 1,
            }),
        }
    }
    let response = PlanAdherence {
        adherence: Adherence {
            done,
            planned: rendered.len(),
        },
        weeks,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// The plan's results: every session, and what happened.
pub fn results(api: &Api, request: &Request) -> ApiResult<Value> {
    let _ = request;
    let current = super::today(api)?;
    let block = active_block(api)?;
    let sessions = match &block {
        Some(block) => api
            .store()
            .plan_sessions_for_block(block.id)
            .map_err(|_| ApiError::internal("the plan could not be read"))?,
        None => Vec::new(),
    };
    let source = block
        .as_ref()
        .map(|block| block.source.clone())
        .unwrap_or_else(|| plan_source::TYPED.to_string());
    let mut rendered: Vec<PlanResult> = Vec::new();
    for session in sessions.iter() {
        let read = session_of(api, session, &source, current)?;
        rendered.push(PlanResult {
            session_id: session.id,
            date: read.date,
            session_type: read.session_type,
            status: read.status,
            target_distance_m: read.target_distance_m,
            actual_distance_m: read.actual_distance_m,
        });
    }
    let response = PlanResults {
        results: rendered,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Import a plan.
///
/// The plan arrives as a block and its sessions, and it is written through
/// the store — so importing the same plan twice updates the plan that is
/// already there instead of adding a second one.
pub fn import(api: &Api, request: &Request) -> ApiResult<Value> {
    let body = match &request.body {
        Some(body) => body.clone(),
        None => return Err(ApiError::bad_request("the import has no plan in it")),
    };
    let name = body
        .get("name")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ApiError::bad_request("the plan has no name"))?;
    let source = body
        .get("source")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| SOURCES.contains(&text.as_str()))
        .unwrap_or_else(|| plan_source::TYPED.to_string());
    let default_start = super::today(api)?.to_iso();
    let start_date = body
        .get("start_date")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or(default_start);
    let start_date = core_date(&start_date)?.to_iso();
    let end_date = match body.get("end_date").and_then(|value| value.as_str()) {
        Some(text) if !text.trim().is_empty() => Some(core_date(text.trim())?.to_iso()),
        _ => None,
    };
    let goal = body
        .get("goal")
        .and_then(|value| value.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());

    let block = NewPlanBlock {
        name: name.clone(),
        source: source.clone(),
        start_date: start_date.clone(),
        end_date,
        goal,
        status: plan_status::ACTIVE.to_string(),
        notes: None,
        created_at: api.now(),
    };
    let block_id = api
        .store()
        .upsert_plan_block(&block)
        .map_err(|_| ApiError::internal("the plan could not be written"))?;

    let mut written = 0usize;
    if let Some(sessions) = body.get("sessions").and_then(|value| value.as_array()) {
        for session in sessions.iter() {
            let date = session
                .get("date")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .ok_or_else(|| ApiError::bad_request("a session has no date"))?;
            let date = core_date(&date)?.to_iso();
            let session_type = session
                .get("session_type")
                .and_then(|value| value.as_str())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "unclassified".to_string());
            let planned = NewPlanSession {
                block_id,
                date,
                session_type,
                title: session
                    .get("title")
                    .and_then(|value| value.as_str())
                    .map(|text| text.to_string()),
                details: session
                    .get("details")
                    .and_then(|value| value.as_str())
                    .map(|text| text.to_string()),
                target_distance_m: session
                    .get("target_distance_m")
                    .and_then(|value| value.as_f64())
                    .filter(|value| value.is_finite() && *value >= 0.0),
                pace_low_s_km: session
                    .get("pace_low_s_km")
                    .and_then(|value| value.as_f64())
                    .filter(|value| value.is_finite()),
                pace_high_s_km: session
                    .get("pace_high_s_km")
                    .and_then(|value| value.as_f64())
                    .filter(|value| value.is_finite()),
            };
            api.store()
                .upsert_plan_session(&planned)
                .map_err(|_| ApiError::internal("the plan could not be written"))?;
            written += 1;
        }
    }

    let response = PlanImport {
        imported: true,
        block_id,
        sessions: written,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The block in force: the athlete's own plan, and then the inferred one.
fn active_block(api: &Api) -> ApiResult<Option<StoredBlock>> {
    for source in SOURCES.iter() {
        let block = api
            .store()
            .active_plan_block(source)
            .map_err(|_| ApiError::internal("the plan could not be read"))?;
        if block.is_some() {
            return Ok(block);
        }
    }
    Ok(None)
}

/// One block, as it travels.
fn block_of(api: &Api, block: &StoredBlock) -> ApiResult<PlanBlock> {
    let sessions = api
        .store()
        .plan_sessions_for_block(block.id)
        .map_err(|_| ApiError::internal("the plan could not be read"))?;
    Ok(PlanBlock {
        id: block.id,
        name: block.name.clone(),
        source: block.source.clone(),
        start_date: block.start_date.clone(),
        end_date: block.end_date.clone(),
        goal: block.goal.clone(),
        status: block.status.clone(),
        session_count: sessions.len(),
    })
}

/// The source the sessions of a block came from.
fn source_of_sessions(api: &Api, sessions: &[StoredSession]) -> String {
    if sessions.is_empty() {
        return plan_source::TYPED.to_string();
    }
    let block_id = sessions[0].block_id;
    let blocks = api.store().plan_blocks();
    match blocks {
        Ok(blocks) => blocks
            .iter()
            .find(|block| block.id == block_id)
            .map(|block| block.source.clone())
            .unwrap_or_else(|| plan_source::TYPED.to_string()),
        Err(_) => plan_source::TYPED.to_string(),
    }
}

/// The name of the block the sessions came from.
fn block_name_of(api: &Api, sessions: &[StoredSession]) -> ApiResult<String> {
    if sessions.is_empty() {
        return Ok("No plan".to_string());
    }
    let block_id = sessions[0].block_id;
    let blocks = api
        .store()
        .plan_blocks()
        .map_err(|_| ApiError::internal("the plan could not be read"))?;
    Ok(blocks
        .iter()
        .find(|block| block.id == block_id)
        .map(|block| block.name.clone())
        .unwrap_or_else(|| "No plan".to_string()))
}

/// One session, as it travels.
fn session_of(
    api: &Api,
    session: &StoredSession,
    source: &str,
    current: Date,
) -> ApiResult<PlanSession> {
    let date = core_date(&session.date)?;
    let runs = runs_on(api, &session.date)?;
    let actual_distance_m = if runs.is_empty() {
        None
    } else {
        Some(units::round_distance_m(
            runs.iter().map(|run| run.distance_m).sum(),
        ))
    };
    let status = session_status(date, current, source, !runs.is_empty());
    Ok(PlanSession {
        date: session.date.clone(),
        session_type: session.session_type.clone(),
        title: session.title.clone(),
        sentence: session_sentence(date, current, session.target_distance_m, &session.session_type),
        target_distance_m: session.target_distance_m,
        actual_distance_m,
        status: status.to_string(),
        source: source.to_string(),
    })
}
