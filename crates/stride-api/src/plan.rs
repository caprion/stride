//! The seam to the core's plan engine.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's domain types: it reads the
//! store, hands the core the types the core works on, and reads the core's
//! answers back.
//!
//! It decides nothing. There is no rule about what a plan is in here, no
//! wording of this crate's own, and no second implementation of the
//! engine's answers — the baseline, the session of a day, the sessions of a
//! week, the count and the results all come from the core, and this module
//! only carries them.
//!
//! ## The two vocabularies
//!
//! The store writes a session's kind as a word (`easy`, `long`, `tempo`,
//! `rest`, `unclassified`) and the core has its own kinds. The mapping
//! between them is here, and it is a mapping and nothing more: a word the
//! core names becomes that kind, and **a word the core does not name keeps
//! its own name** — it is never given a kind it did not have.

use stride_core::date::Date;
use stride_core::plan::{
    Adherence as CoreAdherence, PlanBlock as CoreBlock, PlanSession as CoreSession,
    PlanSource as CoreSource, RunFact, SessionKind, SessionResult as CoreSessionResult,
};
use stride_store::model::plan_source;
use stride_store::{PlanBlock as StoredBlock, PlanSession as StoredSession};

use crate::dto::{Adherence, PlanResult, PlanSession};
use crate::error::ApiResult;
use crate::render;
use crate::state::Api;

/// The sources the app prefers, in order.
///
/// The athlete's own plan comes first: a plan the athlete wrote or imported
/// is the plan in force. A baseline is what the app inferred, and it is the
/// last thing the app offers.
pub const SOURCES: [CoreSource; 3] = [
    CoreSource::Prescribed,
    CoreSource::Typed,
    CoreSource::Baseline,
];

/// The plan in force.
#[derive(Clone, Debug)]
pub struct ActivePlan {
    /// The block, as the core reads it.
    pub block: CoreBlock,
    /// The store's block, when the plan is one that was written down.
    ///
    /// `None` for the baseline: the baseline is inferred, and it is not
    /// stored.
    pub stored: Option<StoredBlock>,
    /// The source in force.
    pub source: CoreSource,
    /// The block's name.
    pub name: String,
    /// The first day of the block, `YYYY-MM-DD`.
    pub start_date: String,
    /// The last day of the block, `YYYY-MM-DD`. `None` when it is
    /// open-ended.
    pub end_date: Option<String>,
    /// The block's goal.
    pub goal: Option<String>,
    /// How many weeks of history the baseline was read from.
    ///
    /// `None` when the plan is not a baseline.
    pub baseline_weeks: Option<usize>,
}

impl ActivePlan {
    /// Whether the plan is a baseline: inferred, not written down.
    pub fn is_baseline(&self) -> bool {
        self.source == CoreSource::Baseline
    }

    /// The source's name, as the store writes it.
    pub fn source_name(&self) -> &'static str {
        source_name(self.source)
    }
}

/// The plan in force: the athlete's own plan, or the baseline.
///
/// `None` when there is no plan at all: no plan has been written down and
/// there is not enough history to infer one.
pub fn active(api: &Api) -> ApiResult<Option<ActivePlan>> {
    // The athlete's own plan wins. A baseline is the last thing the app
    // offers.
    for source in SOURCES.iter() {
        let stored = api
            .store()
            .active_plan_block(source_name(*source))
            .map_err(|error| api.store_error("the plan could not be read", &error))?;
        if let Some(stored) = stored {
            return Ok(Some(from_stored(api, &stored)?));
        }
    }

    // No plan of the athlete's own: the baseline, read from the athlete's
    // own history.
    let weeks = api.config().baseline_weeks;
    let today = crate::routes::today(api)?;
    let from = today.add_days(-(weeks as i64 * 7)).to_iso();
    let facts = run_facts(api, &from, &today.to_iso())?;
    let block = match stride_core::plan::baseline_from_history(&facts, weeks) {
        Some(block) => block,
        None => return Ok(None),
    };
    let start = block
        .sessions
        .iter()
        .map(|session| session.date)
        .min()
        .map(|date| date.to_iso())
        .unwrap_or_else(|| today.to_iso());
    let end = block
        .sessions
        .iter()
        .map(|session| session.date)
        .max()
        .map(|date| date.to_iso());
    Ok(Some(ActivePlan {
        name: block.name.clone(),
        source: block.source,
        start_date: start,
        end_date: end,
        goal: None,
        baseline_weeks: Some(weeks),
        stored: None,
        block,
    }))
}

