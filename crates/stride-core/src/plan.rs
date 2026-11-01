//! The plan engine: the plan the athlete has, and what it says.
//!
//! **This file is a stand-in.** The plan engine has one owner — this crate —
//! and the engine itself is written in the core's own pull request. What is
//! here is the **interface** the API builds to, written out so that the API
//! can be built, run and tested before that file lands. The core's own file
//! replaces this one wholesale; the API calls only the interface below.
//!
//! ## What the engine decides
//!
//! * **the baseline** — what the athlete's own history says their week
//!   looks like. A baseline is a *source*, never a prescription: it never
//!   yields "not done".
//! * **the session of a day** and **the sessions of a week**.
//! * **the adherence** — a count: planned, done, not done. Never a score.
//! * **the results** — what was planned and what happened.
//! * **the wording** — the label of a session, and the difference between
//!   what was planned and what was run.
//!
//! ## The rules
//!
//! * Pure by contract: no I/O, no clock, no randomness. The day a session
//!   belongs to and the runs that happened are injected by the caller.
//! * **No value is `None`, never `0`.** A session with no target distance
//!   has no target.
//! * **No verdict.** A session is *done*, *not done* or *planned*. A
//!   baseline is never "not done": an inferred week is not a prescription.
//! * **A count is a count.** The adherence is three numbers, and there is no
//!   percentage anywhere in this module.

use crate::date::Date;
use crate::statement;
use crate::types::Distance;
use crate::units;

/// Where a plan came from.
///
/// The three sources are the whole set: a plan the athlete wrote or
/// imported, and the week the app inferred from the athlete's own history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlanSource {
    /// Inferred from the athlete's own history. A baseline, never a
    /// prescription.
    Baseline,
    /// Written down by the athlete — in Garmin Connect, or imported.
    Prescribed,
    /// Typed into the app.
    Typed,
}

impl PlanSource {
    /// The three sources, in the order the app prefers them.
    ///
    /// The athlete's own plan comes first: a plan the athlete wrote or
    /// imported is the plan in force. A baseline is what the app inferred,
    /// and it is the last thing the app offers.
    pub const ALL: [PlanSource; 3] = [
        PlanSource::Prescribed,
        PlanSource::Typed,
        PlanSource::Baseline,
    ];

    /// The source's name, as it travels.
    pub fn name(self) -> &'static str {
        match self {
            PlanSource::Baseline => "baseline",
            PlanSource::Prescribed => "prescribed",
            PlanSource::Typed => "typed",
        }
    }

    /// The source with this name.
    pub fn from_name(name: &str) -> Option<PlanSource> {
        match name {
            "baseline" => Some(PlanSource::Baseline),
            "prescribed" => Some(PlanSource::Prescribed),
            "typed" => Some(PlanSource::Typed),
            _ => None,
        }
    }
}

/// The kind of a planned session.
///
/// The kinds the engine names are the ones every plan uses. Everything else
/// is [`SessionKind::Other`], which carries the athlete's own word for it —
/// so a session the engine does not recognise keeps its name instead of
/// being given one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionKind {
    /// An easy run.
    Easy,
    /// A long run.
    Long,
    /// A session of quality: a tempo, an interval set, a race.
    Quality,
    /// A day with no run planned.
    Rest,
    /// A session the engine does not name: the athlete's own word for it.
    Other(String),
}

impl SessionKind {
    /// The kind's name, as it travels.
    pub fn name(&self) -> &str {
        match self {
            SessionKind::Easy => "easy",
            SessionKind::Long => "long",
            SessionKind::Quality => "quality",
            SessionKind::Rest => "rest",
            SessionKind::Other(name) => name.as_str(),
        }
    }

    /// The kind with this name.
    ///
    /// A name the engine does not name is [`SessionKind::Other`]: it keeps
    /// its own word, and it is never given one of the named kinds.
    pub fn from_name(name: &str) -> SessionKind {
        match name.trim().to_ascii_lowercase().as_str() {
            "easy" => SessionKind::Easy,
            "long" => SessionKind::Long,
            "quality" => SessionKind::Quality,
            "rest" => SessionKind::Rest,
            other => SessionKind::Other(other.to_string()),
        }
    }
}

/// One planned session.
///
/// The session is the **intent**: the day, the kind, and the distance the
/// athlete wrote down. Whether it was done, not done or is still planned is
/// **not** here — it is computed on read, from the date, the runs of that
/// day and the block's source.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanSession {
    /// The day of the session.
    pub date: Date,
    /// The kind of session.
    pub kind: SessionKind,
    /// The distance that was planned, in metres. `None` when there is none.
    pub target_distance_m: Option<f64>,
    /// The session's own title, in the athlete's words.
    pub label: String,
}

