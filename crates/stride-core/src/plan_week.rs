//! The plan's week against the week that happened.
//!
//! **The plan is the intent; the week is the fact.** This module reads
//! the two against each other and says what it finds — and it says it
//! as plain fact: a session is *done* or *not done*, a distance is
//! compared with the distance that was planned, and a pace is compared
//! with the plan's pace **only when both of them exist**.
//!
//! ## The rules this module holds
//!
//! * **Done and not done are facts.** They are the plan engine's own
//!   statuses, and nothing here grades the athlete, counts a percentage
//!   or calls a session missed.
//! * **The paces are compared only when both exist.** The general
//!   aerobic day carries a heart-rate band and no pace, and a run
//!   without a pace is compared on its distance or on nothing — never
//!   against a pace that is not there.
//! * **The comparison is like for like.** The days that have passed are
//!   compared with the days that have passed: the plan's first three
//!   days against the athlete's first three days, never the plan's
//!   whole week against a part of it.
//! * **One plan.** The athlete's own plan is compared with the week
//!   that happened. Two plans are never compared with each other, and
//!   the app never says which plan is better.
//!
//! There is no clock here: the day is an argument, and the facts of the
//! week are the caller's.

use crate::date::Date;
use crate::plan::{distance_comparison, SessionStatus};
use crate::plan_bases::{
    read_entry, read_pacing, read_source, AdoptedPlan, AdoptedWeek, DateWords, EntryReading,
    PacingReading, PlanReading,
};
use crate::strings;
use crate::training_paces::PaceValue;
use crate::units;

/// Where the week stands against the plan.
///
/// The four states the card has: no plan at all, a plan whose week has
/// not begun, a plan whose week is running, and a plan whose week has
/// passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeekState {
    /// No plan has been adopted.
    None,
    /// The plan's first day is after this week.
    Before,
    /// This week is inside the plan.
    Running,
    /// The plan's last day is before this week.
    After,
}

impl WeekState {
    /// The stable name of this state.
    pub fn as_str(self) -> &'static str {
        match self {
            WeekState::None => "none",
            WeekState::Before => "before",
            WeekState::Running => "running",
            WeekState::After => "after",
        }
    }
}

/// What happened on a day, as the plan's week reads it.
///
/// A day with no run is **absent from the list**: a fact of no value is
/// not a zero, and a run with no distance keeps the distance absent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeekFact {
    /// The day.
    pub date: Date,
    /// The distance run that day, in metres.
    pub distance_m: Option<f64>,
    /// The pace of that day's running, in seconds per kilometre.
    pub pace_s_per_km: Option<f64>,
}

impl WeekFact {
    /// One day of the week that happened.
    pub fn new(date: Date, distance_m: Option<f64>, pace_s_per_km: Option<f64>) -> Self {
        Self {
            date,
            distance_m,
            pace_s_per_km,
        }
    }
}

/// The plan's week: its label and its days.
#[derive(Clone, Debug, PartialEq)]
pub struct WeekWindow {
    /// The week's own label: "3 weeks to go".
    pub label: String,
    /// The first day of the week.
    pub start_date: Date,
    /// The last day of the week.
    pub end_date: Date,
}

/// The week's totals, against the plan's.
///
/// **Like for like**: the plan's days that have passed against the
/// athlete's days that have passed.
#[derive(Clone, Debug, PartialEq)]
pub struct WeekTotals {
    /// The distance the plan asked for, in metres.
    pub planned_m: f64,
    /// The distance that was run, in metres.
    pub actual_m: f64,
    /// The difference between the two, in metres: the athlete's distance
    /// minus the plan's.
    pub difference_m: f64,
    /// The line the card leads with: "38.4 km against the plan's 42 km".
    pub line: Option<String>,
}

/// One planned session and what happened on its day.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionComparison {
    /// The day the session falls on.
    pub date: Date,
    /// The distance that was run, in metres.
    pub actual_distance_m: Option<f64>,
    /// The difference between the distance run and the distance planned,
    /// when there is one to state.
    pub difference: Option<String>,
    /// The pace that was run, read against the plan's pace.
    ///
    /// Absent unless **both** of them exist.
    pub pace: Option<String>,
    /// The plain status of the session: done, not done or planned.
    ///
    /// `None` for a day that is not a run: a rest day is not done and
    /// not not done — it is a rest day.
    pub status: Option<SessionStatus>,
}