/// The plan in force, or the failure that says there is none.
///
/// The routes that have to answer with something use this one: a plan that
/// is not there is a fact the app shows, and the answer says it.
pub fn active_or_empty(api: &Api) -> ApiResult<ActivePlan> {
    match active(api)? {
        Some(plan) => Ok(plan),
        None => Ok(ActivePlan {
            block: CoreBlock {
                source: CoreSource::Baseline,
                name: "No plan".to_string(),
                sessions: Vec::new(),
            },
            stored: None,
            source: CoreSource::Baseline,
            name: "No plan".to_string(),
            start_date: crate::routes::today(api)?.to_iso(),
            end_date: None,
            goal: None,
            baseline_weeks: None,
        }),
    }
}

/// The block the athlete's own plan holds, as the core reads it.
fn from_stored(api: &Api, stored: &StoredBlock) -> ApiResult<ActivePlan> {
    let sessions = api
        .store()
        .plan_sessions_for_block(stored.id)
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    let core_sessions: Vec<CoreSession> = sessions.iter().filter_map(core_session).collect();
    Ok(ActivePlan {
        block: CoreBlock {
            source: source_of(stored.source.as_str()),
            name: stored.name.clone(),
            sessions: core_sessions,
        },
        stored: Some(stored.clone()),
        source: source_of(stored.source.as_str()),
        name: stored.name.clone(),
        start_date: stored.start_date.clone(),
        end_date: stored.end_date.clone(),
        goal: stored.goal.clone(),
        baseline_weeks: None,
    })
}

/// One of the store's sessions, as the core reads it.
///
/// `None` when the row's day cannot be read: a session without a day is
/// not a session, and the app never invents one.
fn core_session(session: &StoredSession) -> Option<CoreSession> {
    let date = Date::parse(&session.date)?;
    Some(CoreSession {
        date,
        kind: kind_of(&session.session_type),
        target_distance_m: session.target_distance_m,
        label: session.title.clone().unwrap_or_default(),
    })
}

/// The core's kind, from the store's word for it.
///
/// A word the core names becomes that kind. **A word the core does not name
/// keeps its own name** — an imported workout the engine does not know is
/// `unclassified`, and it is never given a kind it did not have.
pub fn kind_of(session_type: &str) -> SessionKind {
    match session_type.trim().to_ascii_lowercase().as_str() {
        "easy" => SessionKind::Easy,
        "long" => SessionKind::Long,
        "quality" => SessionKind::Quality,
        "rest" => SessionKind::Rest,
        other => SessionKind::Other(other.to_string()),
    }
}

/// The kind's name, as the store writes it.
pub fn kind_name(kind: &SessionKind) -> String {
    match kind {
        SessionKind::Easy => "easy".to_string(),
        SessionKind::Long => "long".to_string(),
        SessionKind::Quality => "quality".to_string(),
        SessionKind::Rest => "rest".to_string(),
        SessionKind::Other(name) => name.clone(),
    }
}

/// The source's name, as the store writes it.
pub fn source_name(source: CoreSource) -> &'static str {
    match source {
        CoreSource::Baseline => plan_source::BASELINE,
        CoreSource::Prescribed => plan_source::PRESCRIBED,
        CoreSource::Typed => plan_source::TYPED,
    }
}

/// The core's source, from the store's word for it.
///
/// A word the store does not have is the baseline: the app's own inference
/// is the last thing it offers.
fn source_of(source: &str) -> CoreSource {
    match source {
        plan_source::PRESCRIBED => CoreSource::Prescribed,
        plan_source::TYPED => CoreSource::Typed,
        _ => CoreSource::Baseline,
    }
}

/// The facts of the runs of a window, for the engine.
///
/// A run with no distance is still a run: the distance is `None`, never `0`.
pub fn run_facts(api: &Api, from: &str, to: &str) -> ApiResult<Vec<RunFact>> {
    let runs = api
        .store()
        .activities_between(from, to)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    let mut facts: Vec<RunFact> = Vec::with_capacity(runs.len());
    for run in runs.iter() {
        let date = match Date::parse(&run.local_date) {
            Some(date) => date,
            None => continue,
        };
        facts.push(RunFact {
            date,
            distance_m: Some(run.distance_m),
        });
    }
    Ok(facts)
}

