//! The plan engine: what the athlete intends to do, and what happened.
//!
//! A plan is a set of dated sessions with a source. The source is always
//! named: a baseline is inferred from the athlete's history, while a
//! prescribed or typed plan is an intent the athlete wrote down. The core
//! owns the baseline, the week, the adherence, the results and the distance
//! comparison, so the API and the app cannot answer those questions in two
//! different ways.
//!
//! The rules that shape this module:
//!
//! * a **baseline is not a prescription**. It never produces a "not done",
//!   never a shortfall and never a verdict; it says what the athlete
//!   usually does;
//! * "done" and "not done" are facts, not grades;
//! * a distance comparison appears only when the gap from the target is
//!   beyond ten per cent, and it is neutral in either direction;
//! * no value is `None`, never `0`: a session without a target has no target;
//! * everything time-dependent is an argument. There is no clock here.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::band;
use crate::date::Date;
use crate::strings;
use crate::units;

/// The fewest weeks of history a usual week may be inferred from.
///
/// Eight weeks is enough to see a weekly shape without turning one unusual
/// week into the athlete's habit. The threshold is the same one the old
/// plan-source oracle used.
pub const MIN_BASELINE_WEEKS: usize = 8;

/// The fewest usable runs a baseline needs.
///
/// A run with no distance cannot describe a usual distance, so it does not
/// count towards this minimum.
pub const MIN_BASELINE_RUNS: usize = 12;

/// The gap from the target above which a distance comparison is news.
///
/// Contract §5: the comparison is shown only when the gap exceeds ten per
/// cent. Exactly ten per cent is not beyond ten per cent.
pub const DISTANCE_COMPARISON_THRESHOLD: f64 = 0.10;

/// Where a plan came from. The app names the source; it never hides it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanSource {
    /// Inferred from the athlete's own history. A baseline, never a
    /// prescription.
    Baseline,
    /// Imported from a plan the athlete wrote down.
    Prescribed,
    /// Typed into the app.
    Typed,
}

impl PlanSource {
    /// The stable name of this source.
    pub fn as_str(self) -> &'static str {
        match self {
            PlanSource::Baseline => "baseline",
            PlanSource::Prescribed => "prescribed",
            PlanSource::Typed => "typed",
        }
    }

    /// The catalogue id of this source's banner.
    pub fn banner_id(self) -> &'static str {
        match self {
            PlanSource::Baseline => "statement.plan.source.baseline",
            PlanSource::Prescribed => "statement.plan.source.prescribed",
            PlanSource::Typed => "statement.plan.source.typed",
        }
    }

    /// The source banner: "your usual week, inferred from your history".
    pub fn banner(self) -> String {
        strings::render(self.banner_id(), &[]).unwrap_or_default()
    }
}

/// What a session is. This is the athlete's own vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    /// An easy run.
    Easy,
    /// A long run.
    Long,
    /// A quality session.
    Quality,
    /// A rest day.
    Rest,
    /// A kind the athlete named themselves.
    Other(String),
}

impl SessionKind {
    /// The word the plan uses for this kind.
    pub fn word(&self) -> String {
        let id = match self {
            SessionKind::Easy => "statement.plan.kind.easy",
            SessionKind::Long => "statement.plan.kind.long",
            SessionKind::Quality => "statement.plan.kind.quality",
            SessionKind::Rest => "statement.plan.kind.rest",
            SessionKind::Other(_) => "statement.plan.kind.other",
        };
        let value = match self {
            SessionKind::Other(name) if name.trim().is_empty() => "other".to_string(),
            SessionKind::Other(name) => name.clone(),
            _ => String::new(),
        };
        strings::render(id, &[("kind", &value)]).unwrap_or_default()
    }
}

/// One dated intent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanSession {
    /// The day the session belongs to.
    pub date: Date,
    /// What kind of session it is.
    pub kind: SessionKind,
    /// The target distance, in metres. `None` means the session has no
    /// distance target; it never means zero.
    pub target_distance_m: Option<f64>,
    /// The label as the app reads it: "10 km · easy".
    pub label: String,
}