/// One block of a plan: a set of sessions, and where they came from.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanBlock {
    /// Where the block came from.
    pub source: PlanSource,
    /// The block's name.
    pub name: String,
    /// The block's sessions.
    pub sessions: Vec<PlanSession>,
}

/// One thing the athlete did.
///
/// The fact is a day and, when there was one, a distance. A run with no
/// distance is still a run: the distance is `None`, not `0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunFact {
    /// The day of the run.
    pub date: Date,
    /// The distance of the run, in metres. `None` when there was none.
    pub distance_m: Option<f64>,
}

/// How many of the planned sessions were run.
///
/// A count, never a score: there is no percentage here, and there is no
/// grade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adherence {
    /// How many sessions the block planned.
    pub planned: usize,
    /// How many of them were run.
    pub done: usize,
    /// How many of them were not.
    ///
    /// A **baseline** never counts here: an inferred week is not a
    /// prescription, and it cannot be "not done".
    pub not_done: usize,
}

/// What one planned session turned into.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionResult {
    /// The day of the session.
    pub date: Date,
    /// The kind of session.
    pub kind: SessionKind,
    /// The session's own title.
    pub label: String,
    /// The distance that was planned, in metres. `None` when there was none.
    pub target_distance_m: Option<f64>,
    /// The distance that was run, in metres. `None` when nothing was run.
    pub actual_distance_m: Option<f64>,
    /// The difference between the two, in metres. `None` when there is no
    /// difference to state.
    pub difference_m: Option<f64>,
}

/// The baseline: the week the athlete's own history describes.
///
/// The baseline is built from the last `weeks` weeks of the athlete's runs.
/// For every weekday the athlete ran on, the median distance of those runs
/// is the session for that weekday — so the baseline is the athlete's own
/// week, described back to them.
///
/// `None` when there is not enough history to describe a week: a baseline
/// drawn from one run is a point, and the app would be inventing a week.
pub fn baseline_from_history(runs: &[RunFact], weeks: usize) -> Option<PlanBlock> {
    if weeks == 0 {
        return None;
    }
    let newest = runs.iter().map(|run| run.date).max()?;
    let first_day = newest.add_days(-(weeks as i64 * 7 - 1));

    // Every run of the window, grouped by the weekday it fell on.
    let mut by_weekday: Vec<Vec<Option<f64>>> = vec![Vec::new(); 7];
    for run in runs.iter() {
        if run.date < first_day || run.date > newest {
            continue;
        }
        let weekday = weekday_of(run.date);
        by_weekday[weekday].push(run.distance_m);
    }

    let mut sessions: Vec<PlanSession> = Vec::new();
    for (weekday, distances) in by_weekday.iter().enumerate() {
        if distances.is_empty() {
            continue;
        }
        // The median of the distances that were measured. A run with no
        // distance is a run, and it counts as one: it is left out of the
        // median, but it still makes the weekday a weekday the athlete ran
        // on.
        let measured: Vec<f64> = distances.iter().filter_map(|value| *value).collect();
        let target = if measured.is_empty() {
            None
        } else {
            crate::band::median(&measured)
        };
        // The session is dated on the most recent run of that weekday, so
        // the weekday is the session's own.
        let date = runs
            .iter()
            .filter(|run| run.date <= newest && weekday_of(run.date) == weekday)
            .map(|run| run.date)
            .max()
            .unwrap_or(newest);
        sessions.push(PlanSession {
            date,
            kind: SessionKind::Easy,
            target_distance_m: target,
            label: String::new(),
        });
    }

    if sessions.is_empty() {
        return None;
    }
    Some(PlanBlock {
        source: PlanSource::Baseline,
        name: "Your usual week".to_string(),
        sessions,
    })
}

/// The session of a day.
///
/// For a block with dates, the session **on** that day. For a baseline — a
/// pattern rather than a set of days — the session of that weekday, which
/// is what the pattern means.
pub fn today_of(block: &PlanBlock, day: Date) -> Option<&PlanSession> {
    match block.source {
        PlanSource::Baseline => block
            .sessions
            .iter()
            .find(|session| weekday_of(session.date) == weekday_of(day)),
        _ => block.sessions.iter().find(|session| session.date == day),
    }
}