/// The distance run on a day, in metres.
///
/// `None` when nothing was run: a day with no run has no distance, and it
/// is never a zero.
pub fn distance_on(api: &Api, date: &str) -> ApiResult<Option<f64>> {
    let runs = api
        .store()
        .activities_for_date(date)
        .map_err(|error| api.store_error("the runs could not be read", &error))?;
    if runs.is_empty() {
        return Ok(None);
    }
    Ok(Some(runs.iter().map(|run| run.distance_m).sum()))
}

/// One session of the plan, as the answer carries it.
///
/// The label and the wording are the core's, the target and the actual are
/// the core's rendering of the numbers, and **the status is the core's
/// answer**: *done*, *not done* or *planned*.
pub fn session_of(
    session: &CoreSession,
    date: Date,
    source: CoreSource,
    current: Date,
    facts: &[RunFact],
    actual_m: Option<f64>,
    ran: bool,
) -> PlanSession {
    let label = stride_core::plan::session_label(session);
    let sentence = crate::routes::session_sentence(date, current, &label);
    PlanSession {
        date: date.to_iso(),
        session_type: kind_name(&session.kind),
        title: Some(session.label.clone()).filter(|title| !title.is_empty()),
        label,
        sentence,
        target: render::distance(session.target_distance_m),
        actual: render::distance(actual_m),
        status: status_of(source, session, facts).to_string(),
        ran,
        source: source_name(source).to_string(),
    }
}

/// The status of one session: *done*, *not done* or *planned*.
///
/// **The core's answer.** The engine is asked about the session on its own,
/// and it answers with the same rule it uses for the count — so a session's
/// status and the block's count can never disagree, and the rule lives in
/// one place.
///
/// A **baseline** never says "not done": an inferred week is not a
/// prescription, and that is the engine's rule and not this crate's.
pub fn status_of(source: CoreSource, session: &CoreSession, facts: &[RunFact]) -> &'static str {
    let one = CoreBlock {
        source,
        name: String::new(),
        sessions: vec![session.clone()],
    };
    let count = stride_core::plan::adherence_of(&one, facts);
    if count.done > 0 {
        "done"
    } else if count.not_done > 0 {
        "not_done"
    } else {
        "planned"
    }
}

/// The count, as the answer carries it.
pub fn adherence_of(adherence: CoreAdherence) -> Adherence {
    Adherence {
        done: adherence.done,
        planned: adherence.planned,
        not_done: adherence.not_done,
    }
}

/// The results of the block, as the answer carries them.
///
/// The intent, what happened and the difference between the two — the
/// difference as the core states it, and only when there is one to state.
pub fn results_of(api: &Api, plan: &ActivePlan, facts: &[RunFact]) -> ApiResult<Vec<PlanResult>> {
    let source = plan.source;
    let results: Vec<CoreSessionResult> = stride_core::plan::results_of(&plan.block, facts);
    // The ids belong to the store: a session that was written down has one,
    // and a session of the baseline does not.
    let stored: Vec<(String, i64)> = match plan.stored.as_ref() {
        Some(block) => api
            .store()
            .plan_sessions_for_block(block.id)
            .map_err(|error| api.store_error("the plan could not be read", &error))?
            .iter()
            .map(|session| (session.date.clone(), session.id))
            .collect(),
        None => Vec::new(),
    };
    let mut rendered: Vec<PlanResult> = Vec::with_capacity(results.len());
    for result in results.iter() {
        let date = result.date.to_iso();
        let actual_m = result.actual_distance_m;
        let session_id = stored
            .iter()
            .find(|(stored_date, _)| *stored_date == date)
            .map(|(_, id)| *id);
        rendered.push(PlanResult {
            session_id,
            date: date.clone(),
            session_type: kind_name(&result.kind),
            label: stride_core::plan::session_label(&CoreSession {
                date: result.date,
                kind: result.kind.clone(),
                target_distance_m: result.target_distance_m,
                label: result.label.clone(),
            }),
            status: status_of(
                source,
                &CoreSession {
                    date: result.date,
                    kind: result.kind.clone(),
                    target_distance_m: result.target_distance_m,
                    label: result.label.clone(),
                },
                facts,
            )
            .to_string(),
            target: render::distance(result.target_distance_m),
            actual: render::distance(actual_m),
            difference: stride_core::plan::distance_comparison(actual_m, result.target_distance_m),
        });
    }
    Ok(rendered)
}