impl PlanSession {
    /// A dated intent, with its label rendered by the core.
    pub fn new(date: Date, kind: SessionKind, target_distance_m: Option<f64>) -> Self {
        let target_distance_m =
            target_distance_m.filter(|distance| distance.is_finite() && *distance >= 0.0);
        let mut session = Self {
            date,
            kind,
            target_distance_m,
            label: String::new(),
        };
        session.label = session_label(&session);
        session
    }
}

/// A block of sessions: one source and one date range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanBlock {
    /// Where the block came from.
    pub source: PlanSource,
    /// The block's name.
    pub name: String,
    /// The dated sessions in the block, oldest first.
    pub sessions: Vec<PlanSession>,
}

impl PlanBlock {
    /// A block whose sessions are put in date order.
    pub fn new(source: PlanSource, name: String, mut sessions: Vec<PlanSession>) -> Self {
        sessions.sort_by_key(|session| session.date);
        Self {
            source,
            name,
            sessions,
        }
    }
}

/// What actually happened, as the plan engine needs to see it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunFact {
    /// The athlete-local day of the run.
    pub date: Date,
    /// The distance run, in metres. `None` means the distance is absent, not
    /// zero.
    pub distance_m: Option<f64>,
}

impl RunFact {
    /// One run fact.
    pub fn new(date: Date, distance_m: Option<f64>) -> Self {
        Self { date, distance_m }
    }
}

/// The plain status of a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// A run happened on the session's day.
    Done,
    /// The day has not arrived, or no prescription exists to call it not
    /// done.
    Planned,
    /// A prescribed session's day arrived without a run.
    NotDone,
}

/// The count of a plan's adherence. It is a count, never a score.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adherence {
    /// How many sessions the plan contains.
    pub planned: usize,
    /// How many of them have a run on their day.
    pub done: usize,
    /// How many prescribed sessions have passed without a run.
    ///
    /// This is always zero for a baseline: a baseline is not a prescription.
    pub not_done: usize,
}

impl Adherence {
    /// The adherence line: "7 of 8 sessions run".
    ///
    /// `None` when there is no plan. A zero-plan count never becomes "0 of
    /// 0 sessions run".
    pub fn line(&self) -> Option<String> {
        if self.planned == 0 {
            return None;
        }
        let done = self.done.to_string();
        let planned = self.planned.to_string();
        strings::render(
            "statement.plan.adherence",
            &[("done", &done), ("planned", &planned)],
        )
    }

    /// The baseline line: "3 sessions in your usual week".
    ///
    /// It states what happened and never implies a shortfall.
    pub fn baseline_line(&self) -> Option<String> {
        if self.planned == 0 {
            return None;
        }
        if self.done == 1 {
            return strings::render("statement.plan.adherence.baseline.one", &[]);
        }
        let done = self.done.to_string();
        strings::render("statement.plan.adherence.baseline", &[("done", &done)])
    }

    /// The line appropriate to the plan's source.
    pub fn line_for(&self, source: PlanSource) -> Option<String> {
        match source {
            PlanSource::Baseline => self.baseline_line(),
            PlanSource::Prescribed | PlanSource::Typed => self.line(),
        }
    }
}

/// One session's intent, what happened, and the difference between them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionResult {
    /// The intent.
    pub session: PlanSession,
    /// Whether a run happened on the session's day.
    pub ran: bool,
    /// The distance actually run, in metres, when there is one.
    pub actual: Option<f64>,
    /// The neutral distance difference, when it is beyond ten per cent.
    pub difference: Option<String>,
    /// The result line: "8.4 km of a 10 km target".
    pub line: Option<String>,
    /// The plain status of the session.
    pub status: SessionStatus,
}

impl SessionResult {
    /// The intent.
    pub fn intent(&self) -> &PlanSession {
        &self.session
    }

    /// The distance actually run, in metres.
    pub fn actual_distance_m(&self) -> Option<f64> {
        self.actual
    }
}