/// The plan's week against the week that happened.
#[derive(Clone, Debug, PartialEq)]
pub struct WeekComparison {
    /// Where the week stands against the plan.
    pub state: WeekState,
    /// The state's own sentence: "no plan has been adopted".
    pub state_line: Option<String>,
    /// The plan's own source.
    pub source: Option<PlanReading>,
    /// What the plan was paced with.
    pub pacing: Option<PacingReading>,
    /// The tier's entry requirement beside the athlete's own mileage.
    pub entry: Option<EntryReading>,
    /// The plan's week, when this week is inside the plan.
    pub week: Option<WeekWindow>,
    /// The week's totals, against the plan's.
    pub totals: Option<WeekTotals>,
    /// The sessions of the plan's week, each with what happened on it.
    pub sessions: Vec<SessionComparison>,
}

/// The line that points at the programmes there are to adopt.
///
/// The app never offers a programme the athlete did not adopt, and it
/// never compares two of them: it says how many there are, and the
/// athlete picks.
pub fn choose_line() -> Option<String> {
    let n = crate::plan_bases::programmes().len().to_string();
    strings::render("statement.plan.adoption.choose", &[("n", &n)])
}

/// The plan's week against the week that happened.
///
/// The plan is the athlete's own: `None` when no plan has been adopted,
/// and the answer then says so. The day is the athlete's own day, and
/// the facts are the runs of the week that holds it.
///
/// **Nothing here compares two plans**, and nothing here says which
/// plan is better: the plan in force is the athlete's own, and it is
/// compared with the week that happened.
pub fn compare_week(
    plan: Option<&AdoptedPlan>,
    day: Date,
    facts: &[WeekFact],
    words: &DateWords,
) -> WeekComparison {
    let plan = match plan {
        Some(plan) => plan,
        None => {
            return WeekComparison {
                state: WeekState::None,
                state_line: strings::text("statement.plan.adoption.none")
                    .map(|text| text.to_string()),
                source: None,
                pacing: None,
                entry: None,
                week: None,
                totals: None,
                sessions: Vec::new(),
            }
        }
    };

    let source = Some(read_source(plan));
    let pacing = Some(read_pacing(plan, words));
    let entry = Some(read_entry(plan));

    let state = if day < plan.start_date {
        WeekState::Before
    } else if day > plan.end_date {
        WeekState::After
    } else {
        WeekState::Running
    };

    let state_line = match state {
        WeekState::None => None,
        WeekState::Before => words
            .start
            .as_ref()
            .and_then(|date| strings::render("statement.plan.week.before", &[("date", date)])),
        WeekState::After => words
            .end
            .as_ref()
            .and_then(|date| strings::render("statement.plan.week.after", &[("date", date)])),
        WeekState::Running => None,
    };

    let week = plan
        .weeks
        .iter()
        .find(|week| day >= week.start_date && day <= week.end_date);

    match week {
        Some(week) => {
            let sessions = sessions_of(week, day, facts);
            let totals = totals_of(week, day, facts);
            WeekComparison {
                state,
                state_line,
                source,
                pacing,
                entry,
                week: Some(WeekWindow {
                    label: week.label.clone(),
                    start_date: week.start_date,
                    end_date: week.end_date,
                }),
                totals,
                sessions,
            }
        }
        None => WeekComparison {
            state,
            state_line,
            source,
            pacing,
            entry,
            week: None,
            totals: None,
            sessions: Vec::new(),
        },
    }
}