/// The sessions of the week a day is in.
///
/// The seven days of that week, in the order they fall. For a baseline the
/// sessions are dated into the week asked for, because a baseline is a
/// pattern and a pattern is asked about a week.
pub fn week_of(block: &PlanBlock, day: Date) -> Vec<PlanSession> {
    let start = week_start(day);
    let end = start.add_days(6);
    match block.source {
        PlanSource::Baseline => {
            let mut sessions: Vec<PlanSession> = block
                .sessions
                .iter()
                .map(|session| {
                    let mut moved = session.clone();
                    moved.date = start.add_days(weekday_of(moved.date) as i64);
                    moved
                })
                .collect();
            sessions.sort_by_key(|session| session.date);
            sessions
        }
        _ => {
            let mut sessions: Vec<PlanSession> = block
                .sessions
                .iter()
                .filter(|session| session.date >= start && session.date <= end)
                .cloned()
                .collect();
            sessions.sort_by_key(|session| session.date);
            sessions
        }
    }
}

/// How many of the planned sessions were run.
///
/// The count is of the block's own sessions, and the runs are matched by
/// their day. A **baseline** never yields "not done": an inferred week is
/// not a prescription.
pub fn adherence_of(block: &PlanBlock, actual: &[RunFact]) -> Adherence {
    let mut adherence = Adherence {
        planned: block.sessions.len(),
        done: 0,
        not_done: 0,
    };
    let newest = actual.iter().map(|run| run.date).max();
    for session in block.sessions.iter() {
        if ran_on(actual, session.date) {
            adherence.done += 1;
            continue;
        }
        // A session that is still to come is planned, not missed — and a
        // baseline is never "not done" at all.
        let is_past = match newest {
            Some(newest) => session.date < newest,
            None => false,
        };
        if is_past && block.source != PlanSource::Baseline {
            adherence.not_done += 1;
        }
    }
    adherence
}

/// What every planned session turned into.
///
/// The intent, what happened, and the difference between the two — as
/// numbers. The wording is [`distance_comparison`]'s business.
pub fn results_of(block: &PlanBlock, actual: &[RunFact]) -> Vec<SessionResult> {
    let mut results: Vec<SessionResult> = Vec::new();
    for session in block.sessions.iter() {
        let actual_distance_m = distance_on(actual, session.date);
        let difference_m = match (session.target_distance_m, actual_distance_m) {
            (Some(target), Some(actual)) => Some(actual - target),
            _ => None,
        };
        results.push(SessionResult {
            date: session.date,
            kind: session.kind.clone(),
            label: session.label.clone(),
            target_distance_m: session.target_distance_m,
            actual_distance_m,
            difference_m,
        });
    }
    results.sort_by_key(|result| result.date);
    results
}

/// The difference between what was planned and what was run.
///
/// The core renders it: "2.4 km more than the 10 km target", and it says it
/// only when the gap from the target is beyond ten per cent. Within ten per
/// cent it is absent — 10.2 km against a 10 km target is not news, and
/// neither more nor less is good or bad.
pub fn distance_comparison(actual_m: Option<f64>, target_m: Option<f64>) -> Option<String> {
    let actual = Distance::from_metres(actual_m?)?;
    let target = Distance::from_metres(target_m?)?;
    statement::distance_comparison(actual, target)
}

/// The label of a session — "10 km · easy".
///
/// The distance is the core's own rendering of it, and the kind is the
/// session's own word. A session with no distance is labelled by its kind
/// alone.
pub fn session_label(session: &PlanSession) -> String {
    let kind = session.kind.name();
    match units::render_distance(session.target_distance_m) {
        Some(distance) => format!("{distance} · {kind}"),
        None => kind.to_string(),
    }
}

/// Whether the athlete ran on a day.
fn ran_on(actual: &[RunFact], day: Date) -> bool {
    actual.iter().any(|run| run.date == day)
}

/// The distance run on a day, in metres.
///
/// The sum of the runs of that day. A day with no run has no distance, and
/// a day whose runs had no distance has none either: `None`, never `0`.
fn distance_on(actual: &[RunFact], day: Date) -> Option<f64> {
    let distances: Vec<f64> = actual
        .iter()
        .filter(|run| run.date == day)
        .filter_map(|run| run.distance_m)
        .collect();
    if distances.is_empty() {
        return None;
    }
    Some(distances.iter().sum())
}

/// The weekday of a day, from Monday (zero) to Sunday (six).
fn weekday_of(day: Date) -> usize {
    // 1970-01-01 was a Thursday, so the epoch day counts from there.
    let epoch_day = day.epoch_day();
    let weekday = (epoch_day + 3).rem_euclid(7);
    weekday as usize
}