/// The label of a session: "10 km · easy".
///
/// A session with no target says so: "easy · no distance target". The
/// target is never rendered as zero.
pub fn session_label(session: &PlanSession) -> String {
    let kind = session.kind.word();
    let target = session
        .target_distance_m
        .filter(|distance| distance.is_finite() && *distance >= 0.0)
        .and_then(|distance| units::render_distance(Some(distance)));
    match target {
        Some(distance) => strings::render(
            "statement.plan.session.label",
            &[("distance", &distance), ("kind", &kind)],
        )
        .unwrap_or_default(),
        None => strings::render("statement.plan.session.no_target", &[("kind", &kind)])
            .unwrap_or_default(),
    }
}

/// The source banner for a plan.
pub fn source_banner(source: PlanSource) -> String {
    source.banner()
}

/// The name of a weekday: "Monday".
pub fn day_label(day: Date) -> String {
    let id = match weekday_index(day) {
        0 => "statement.plan.day.monday",
        1 => "statement.plan.day.tuesday",
        2 => "statement.plan.day.wednesday",
        3 => "statement.plan.day.thursday",
        4 => "statement.plan.day.friday",
        5 => "statement.plan.day.saturday",
        _ => "statement.plan.day.sunday",
    };
    strings::render(id, &[]).unwrap_or_default()
}

/// The Monday of the calendar week containing a day.
pub fn week_start(day: Date) -> Date {
    day.add_days(-weekday_index(day))
}

/// The sessions in the calendar week containing a day, oldest first.
///
/// The week is Monday through Sunday, the same week the API's plan routes
/// use. A day without an intent has no session: absence is not converted
/// into a rest day.
pub fn week_of(block: &PlanBlock, day: Date) -> Vec<PlanSession> {
    let start = week_start(day);
    let end = start.add_days(6);
    let mut sessions: Vec<PlanSession> = block
        .sessions
        .iter()
        .filter(|session| session.date >= start && session.date <= end)
        .cloned()
        .collect();
    sessions.sort_by_key(|session| session.date);
    sessions
}

/// The session on a day, when the plan has one.
pub fn today_of(block: &PlanBlock, day: Date) -> Option<&PlanSession> {
    block.sessions.iter().find(|session| session.date == day)
}

/// The adherence count for a block.
///
/// A baseline never has a "not done": it describes what the athlete
/// usually does, not what they were told to do. For a prescribed or typed
/// plan, every session without a matching run is counted as not done.
pub fn adherence_of(block: &PlanBlock, actual: &[RunFact]) -> Adherence {
    let planned = block.sessions.len();
    let done = block
        .sessions
        .iter()
        .filter(|session| actual.iter().any(|fact| fact.date == session.date))
        .count();
    let not_done = match block.source {
        PlanSource::Baseline => 0,
        PlanSource::Prescribed | PlanSource::Typed => planned.saturating_sub(done),
    };
    Adherence {
        planned,
        done,
        not_done,
    }
}

/// The adherence count as of a day.
///
/// Only a prescribed session whose day has passed can be "not done". A
/// future session is still planned, and a baseline never produces a
/// shortfall.
pub fn adherence_of_at(block: &PlanBlock, actual: &[RunFact], day: Date) -> Adherence {
    let planned = block.sessions.len();
    let done = block
        .sessions
        .iter()
        .filter(|session| actual.iter().any(|fact| fact.date == session.date))
        .count();
    let not_done = match block.source {
        PlanSource::Baseline => 0,
        PlanSource::Prescribed | PlanSource::Typed => block
            .sessions
            .iter()
            .filter(|session| session.date < day)
            .filter(|session| !actual.iter().any(|fact| fact.date == session.date))
            .count(),
    };
    Adherence {
        planned,
        done,
        not_done,
    }
}

/// The result of every session in a block.
///
/// The result line states the actual and the target neutrally. The separate
/// difference is present only when the gap is beyond ten per cent.
pub fn results_of(block: &PlanBlock, actual: &[RunFact]) -> Vec<SessionResult> {
    block
        .sessions
        .iter()
        .map(|session| session_result(block.source, session, actual, None))
        .collect()
}