/// The sessions of the plan's week, each with what happened on its day.
fn sessions_of(week: &AdoptedWeek, day: Date, facts: &[WeekFact]) -> Vec<SessionComparison> {
    week.sessions
        .iter()
        .map(|session| {
            let fact = facts.iter().find(|fact| fact.date == session.date);
            let ran = fact.is_some();
            let actual_distance_m = fact.and_then(|fact| fact.distance_m);
            let difference = distance_comparison(actual_distance_m, session.distance_m);
            let pace = pace_comparison(&session.pace, fact.and_then(|fact| fact.pace_s_per_km));
            let status = if !session.kind.is_run() {
                // A rest day is not a run: it is neither done nor not
                // done, and the app never counts it as either.
                None
            } else if ran {
                Some(SessionStatus::Done)
            } else if session.date < day {
                Some(SessionStatus::NotDone)
            } else {
                Some(SessionStatus::Planned)
            };
            SessionComparison {
                date: session.date,
                actual_distance_m,
                difference,
                pace,
                status,
            }
        })
        .collect()
}

/// The week's totals, against the plan's.
///
/// **Like for like**: the days that have passed on both sides. The
/// plan's first three days are compared with the athlete's first three
/// days, and the rest of the plan's week is not counted against a part
/// of a week.
///
/// A session of the day that is still running is not counted against
/// the athlete either: the day is not over, so the plan's ask for it has
/// not been settled yet. The day it was run, it counts.
fn totals_of(week: &AdoptedWeek, day: Date, facts: &[WeekFact]) -> Option<WeekTotals> {
    let elapsed_end = if day < week.end_date {
        day
    } else {
        week.end_date
    };
    if elapsed_end < week.start_date {
        return None;
    }
    let planned_m: f64 = week
        .sessions
        .iter()
        .filter(|session| session.date >= week.start_date && session.date <= elapsed_end)
        .filter(|session| session.date < day || facts.iter().any(|fact| fact.date == session.date))
        .filter_map(|session| session.distance_m)
        .sum();
    let actual_m: f64 = facts
        .iter()
        .filter(|fact| fact.date >= week.start_date && fact.date <= elapsed_end)
        .filter_map(|fact| fact.distance_m)
        .sum();
    let line = totals_line(actual_m, planned_m);
    Some(WeekTotals {
        planned_m,
        actual_m,
        difference_m: actual_m - planned_m,
        line,
    })
}

/// The totals line: "38.4 km against the plan's 42 km".
///
/// The line is the core's, and the two numbers in it are the two the
/// card shows: the difference beside it is the same subtraction.
fn totals_line(actual_m: f64, planned_m: f64) -> Option<String> {
    let actual = units::render_distance(Some(actual_m))?;
    let planned = units::render_distance(Some(planned_m))?;
    strings::render(
        "statement.plan.week.totals",
        &[("actual", &actual), ("planned", &planned)],
    )
}

/// The pace that was run, read against the plan's pace.
///
/// **Both must exist.** A session with no pace — the general aerobic
/// day, which carries a heart-rate band and no pace — is compared on
/// its distance or on nothing, and never against a pace that is not
/// there.
///
/// The comparison is against the band the plan gives: inside it, faster
/// than it, or slower than it, with the gap to the nearest edge.
fn pace_comparison(planned: &PaceValue, actual_s_per_km: Option<f64>) -> Option<String> {
    let band = planned.band()?;
    let actual = actual_s_per_km?;
    if !actual.is_finite() || actual <= 0.0 {
        return None;
    }
    let pace = units::render_pace(Some(actual))?;
    let band_text = band.render()?;
    if actual >= band.fast_s_per_km && actual <= band.slow_s_per_km {
        return strings::render(
            "statement.plan.week.pace.within",
            &[("pace", &pace), ("band", &band_text)],
        );
    }
    let (gap, id) = if actual < band.fast_s_per_km {
        (
            band.fast_s_per_km - actual,
            "statement.plan.week.pace.faster",
        )
    } else {
        (
            actual - band.slow_s_per_km,
            "statement.plan.week.pace.slower",
        )
    };
    let gap = units::round_half_even(gap, 0) as i64;
    let gap_text = strings::render("value.pace_difference", &[("value", &gap.to_string())])?;
    strings::render(
        id,
        &[("pace", &pace), ("gap", &gap_text), ("band", &band_text)],
    )
}