/// The first day of the week a day is in: the Monday.
fn week_start(day: Date) -> Date {
    let offset = weekday_of(day) as i64;
    day.add_days(-offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A date, as a test reads it.
    fn day(text: &str) -> Result<Date, Box<dyn std::error::Error>> {
        match Date::parse(text) {
            Some(date) => Ok(date),
            None => Err(format!("the date {text} could not be read").into()),
        }
    }

    /// A date, as a test makes it.
    fn date(year: i32, month: u32, day: u32) -> Result<Date, Box<dyn std::error::Error>> {
        match Date::from_ymd(year, month, day) {
            Some(date) => Ok(date),
            None => Err("the date could not be made".into()),
        }
    }

    /// A fact, as a test writes it.
    fn fact(text: &str, distance_m: Option<f64>) -> Result<RunFact, Box<dyn std::error::Error>> {
        Ok(RunFact {
            date: day(text)?,
            distance_m,
        })
    }

    #[test]
    fn a_session_is_the_intent_and_nothing_else() -> Result<(), Box<dyn std::error::Error>> {
        let planned = PlanSession {
            date: date(2026, 5, 12)?,
            kind: SessionKind::Easy,
            target_distance_m: Some(10_000.0),
            label: String::new(),
        };
        assert_eq!(session_label(&planned), "10 km · easy");
        assert_eq!(
            session_label(&PlanSession {
                date: date(2026, 5, 12)?,
                kind: SessionKind::Rest,
                target_distance_m: None,
                label: String::new(),
            }),
            "rest"
        );
        Ok(())
    }

    #[test]
    fn a_kind_the_engine_does_not_name_keeps_its_own_name() {
        assert_eq!(
            SessionKind::from_name("tempo"),
            SessionKind::Other("tempo".to_string())
        );
        assert_eq!(SessionKind::from_name("Easy"), SessionKind::Easy);
        assert_eq!(SessionKind::from_name("tempo").name(), "tempo");
    }

    #[test]
    fn the_adherence_is_a_count_and_never_a_score() -> Result<(), Box<dyn std::error::Error>> {
        let block = PlanBlock {
            source: PlanSource::Prescribed,
            name: "A plan".to_string(),
            sessions: vec![
                PlanSession {
                    date: date(2026, 5, 10)?,
                    kind: SessionKind::Easy,
                    target_distance_m: Some(10_000.0),
                    label: String::new(),
                },
                PlanSession {
                    date: date(2026, 5, 11)?,
                    kind: SessionKind::Easy,
                    target_distance_m: Some(8_000.0),
                    label: String::new(),
                },
            ],
        };
        let actual = vec![
            RunFact {
                date: date(2026, 5, 10)?,
                distance_m: Some(10_400.0),
            },
            RunFact {
                date: date(2026, 5, 12)?,
                distance_m: None,
            },
        ];
        let adherence = adherence_of(&block, &actual);
        assert_eq!(adherence.planned, 2);
        assert_eq!(adherence.done, 1);
        assert_eq!(adherence.not_done, 1);
        Ok(())
    }

    #[test]
    fn a_baseline_never_yields_not_done() -> Result<(), Box<dyn std::error::Error>> {
        let block = PlanBlock {
            source: PlanSource::Baseline,
            name: "Your usual week".to_string(),
            sessions: vec![PlanSession {
                date: date(2026, 5, 10)?,
                kind: SessionKind::Easy,
                target_distance_m: Some(10_000.0),
                label: String::new(),
            }],
        };
        let actual = vec![RunFact {
            date: date(2026, 5, 12)?,
            distance_m: None,
        }];
        let adherence = adherence_of(&block, &actual);
        assert_eq!(adherence.not_done, 0);
        assert_eq!(adherence.planned, 1);
        Ok(())
    }

    #[test]
    fn the_difference_is_stated_only_beyond_ten_per_cent() {
        assert!(distance_comparison(Some(10_200.0), Some(10_000.0)).is_none());
        assert_eq!(
            distance_comparison(Some(12_400.0), Some(10_000.0)),
            Some("2.4 km more than the 10 km target".to_string())
        );
        assert!(distance_comparison(None, Some(10_000.0)).is_none());
    }

    #[test]
    fn the_baseline_describes_the_athletes_own_week() -> Result<(), Box<dyn std::error::Error>> {
        let runs = vec![
            fact("2026-05-04", Some(10_000.0))?,
            fact("2026-05-06", Some(12_000.0))?,
            fact("2026-05-11", Some(11_000.0))?,
            fact("2026-05-13", Some(13_000.0))?,
        ];
        let block = match baseline_from_history(&runs, 4) {
            Some(block) => block,
            None => return Err("there was enough history for a baseline".into()),
        };
        assert_eq!(block.source, PlanSource::Baseline);
        assert_eq!(block.sessions.len(), 2);
        Ok(())
    }
}