/// The results of a block as of a day.
///
/// A prescribed session is "not done" only after its day has passed. A
/// baseline session without a run stays "planned": there was no prescription
/// to miss.
pub fn results_of_at(block: &PlanBlock, actual: &[RunFact], day: Date) -> Vec<SessionResult> {
    block
        .sessions
        .iter()
        .map(|session| session_result(block.source, session, actual, Some(day)))
        .collect()
}

/// One session's result.
fn session_result(
    source: PlanSource,
    session: &PlanSession,
    actual: &[RunFact],
    day: Option<Date>,
) -> SessionResult {
    let facts: Vec<&RunFact> = actual
        .iter()
        .filter(|fact| fact.date == session.date)
        .collect();
    let ran = !facts.is_empty();
    let distances: Vec<f64> = facts
        .iter()
        .filter_map(|fact| fact.distance_m)
        .filter(|distance| distance.is_finite() && *distance >= 0.0)
        .collect();
    let actual_distance_m = if distances.is_empty() {
        None
    } else {
        Some(distances.iter().sum())
    };
    let difference = distance_comparison(actual_distance_m, session.target_distance_m);
    let line = result_line(ran, actual_distance_m, session.target_distance_m);
    let status = if ran {
        SessionStatus::Done
    } else {
        let has_passed = day.map(|day| session.date < day).unwrap_or(true);
        match source {
            // A baseline is not a prescription, so it never says "not done".
            PlanSource::Baseline => SessionStatus::Planned,
            PlanSource::Prescribed | PlanSource::Typed => {
                if has_passed {
                    SessionStatus::NotDone
                } else {
                    SessionStatus::Planned
                }
            }
        }
    };
    SessionResult {
        session: session.clone(),
        ran,
        actual: actual_distance_m,
        difference,
        line,
        status,
    }
}

/// The result line for one session.
fn result_line(ran: bool, actual_m: Option<f64>, target_m: Option<f64>) -> Option<String> {
    let actual = actual_m.and_then(|distance| units::render_distance(Some(distance)));
    let target = target_m.and_then(|distance| units::render_distance(Some(distance)));
    match (ran, actual, target) {
        (true, Some(actual), Some(target)) => strings::render(
            "statement.plan.result.target",
            &[("actual", &actual), ("target", &target)],
        ),
        (true, Some(actual), None) => {
            strings::render("statement.plan.result.no_target", &[("actual", &actual)])
        }
        (false, None, Some(target)) => {
            strings::render("statement.plan.result.no_run", &[("target", &target)])
        }
        (false, None, None) => strings::render("statement.plan.result.empty", &[]),
        (true, None, Some(target)) => strings::render(
            "statement.plan.result.no_distance_with_target",
            &[("target", &target)],
        ),
        (true, None, None) => strings::render("statement.plan.result.no_distance", &[]),
        // A run with a distance and no target was handled above. This arm
        // keeps the match exhaustive if a future value is added.
        (false, Some(_), _) => None,
    }
}

/// The distance comparison, when there is one.
///
/// The comparison is neutral in both directions: "2.4 km more than the 10 km
/// target" and "2 km less than the 10 km target" say the same kind of thing.
/// It is absent when either value is absent, and when the gap is within ten
/// per cent of the target.
pub fn distance_comparison(actual_m: Option<f64>, target_m: Option<f64>) -> Option<String> {
    let actual = actual_m?;
    let target = target_m?;
    if !actual.is_finite() || !target.is_finite() || actual < 0.0 || target <= 0.0 {
        return None;
    }
    let gap = actual - target;
    if gap.abs() / target <= DISTANCE_COMPARISON_THRESHOLD {
        return None;
    }
    let gap_text = format!("{} km", units::format_distance_km_short(gap.abs()));
    let target_text = format!("{} km", units::format_distance_km_short(target));
    let id = if gap > 0.0 {
        "statement.distance.more"
    } else {
        "statement.distance.less"
    };
    strings::render(id, &[("gap", &gap_text), ("target", &target_text)])
}

