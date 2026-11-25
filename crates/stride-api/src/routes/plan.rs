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
//! the sessions of the week, the count, the status of a session and the
//! results. This crate reads the store, hands the core the domain types it
//! works on, and carries the core's answers back. The status of a session
//! is **computed on read** and never stored — a stored verdict would be a
//! judgement the plan forbids, and a baseline never yields "not done": an
//! inferred baseline is not a prescription.
//!
//! ## The sources
//!
//! Three: a plan the athlete **imported** (`prescribed`), a plan the
//! athlete **typed** in (`typed`), and the **baseline** the app inferred
//! from the athlete's own history (`baseline`). The source is always named
//! in every answer, so the app never shows a plan without saying where it
//! came from.

use std::collections::BTreeMap;

use serde_json::Value;
use stride_core::plan::PlanBlock as CoreBlock;
use stride_store::model::{plan_source, plan_status};
use stride_store::{NewPlanBlock, NewPlanSession};

use crate::dto::{
    AdherenceWeek, AdoptionInsight, AdoptionWritten, Counts, CurrentPlan, PlanAdherence, PlanBlock,
    PlanBlocks, PlanImport, PlanResult, PlanResults, PlanSession, PlanSource, PlanSources,
    PlanToday, PlanWeek,
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
        block: active.as_ref().map(block_of),
        source: active
            .as_ref()
            .map(|active| active.source_name())
            .unwrap_or_else(|| plan_source::BASELINE)
            .to_string(),
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
    let results = stride_core::plan::results_of_at(&active.block, &facts, current);
    let planned = results
        .iter()
        .find(|result| result.session.date == current)
        .map(|result| plan::session_of(&active, result, current));
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
/// rendered text, not a number the app has to format — and the plain
/// status of the session. The week's counts are the api's count of the
/// core's statuses.
///
/// The week also carries the **adopted** plan's own week, when a plan is
/// in force: the sessions it holds, their kinds, their distances, their
/// paces where they have one, and the plan's own source.
pub fn week(api: &Api, request: &Request) -> ApiResult<Value> {
    let day = match request.query.get("date") {
        Some(text) if !text.trim().is_empty() => core_date(text.trim())?,
        _ => super::today(api)?,
    };
    let start = week_start(day);
    let end = start.add_days(6);
    let active = plan::active_or_empty(api)?;
    let facts = plan::run_facts(api, "0000-01-01", &end.to_iso())?;

    let results = stride_core::plan::results_of_at(&active.block, &facts, day);
    let rendered: Vec<PlanSession> = results
        .iter()
        .filter(|result| result.session.date >= start && result.session.date <= end)
        .map(|result| plan::session_of(&active, result, day))
        .collect();

    // The week's counts: the api counts the core's statuses, so the app
    // never counts the plan's sessions itself.
    let counts = Counts {
        done: rendered
            .iter()
            .filter(|session| session.status == "done")
            .count(),
        planned: rendered
            .iter()
            .filter(|session| session.status == "planned")
            .count(),
        not_done: rendered
            .iter()
            .filter(|session| session.status == "not_done")
            .count(),
    };

    // The week's target: the sum of the sessions' targets, as the core
    // renders it.
    let metres: f64 = render::total_of(
        rendered
            .iter()
            .filter_map(|session| session.target.as_ref().and_then(|value| value.raw)),
    );
    let target = render::distance(if metres > 0.0 { Some(metres) } else { None });

    // The adopted plan's own week, when a plan is in force and the week
    // asked for is in it — **and the week against what happened**. The
    // comparison is the core's: every session says whether it was done,
    // and the totals say how the week stands against the plan's.
    let comparison = crate::adoption::comparison(api, day)?;
    let adopted = crate::adoption::week_at(api, day)?
        .map(|week| crate::adoption::week_row(&week, &comparison, day));

    let response = PlanWeek {
        block_name: active.name.clone(),
        source: active.source_name().to_string(),
        week_start: start.to_iso(),
        week_end: end.to_iso(),
        today: super::today(api)?.to_iso(),
        target,
        adherence: Some(crate::dto::Adherence {
            done: counts.done,
            planned: counts.done + counts.planned + counts.not_done,
        }),
        counts,
        sessions: rendered,
        adopted,
        against: crate::adoption::against_row(&comparison),
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
        let sessions = api
            .store()
            .plan_sessions_for_block(block.id)
            .map_err(|error| api.store_error("the plan could not be read", &error))?;
        rendered.push(PlanBlock {
            id: Some(block.id),
            name: block.name.clone(),
            source: block.source.clone(),
            start_date: block.start_date.clone(),
            end_date: block.end_date.clone(),
            goal: block.goal.clone(),
            status: block.status.clone(),
            session_count: sessions.len(),
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
        let name = source.as_str();
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
            .map(|active| active.source_name())
            .unwrap_or_else(|| plan_source::BASELINE)
            .to_string(),
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
    let count = stride_core::plan::adherence_of_at(&active.block, &facts, current);

    // The weeks of the block, each counted by the engine.
    let mut by_week: BTreeMap<String, Vec<stride_core::plan::PlanSession>> = BTreeMap::new();
    for session in active.block.sessions.iter() {
        by_week
            .entry(week_start(session.date).to_iso())
            .or_default()
            .push(session.clone());
    }
    let mut weeks: Vec<AdherenceWeek> = Vec::with_capacity(by_week.len());
    for (week, sessions) in by_week.iter() {
        let week_block = CoreBlock::new(active.source, active.name.clone(), sessions.clone());
        let week_count = stride_core::plan::adherence_of_at(&week_block, &facts, current);
        let counts = plan::counts_of(week_count);
        weeks.push(AdherenceWeek {
            week_start: week.clone(),
            done: counts.done,
            planned: counts.done + counts.planned + counts.not_done,
            not_done: counts.not_done,
        });
    }

    let reason = crate::routes::settings::suspension_reason(api, current)?;
    let response = PlanAdherence {
        adherence: plan::adherence_of(count),
        counts: plan::counts_of(count),
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
    let rendered: Vec<PlanResult> = plan::results_of(&active, &facts, current);
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
    for session in sessions.iter() {
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
        block_id: Some(block_id),
        sessions: written,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// One block, as it travels.
fn block_of(active: &plan::ActivePlan) -> PlanBlock {
    PlanBlock {
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
        session_count: active.block.sessions.len(),
        is_baseline: active.is_baseline(),
    }
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

/// The programmes the app can adopt.
///
/// The seam to the core's catalogue is [`crate::planbases`], and the
/// catalogue is empty until the plan's own step lands the templates.
pub fn bases(api: &Api, _request: &Request) -> ApiResult<Value> {
    ok(&crate::planbases::bases(api))
}

// ---------------------------------------------------------------- adoption

/// What is adopted.
///
/// The plan in force: its programme, its tier, its dates, what it was
/// paced with and the threshold it was paced with, and the tier's entry
/// requirement **beside the athlete's own weekly mileage**.
///
/// The entry requirement and the athlete's own mileage are three facts side
/// by side, unranked (B2): the app states them and never compares them,
/// never marks one as the right one, and never says which level fits.
pub fn adopted(api: &Api, _request: &Request) -> ApiResult<Value> {
    let plan = crate::adoption::in_force(api)?;
    let answer = match plan.as_ref() {
        Some(plan) => {
            let words = crate::adoption::words_of(plan);
            AdoptionInsight {
                generated_at: api.generated_at(),
                plan: Some(crate::adoption::plan_row(plan, &words)),
                unavailable: None,
            }
        }
        None => AdoptionInsight {
            generated_at: api.generated_at(),
            plan: None,
            unavailable: crate::routes::core_line("statement.plan.adoption.none"),
        },
    };
    ok(&answer)
}

/// Adopt a programme.
///
/// The body names the **programme** and the **tier**, the plan's first
/// day, the goal race's distance and the day it is run, and the threshold
/// to pace with — a candidate, or a number the athlete brought.
///
/// The **paces are worked out once, here**, from the threshold in force on
/// the day of the adoption, and they are stored with the plan. A later
/// threshold does not re-pace a running plan.
///
/// The refusals, each with the core's own reason: a programme or a tier
/// that is not in the catalogue, a start date that leaves no room for the
/// twelve weeks and the race week, a goal race the programme does not
/// print, and a plan that is already in force.
///
/// **The athlete's own mileage being below the tier's entry requirement is
/// not a refusal.** The app states the requirement beside their mileage
/// and the athlete chooses.
pub fn adopt(api: &Api, request: &Request) -> ApiResult<Value> {
    let today = super::today(api)?;

    // The programme and the tier the athlete chose.
    let programme_id = body_text(request, "programme_id")
        .ok_or_else(|| ApiError::bad_request("the adoption needs the programme it is for"))?;
    let tier = body_text(request, "tier");
    let programme = stride_core::plan_bases::programme_of(&programme_id, tier.as_deref())
        .map_err(|refusal| refusal_error(&refusal))?;

    // The goal race: its distance and the day it is run.
    let race_date = body_date(request, "race_date")?
        .ok_or_else(|| ApiError::bad_request("the adoption needs the day of the goal race"))?;
    let start_date = body_date(request, "start_date")?;
    let goal_distance_m = match request.body_f64("goal_distance_m")? {
        Some(distance) if distance.is_finite() && distance > 0.0 => distance,
        Some(_) => {
            return Err(ApiError::bad_request(
                "the goal race's distance is not a number",
            ))
        }
        None => programme.goal_race.distance_m,
    };

    // The threshold to pace with, and the race the plan's own rule is read
    // from.
    let threshold = threshold_of(api, request, today)?;
    let race = race_of(api)?;

    // The athlete's own recent mileage: the app computes it, the core only
    // states it beside the tier's entry requirement.
    let mileage = crate::adoption::mileage(api)?;

    let input = stride_core::plan_bases::AdoptionInput {
        programme_id: programme.id.clone(),
        goal_distance_m,
        race_date,
        adopted_on: today,
        start_date,
        threshold,
        race,
        weekly_mileage_m: mileage,
    };
    let plan = stride_core::plan_bases::adopt(&input).map_err(|refusal| refusal_error(&refusal))?;

    // **One plan at a time.** A second plan cannot be adopted while
    // another one is in force.
    let in_force = api
        .store()
        .active_adopted_plans()
        .map_err(|error| api.store_error("the adopted plans could not be read", &error))?;
    for existing in in_force.iter() {
        let overlaps = !(existing.end_date < plan.start_date.to_iso()
            || existing.start_date > plan.end_date.to_iso());
        if overlaps {
            let until = core_date(&existing.end_date)?;
            return Err(refusal_error(
                &stride_core::plan_bases::AdoptionRefusal::PlanInForce { until },
            ));
        }
    }

    // The plan and its own sessions, written together.
    let (row, sessions) = crate::adoption::rows_of(&plan, api.now());
    api.store()
        .write_adopted_plan(&row, &sessions)
        .map_err(|error| api.store_error("the adopted plan could not be written", &error))?;

    let words = crate::adoption::words_of(&plan);
    let answer = AdoptionWritten {
        adopted: true,
        plan: crate::adoption::plan_row(&plan, &words),
        generated_at: api.generated_at(),
    };
    ok(&answer)
}

/// A refusal, as the api carries it: the core's own reason.
fn refusal_error(refusal: &stride_core::plan_bases::AdoptionRefusal) -> ApiError {
    let reason = refusal
        .reason()
        .unwrap_or_else(|| "the adoption could not be made".to_string());
    ApiError::bad_request(reason)
}

/// The threshold the athlete named, as the core reads it.
///
/// The body names which candidate the plan is paced with — the watch's own
/// zone model, a race, a field test, or a number the athlete brought — and
/// the value comes from that candidate, so the plan and the candidate
/// cannot disagree. A typed choice carries the value itself.
///
/// `None` when the body names no threshold: the plan is then paced by the
/// threshold **in force on the day of the adoption**, and by nothing else.
fn threshold_of(
    api: &Api,
    request: &Request,
    today: stride_core::date::Date,
) -> ApiResult<Option<stride_core::threshold::ThresholdChoice>> {
    use stride_core::threshold::{ThresholdBase, ThresholdChoice, ThresholdSource};

    let named = match request.body.as_ref().and_then(|body| body.get("threshold")) {
        Some(named) => named.clone(),
        // No threshold is named: the **threshold in force on the day of
        // the adoption** paces the plan, and nothing else does (B4).
        None => return in_force_threshold(api, today),
    };
    let source = named
        .get("source")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    let base = match named
        .get("base")
        .and_then(|value| value.as_str())
        .unwrap_or("pfitzinger")
    {
        "eighty_twenty" => ThresholdBase::EightyTwenty,
        _ => ThresholdBase::Pfitzinger,
    };

    match source.as_str() {
        "watch_zone_model" | "race" | "field_test" => {
            let facts = crate::athlete::Facts::read(api, today)?;
            let run_id = named
                .get("origin_ref")
                .and_then(|value| value.as_str())
                .and_then(|text| text.trim().parse::<i64>().ok());
            let inputs = super::threshold::inputs_of(api, &facts, run_id)?;
            let computed = stride_core::threshold::candidates(&inputs);
            let candidate = computed
                .iter()
                .find(|candidate| candidate.source.key() == source)
                .ok_or_else(|| {
                    ApiError::bad_request("the choice names a candidate that is not there")
                })?;
            let heart_rate_bpm = candidate.value.heart_rate_bpm();
            let pace = candidate.value.pace();
            Ok(Some(ThresholdChoice {
                source: candidate.source,
                heart_rate_bpm,
                pace,
                rule_id: candidate.rule.map(|rule| rule.id.to_string()),
                base,
                origin_date: candidate.date,
                origin_ref: None,
                label: None,
                entered_on: today,
                updated_on: today,
                in_force: false,
            }))
        }
        "typed" => {
            let heart_rate_bpm = number_of(&named, "heart_rate_bpm");
            let fast = number_of(&named, "pace_fast_s_per_km");
            let slow = number_of(&named, "pace_slow_s_per_km");
            if heart_rate_bpm.is_none() && fast.is_none() {
                return Err(ApiError::bad_request("the choice carries no value"));
            }
            let label = named
                .get("label")
                .and_then(|value| value.as_str())
                .and_then(super::athlete::typed_source_of)
                .ok_or_else(|| ApiError::bad_request("the label is not one the app knows"))?;
            let (low, high) = match (fast, slow) {
                (Some(fast), Some(slow)) => (Some(fast.min(slow)), Some(fast.max(slow))),
                (Some(fast), None) => (Some(fast), Some(fast)),
                (None, Some(slow)) => (Some(slow), Some(slow)),
                (None, None) => (None, None),
            };
            let pace = low.map(|low| {
                stride_core::training_paces::PaceBand::between(low, high.unwrap_or(low))
            });
            Ok(Some(ThresholdChoice {
                source: ThresholdSource::Typed,
                heart_rate_bpm,
                pace,
                rule_id: None,
                base,
                origin_date: None,
                origin_ref: None,
                label: Some(label),
                entered_on: today,
                updated_on: today,
                in_force: false,
            }))
        }
        _ => Err(ApiError::bad_request(
            "the candidate is not one of the four",
        )),
    }
}

/// The threshold in force on the day of the adoption.
///
/// The athlete's own choice, dated: the last choice made on or before the
/// day the plan was adopted. A choice made later does not reach back into
/// a plan that is already running.
fn in_force_threshold(
    api: &Api,
    today: stride_core::date::Date,
) -> ApiResult<Option<stride_core::threshold::ThresholdChoice>> {
    let stored = api
        .store()
        .threshold_choices()
        .map_err(|error| api.store_error("the threshold choices could not be read", &error))?;
    let choices: Vec<stride_core::threshold::ThresholdChoice> = stored
        .iter()
        .filter_map(super::threshold::choice_of)
        .collect();
    Ok(stride_core::threshold::in_force_at(&choices, today).cloned())
}

/// The race the plan's own rule is read from: the athlete's own race.
fn race_of(api: &Api) -> ApiResult<Option<stride_core::training_paces::Race>> {
    let stored = api
        .store()
        .anchor_race()
        .map_err(|error| api.store_error("the race could not be read", &error))?;
    Ok(stored.as_ref().and_then(|race| {
        let date = stride_core::date::Date::parse(&race.race_date)?;
        Some(stride_core::training_paces::Race::new(
            race.distance_m,
            race.time_s,
            date,
        ))
    }))
}

/// A field of the body, as a trimmed string. A number counts as one too.
fn body_text(request: &Request, name: &str) -> Option<String> {
    let value = request.body.as_ref()?.get(name)?;
    let text = match value {
        serde_json::Value::String(text) => text.trim().to_string(),
        serde_json::Value::Number(number) => number.to_string(),
        _ => return None,
    };
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// A field of the body, as a day.
fn body_date(request: &Request, name: &str) -> ApiResult<Option<stride_core::date::Date>> {
    let text = match body_text(request, name) {
        Some(text) => text,
        None => return Ok(None),
    };
    match core_date(&text) {
        Ok(date) => Ok(Some(date)),
        Err(_) => Err(ApiError::bad_request("the day could not be read")),
    }
}

/// A number in a body object.
fn number_of(body: &serde_json::Value, name: &str) -> Option<f64> {
    body.get(name)
        .and_then(|value| value.as_f64())
        .filter(|value| value.is_finite() && *value > 0.0)
}
