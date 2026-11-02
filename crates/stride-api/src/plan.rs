//! The seam to the core's plan engine.
//!
//! **The engine has one owner: `stride-core`.** This module is the adapter
//! between the store's rows and the engine's domain types: it reads the
//! store, hands the core the types the core works on, and reads the core's
//! answers back.
//!
//! It decides nothing. There is no rule about what a plan is in here, no
//! wording of this crate's own, and no second implementation of the
//! engine's answers — the baseline, the session of a day, the sessions of
//! the week, the count, the status of a session and the results all come
//! from the core, and this module only carries them.
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
    SessionStatus,
};
use stride_store::model::plan_source;
use stride_store::{PlanBlock as StoredBlock, PlanSession as StoredSession};

use crate::dto::{Adherence, Counts, PlanResult, PlanSession};
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
    /// The store's rows: the session titles and ids, matched by day.
    pub rows: Vec<StoredSession>,
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
        self.source.as_str()
    }

    /// The store's row for a session, when the plan was written down.
    ///
    /// The row carries the title the athlete wrote and the session's id.
    pub fn row_of(&self, session: &CoreSession) -> Option<&StoredSession> {
        self.rows.iter().find(|row| {
            row.date == session.date.to_iso() && kind_of(&row.session_type) == session.kind
        })
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
            .active_plan_block(source.as_str())
            .map_err(|error| api.store_error("the plan could not be read", &error))?;
        if let Some(stored) = stored {
            return Ok(Some(from_stored(api, &stored)?));
        }
    }

    // No plan of the athlete's own: the baseline, read from the athlete's
    // own history. The engine says when the history is too thin, and it
    // says nothing rather than inventing a week.
    let weeks = api.config().baseline_weeks;
    let today = crate::routes::today(api)?;
    let from = today.add_days(-(weeks as i64 * 7)).to_iso();
    let facts = run_facts(api, &from, &today.to_iso())?;
    let block = match stride_core::plan::baseline_from_history_at(&facts, weeks, today) {
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
        rows: Vec::new(),
        block,
    }))
}

/// The plan in force, or the empty one when there is none.
///
/// The routes that have to answer with something use this one: a plan that
/// is not there is a fact the app shows, and the answer says it.
pub fn active_or_empty(api: &Api) -> ApiResult<ActivePlan> {
    match active(api)? {
        Some(plan) => Ok(plan),
        None => Ok(ActivePlan {
            block: CoreBlock::new(CoreSource::Baseline, "No plan".to_string(), Vec::new()),
            stored: None,
            rows: Vec::new(),
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
    let rows = api
        .store()
        .plan_sessions_for_block(stored.id)
        .map_err(|error| api.store_error("the plan could not be read", &error))?;
    let sessions: Vec<CoreSession> = rows.iter().filter_map(core_session).collect();
    Ok(ActivePlan {
        block: CoreBlock::new(
            source_of(stored.source.as_str()),
            stored.name.clone(),
            sessions,
        ),
        stored: Some(stored.clone()),
        rows,
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
    let mut core = CoreSession::new(
        date,
        kind_of(&session.session_type),
        session.target_distance_m,
    );
    // The athlete's own title rides along in the label, and the core's
    // rendering of the session is what the app reads.
    if let Some(title) = session.title.as_ref() {
        if !title.trim().is_empty() {
            core.label = title.clone();
        }
    }
    Some(core)
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

/// The status of a session, as the app reads it.
///
/// The core's own statuses, spelled the app's way: `done`, `planned`,
/// `not_done`.
pub fn status_name(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Done => "done",
        SessionStatus::Planned => "planned",
        SessionStatus::NotDone => "not_done",
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
        facts.push(RunFact::new(date, Some(run.distance_m)));
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
    Ok(Some(render::total_of(
        runs.iter().map(|run| run.distance_m),
    )))
}

/// One session of the plan, as the answer carries it.
///
/// The sentence is the core's wording, the target and the actual are the
/// core's rendering of the numbers, and the status is the core's answer.
pub fn session_of(plan: &ActivePlan, result: &CoreSessionResult, today: Date) -> PlanSession {
    let date = result.session.date;
    let sentence = crate::routes::session_sentence(date, today, &result.session.label);
    PlanSession {
        date: date.to_iso(),
        session_type: result.session.kind.word(),
        title: plan
            .row_of(&result.session)
            .and_then(|row| row.title.clone())
            .filter(|title| !title.trim().is_empty()),
        sentence,
        target: render::distance(result.session.target_distance_m),
        actual: render::distance(result.actual),
        status: status_name(result.status).to_string(),
        source: plan.source_name().to_string(),
    }
}

/// The count, as the answer carries it.
///
/// The core's count: how many were run and how many the plan holds.
pub fn adherence_of(adherence: CoreAdherence) -> Adherence {
    Adherence {
        done: adherence.done,
        planned: adherence.planned,
    }
}

/// The count by status: done, planned and not done.
///
/// The three are the three statuses, so they add up to the number of
/// sessions in the plan. The core's count is of the same three facts: its
/// `planned` is the number of sessions the plan holds, and the ones still
/// to come are what is left after the done and the not done.
pub fn counts_of(adherence: CoreAdherence) -> Counts {
    Counts {
        done: adherence.done,
        planned: adherence
            .planned
            .saturating_sub(adherence.done)
            .saturating_sub(adherence.not_done),
        not_done: adherence.not_done,
    }
}

/// The results of the block, as the answer carries them.
///
/// The intent, what happened and the difference between the two — the
/// difference as the core states it, and only when there is one to state.
pub fn results_of(plan: &ActivePlan, facts: &[RunFact], today: Date) -> Vec<PlanResult> {
    let results: Vec<CoreSessionResult> =
        stride_core::plan::results_of_at(&plan.block, facts, today);
    let mut rendered: Vec<PlanResult> = Vec::with_capacity(results.len());
    for result in results.iter() {
        let date = result.session.date.to_iso();
        let session_id = plan
            .rows
            .iter()
            .find(|row| row.date == date && kind_of(&row.session_type) == result.session.kind)
            .map(|row| row.id);
        rendered.push(PlanResult {
            session_id,
            date: date.clone(),
            session_type: result.session.kind.word(),
            label: result.session.label.clone(),
            status: status_name(result.status).to_string(),
            target: render::distance(result.session.target_distance_m),
            actual: render::distance(result.actual),
            difference: result.difference.clone(),
        });
    }
    rendered
}