/// The distance comparison with the actual distance in front of it:
/// "12.4 km · 2.4 km more than the 10 km target".
pub fn distance_comparison_line(actual_m: Option<f64>, target_m: Option<f64>) -> Option<String> {
    let comparison = distance_comparison(actual_m, target_m)?;
    let actual = actual_m?;
    let distance = format!("{} km", units::format_distance_km_short(actual));
    strings::render(
        "statement.distance.line",
        &[("distance", &distance), ("comparison", &comparison)],
    )
}

/// The line shown when history is too thin to infer a usual week.
pub fn baseline_not_enough_history() -> String {
    strings::render("statement.plan.baseline.not_enough_history", &[]).unwrap_or_default()
}

/// Infer the athlete's usual week from their history.
///
/// The engine reads the last `weeks` calendar weeks before the most recent
/// usable run. It needs at least eight weeks and twelve usable runs; a
/// thinner history yields `None` rather than an invented plan.
///
/// From the usable history it reads:
///
/// * the typical number of sessions in a week (the median weekly count,
///   including weeks with no run);
/// * the typical distance for each weekday that recurs (the median distance
///   on that weekday);
/// * the typical kind: the longest recurring run is long, the second
///   longest is quality, and the rest are easy.
///
/// The block's name states how many weeks were actually used. The sessions
/// are dated in the week containing the most recent usable run, so the
/// result is deterministic and needs no clock.
pub fn baseline_from_history(runs: &[RunFact], weeks: usize) -> Option<PlanBlock> {
    let anchor = runs
        .iter()
        .filter_map(|run| usable_distance(run).map(|distance| (run.date, distance)))
        .map(|(date, _)| date)
        .max()?;
    baseline_from_history_at(runs, weeks, anchor)
}

/// Infer the athlete's usual week for a particular week.
///
/// This is the time-injected form of [`baseline_from_history`]. The history
/// is read from the requested number of weeks before `day`; the resulting
/// sessions are dated in the calendar week containing `day`.
pub fn baseline_from_history_at(runs: &[RunFact], weeks: usize, day: Date) -> Option<PlanBlock> {
    if weeks < MIN_BASELINE_WEEKS {
        return None;
    }
    let requested_weeks = i64::try_from(weeks).unwrap_or(i64::MAX);
    let anchor_week = week_start(day);
    let window_start = anchor_week.add_days(-(requested_weeks.saturating_sub(1)) * 7);
    let usable: Vec<(Date, f64)> = runs
        .iter()
        .filter(|run| run.date >= window_start && run.date <= day)
        .filter_map(|run| usable_distance(run).map(|distance| (run.date, distance)))
        .collect();
    if usable.len() < MIN_BASELINE_RUNS {
        return None;
    }
    let first_week = usable.iter().map(|(date, _)| week_start(*date)).min()?;
    let spanned_weeks = anchor_week.days_since(first_week) / 7 + 1;
    if spanned_weeks < MIN_BASELINE_WEEKS as i64 {
        return None;
    }
    let weeks_used = usize::try_from(spanned_weeks).unwrap_or(weeks).min(weeks);

    // The weekly session counts include empty weeks between the first and
    // last observed week: an athlete who usually runs three times a week
    // should not have that usual week changed by one quiet week.
    let week_count = usize::try_from(spanned_weeks).unwrap_or(0);
    let mut weekly_counts = vec![0usize; week_count];
    for (date, _) in &usable {
        let index = date.days_since(first_week) / 7;
        if let Ok(index) = usize::try_from(index) {
            if let Some(count) = weekly_counts.get_mut(index) {
                *count += 1;
            }
        }
    }
    let typical_sessions = units::round_half_even(
        band::median(
            &weekly_counts
                .iter()
                .map(|count| *count as f64)
                .collect::<Vec<_>>(),
        )
        .unwrap_or(0.0),
        0,
    ) as usize;
    if typical_sessions == 0 {
        return None;
    }
    let typical_sessions = typical_sessions.min(7);

    // Group the usable runs by weekday. A weekday that appears in more
    // distinct weeks is more a part of the usual week than one that appears
    // often in a single week.
    let mut by_weekday: [Vec<(usize, f64)>; 7] = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    for (date, distance) in &usable {
        let weekday = usize::try_from(weekday_index(*date)).unwrap_or(0);
        let week = date.days_since(first_week) / 7;
        if let Ok(week) = usize::try_from(week) {
            by_weekday[weekday].push((week, *distance));
        }
    }
    let mut candidates: Vec<WeekdayCandidate> = Vec::new();
    for (weekday, entries) in by_weekday.iter().enumerate() {
        if entries.is_empty() {
            continue;
        }
        let mut weeks_seen: Vec<usize> = entries.iter().map(|(week, _)| *week).collect();
        weeks_seen.sort_unstable();
        weeks_seen.dedup();
        let distances: Vec<f64> = entries.iter().map(|(_, distance)| *distance).collect();
        let typical_distance = band::median(&distances)?;
        candidates.push(WeekdayCandidate {
            weekday,
            weeks_seen: weeks_seen.len(),
            entries: entries.len(),
            typical_distance,
        });
    }
    candidates.sort_by(|left, right| {
        right
            .weeks_seen
            .cmp(&left.weeks_seen)
            .then_with(|| right.entries.cmp(&left.entries))
            .then_with(|| right.typical_distance.total_cmp(&left.typical_distance))
            .then_with(|| left.weekday.cmp(&right.weekday))
    });
    candidates.truncate(typical_sessions);

    // The longest recurring distance is the long run; the second longest is
    // the quality run; the rest are easy runs.
    let mut by_distance = candidates.clone();
    by_distance.sort_by(|left, right| {
        left.typical_distance
            .total_cmp(&right.typical_distance)
            .then_with(|| left.weekday.cmp(&right.weekday))
    });
    let mut kinds: BTreeMap<usize, SessionKind> = BTreeMap::new();
    for (index, candidate) in by_distance.iter().enumerate() {
        let kind = if by_distance.len() == 1 {
            SessionKind::Easy
        } else if index + 1 == by_distance.len() {
            SessionKind::Long
        } else if index + 2 == by_distance.len() {
            SessionKind::Quality
        } else {
            SessionKind::Easy
        };
        kinds.insert(candidate.weekday, kind);
    }

    let mut sessions = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let date = anchor_week.add_days(candidate.weekday as i64);
        let kind = kinds
            .get(&candidate.weekday)
            .cloned()
            .unwrap_or(SessionKind::Easy);
        sessions.push(PlanSession::new(
            date,
            kind,
            Some(candidate.typical_distance),
        ));
    }
    sessions.sort_by_key(|session| session.date);

    let weeks_text = weeks_used.to_string();
    let name = strings::render(
        "statement.plan.baseline.description",
        &[("weeks", &weeks_text)],
    )
    .unwrap_or_default();
    Some(PlanBlock::new(PlanSource::Baseline, name, sessions))
}

/// A weekday's recurrence and typical distance.
#[derive(Clone, Copy, Debug)]
struct WeekdayCandidate {
    /// Monday is zero.
    weekday: usize,
    /// How many distinct weeks contain this weekday.
    weeks_seen: usize,
    /// How many runs occurred on this weekday.
    entries: usize,
    /// The median distance on this weekday.
    typical_distance: f64,
}

/// The usable distance of a run: a finite, positive distance.
///
/// A run with no distance is a fact, but it cannot describe a usual
/// distance and therefore is not part of the distance inference.
fn usable_distance(run: &RunFact) -> Option<f64> {
    run.distance_m
        .filter(|distance| distance.is_finite() && *distance > 0.0)
}

/// The weekday index, Monday is zero.
///
/// 1970-01-01 was a Thursday, so three days are added before the epoch day
/// is reduced into the Monday-first week.
fn weekday_index(day: Date) -> i64 {
    day.epoch_day().saturating_add(3).rem_euclid(7)
}
