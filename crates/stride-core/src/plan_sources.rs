//! Where a plan comes from when nobody wrote one down.
//!
//! Two sources, and they are **not** the same kind of thing:
//!
//! * **The plan you already have** — inferred from the athlete's own
//!   history: their weekly volume, their rest days, their long-run day,
//!   their quality day and their usual distance per weekday. It is a
//!   **baseline**, not a prescription, and it says so in every sentence it
//!   produces.
//! * **The plan you wrote down** — Garmin's scheduled workouts. It is a
//!   **prescription**, because the athlete wrote it in Garmin Connect.
//!
//! The distinction is not cosmetic. A baseline is compared with *less than
//! usual* / *more than usual* / *on a usual day*; a prescription is
//! compared with *hit* and *missed*. If the screen mixes the two, the
//! athlete cannot tell which kind of comparison they are reading.
//!
//! The reference is `backend/app/services/plan_sources.py`, and the golden
//! fixtures in `crates/stride-core/tests/fixtures/plan_sources/` prove the
//! two agree.
//!
//! # The window
//!
//! The inference looks at the twelve whole weeks that end before the range
//! being compared starts. The window is cut at a week boundary on purpose:
//! a partial last week would put a part-week's volume into the weekly
//! median, and would leak days of the range into the baseline it is
//! compared against.
//!
//! # The minimum history
//!
//! Eight weeks of the twelve, and twelve runs. Eight of twelve means the
//! athlete was seen running in at least two thirds of the window; twelve
//! runs is three observations of each of the four weekdays a runner usually
//! uses. Below that the per-weekday median is a single run, and the plan
//! **declines** rather than inventing a pattern from noise.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::model::{Activity, PlanSession};
use crate::numeric::{median, round_to};

/// How many weeks of history the inference looks at.
pub const HISTORY_WEEKS: usize = 12;

/// How many of those weeks must contain a run before a pattern is inferred.
pub const MIN_HISTORY_WEEKS: usize = 8;

/// How many runs the window must hold before a pattern is inferred.
pub const MIN_HISTORY_RUNS: usize = 12;

/// How many runs on a weekday before that weekday counts as a habit.
pub const MIN_WEEKDAY_RUNS: usize = 3;

/// How far from the usual distance still counts as "on a usual day".
pub const USUAL_TOLERANCE: f64 = 0.10;

/// How wide the usual pace band is, either side of the usual pace.
pub const PACE_BAND_TOLERANCE: f64 = 0.05;

/// The sports the plan engine counts as running. A walk is not a run.
pub const RUN_SPORTS: [&str; 6] = [
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
    "street_running",
    "track_running",
];

/// The session types the `plan_session` table already knows.
///
/// These are the kinds a plan may *prescribe*. `unknown` is not one of
/// them: it is the absence of a kind, and it is what an imported row that
/// names no kind of session carries. It is accepted on the import paths
/// and read as "not known" everywhere else.
pub const SESSION_TYPES: [&str; 8] = [
    "easy", "long", "tempo", "interval", "race", "recovery", "strength", "rest",
];

/// The weekdays, Monday first — the order `date.weekday()` uses.
pub const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// The verdicts a baseline comparison can produce. Never "hit", never
/// "missed": nothing prescribed the session.
pub const BASELINE_VERDICTS: [&str; 3] = ["less than usual", "more than usual", "on a usual day"];

/// What a Garmin plan says about itself.
pub const GARMIN_DESCRIPTION: &str =
    "Scheduled in Garmin Connect. This is the plan you wrote down, so every session is compared \
     against your own targets.";

/// The words an inferred plan uses about itself.
fn inferred_description() -> String {
    format!(
        "Inferred from your last {HISTORY_WEEKS} weeks of running. This is a baseline of how you \
         usually run, not a prescription."
    )
}

// ------------------------------------------------------------------ the plan

/// The plan an athlete already has, inferred from their own history.
///
/// The result is a *baseline*. It is never a prescription, and it declines
/// to say anything at all when the history is too thin to support a
/// pattern — see [`MIN_HISTORY_WEEKS`] and [`MIN_HISTORY_RUNS`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferredPlan {
    /// Where the plan came from: `inferred`.
    pub source: String,
    /// Always `false`. An inferred plan is a baseline, never a prescription.
    pub prescribed: bool,
    /// What the plan says about itself, in words.
    pub description: String,
    /// Whether there was enough history to infer a pattern.
    pub sufficient: bool,
    /// Why the plan declined, when it declined.
    pub reason: Option<String>,
    /// How many of the window's weeks contain a run.
    pub history_weeks: usize,
    /// How many runs the window holds.
    pub history_runs: usize,
    /// The first day of the window the inference looked at.
    pub window_start: Option<String>,
    /// The last day of the window the inference looked at.
    pub window_end: Option<String>,
    /// The median weekly volume of the window, in metres.
    pub weekly_volume_m: Option<f64>,
    /// The lowest weekly volume the athlete actually covered, in metres.
    pub weekly_volume_low_m: Option<f64>,
    /// The highest weekly volume of the window, in metres.
    pub weekly_volume_high_m: Option<f64>,
    /// The weekdays with no runs at all.
    pub rest_days: Vec<String>,
    /// The weekday of the week's longest run.
    pub long_run_day: Option<String>,
    /// The weekday the athlete runs fastest on.
    pub quality_day: Option<String>,
    /// The usual distance per weekday, in metres.
    pub usual_by_weekday: BTreeMap<String, f64>,
    /// The dated sessions of the range.
    pub sessions: Vec<PlanSession>,
}

/// The plan the athlete wrote down, as Garmin's scheduled workouts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkoutsPlan {
    /// Where the plan came from: `garmin`.
    pub source: String,
    /// Always `true`. A scheduled workout is a prescription.
    pub prescribed: bool,
    /// What the plan says about itself, in words.
    pub description: String,
    /// The dated sessions.
    pub sessions: Vec<PlanSession>,
}

/// One scheduled Garmin workout, as `GarminWorkoutClient.scheduled_workouts`
/// returns it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScheduledWorkout {
    /// Garmin's id for the workout.
    #[serde(default, deserialize_with = "deserialize_loose_string")]
    pub workout_id: Option<String>,
    /// The workout's name.
    #[serde(default)]
    pub name: Option<String>,
    /// The day the workout is scheduled for.
    #[serde(default, deserialize_with = "deserialize_loose_string")]
    pub date: Option<String>,
    /// The sport the workout is for.
    #[serde(default)]
    pub sport: Option<String>,
    /// The workout's status.
    #[serde(default)]
    pub status: Option<String>,
}

/// The detail of one Garmin workout, as `GarminWorkoutClient.workout_detail`
/// returns it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkoutDetail {
    /// Garmin's id for the workout, so the detail can be paired with it.
    #[serde(default, deserialize_with = "deserialize_loose_string")]
    pub workout_id: Option<String>,
    /// The session type the workout implies.
    #[serde(default)]
    pub session_type: Option<String>,
    /// The target distance, in metres.
    #[serde(default)]
    pub target_distance_m: Option<f64>,
    /// The lower bound of the pace band, in seconds per kilometre.
    #[serde(default)]
    pub target_pace_low_s_km: Option<f64>,
    /// The upper bound of the pace band, in seconds per kilometre.
    #[serde(default)]
    pub target_pace_high_s_km: Option<f64>,
    /// The heart-rate zone the workout targets.
    #[serde(default)]
    pub hr_zone_target: Option<f64>,
    /// The workout's title.
    #[serde(default)]
    pub title: Option<String>,
    /// The workout's steps, as text.
    #[serde(default)]
    pub details: Option<String>,
}

/// The verdict on one run against the athlete's usual.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineVerdict {
    /// The verdict: `less_than_usual`, `more_than_usual` or `on_a_usual_day`.
    pub verdict: String,
    /// The words the athlete reads.
    pub verdict_label: String,
    /// The usual distance, in metres.
    pub usual_distance_m: Option<f64>,
    /// The distance actually run, in metres.
    pub actual_distance_m: Option<f64>,
    /// The difference from the usual, in metres.
    pub delta_m: Option<f64>,
    /// The difference from the middle of the usual pace band, in seconds per
    /// kilometre.
    pub pace_delta_s_km: Option<f64>,
    /// The sentence the screen shows.
    pub message: String,
}

// -------------------------------------------------------------------- helpers

/// The weekday of a date, Monday being zero.
pub fn weekday_index(date: Date) -> usize {
    (date.to_days() + 3).rem_euclid(7) as usize
}

/// The name of a weekday, Monday being zero.
pub fn weekday_name(index: usize) -> &'static str {
    WEEKDAYS[index % WEEKDAYS.len()]
}

/// Whether a sport type counts as a run.
pub fn is_run_sport(sport_type: &str) -> bool {
    RUN_SPORTS.contains(&sport_type.trim().to_lowercase().as_str())
}

/// Whether a Garmin workout's sport is a run.
///
/// Garmin's calendar uses `running` for road, trail, track and treadmill
/// alike, and the sport strings it returns are not the importer's normalised
/// ones, so a run is anything whose sport says "run".
pub fn is_run_workout(sport: Option<&str>) -> bool {
    match sport {
        None => true,
        Some(sport) => {
            let text = sport.trim().to_lowercase();
            text.is_empty() || RUN_SPORTS.contains(&text.as_str()) || text.contains("run")
        }
    }
}

/// Parse a calendar date out of what Garmin or the database holds.
///
/// Accepts an ISO date (`2026-03-08`), an ISO timestamp
/// (`2026-03-08T09:00:00`) and an epoch in seconds or milliseconds.
pub fn normalise_date(value: &str) -> Option<String> {
    let text = value.trim();
    if text.is_empty() {
        return None;
    }
    if let Some(date) = Date::parse(text) {
        return Some(date.to_string());
    }
    if text.chars().all(|character| character.is_ascii_digit()) {
        if let Ok(number) = text.parse::<i64>() {
            let days = if number.abs() > 100_000_000_000 {
                number.div_euclid(86_400_000)
            } else {
                number.div_euclid(86_400)
            };
            return Some(Date::from_days(days).to_string());
        }
    }
    None
}

/// The average pace of an activity, in seconds per kilometre.
fn pace_of(activity: &Activity) -> Option<f64> {
    activity.pace_s_per_km()
}

/// A distance in kilometres, to one decimal.
fn km(value: Option<f64>) -> String {
    match value {
        Some(value) => format!("{:.1}", value / 1000.0),
        None => "-".to_string(),
    }
}

/// One run, reduced to what the inference needs.
#[derive(Clone, Copy, Debug)]
struct Fact {
    date: Date,
    distance_m: f64,
    moving_time_s: Option<f64>,
}

/// The runs of the history: a run sport, a real distance, a real date.
fn facts(activities: &[Activity]) -> Vec<Fact> {
    let mut runs: Vec<Fact> = activities
        .iter()
        .filter(|activity| is_run_sport(&activity.sport_type) && activity.distance_m > 0.0)
        .filter_map(|activity| {
            let date = Date::parse(&activity.local_date)?;
            Some(Fact {
                date,
                distance_m: activity.distance_m,
                moving_time_s: activity.moving_time_s,
            })
        })
        .collect();
    runs.sort_by_key(|fact| fact.date);
    runs
}

/// The Monday that starts the ISO week the date falls in.
fn iso_week_start(date: Date) -> Date {
    date.iso_week_start()
}

/// The twelve whole weeks that end before the range starts.
/// The twelve whole weeks that end before the range starts.
///
/// The baseline is read from the athlete's history *before* the range it
/// is compared against, so a caller that wants the baseline has to fetch
/// that history with it. This is where that history starts.
pub fn history_window(from_date: &str) -> Option<(String, String)> {
    let start = Date::parse(from_date)?;
    let (from, to) = window(start);
    Some((from.to_string(), to.to_string()))
}

fn window(from_date: Date) -> (Date, Date) {
    let start = iso_week_start(from_date).add_days(-7 * HISTORY_WEEKS as i64);
    let end = iso_week_start(from_date).add_days(-1);
    (start, end)
}

// ------------------------------------------------------------ inferred plan

/// The plan an athlete already has, inferred from their own history.
///
/// The result is a *baseline*. It is never a prescription, and it declines
/// to say anything at all when the history is too thin to support a
/// pattern.
pub fn infer_plan(activities: &[Activity], from_date: &str, to_date: &str) -> InferredPlan {
    let start_date = Date::parse(from_date);
    let end_date = Date::parse(to_date);
    let (Some(start_date), Some(end_date)) = (start_date, end_date) else {
        return declined_plan(
            "The range is not a date, so there is nothing to infer.",
            0,
            0,
            None,
            None,
            None,
            None,
            None,
        );
    };

    let (window_start, window_end) = window(start_date);
    let runs: Vec<Fact> = facts(activities)
        .into_iter()
        .filter(|fact| fact.date >= window_start && fact.date <= window_end)
        .collect();

    // The weekly volume: one bucket per week of the window, zeros included.
    let mondays: Vec<Date> = (0..HISTORY_WEEKS)
        .map(|index| window_start.add_days(7 * index as i64))
        .collect();
    let mut totals: Vec<f64> = vec![0.0; HISTORY_WEEKS];
    for fact in &runs {
        let monday = iso_week_start(fact.date);
        if let Some(index) = mondays.iter().position(|candidate| *candidate == monday) {
            totals[index] += fact.distance_m;
        }
    }

    let covered: Vec<f64> = totals
        .iter()
        .copied()
        .filter(|total| *total > 0.0)
        .collect();
    let history_weeks = covered.len();
    let history_runs = runs.len();

    let weekly_volume = median(&totals);
    let volume_low = covered.iter().copied().reduce(f64::min);
    let volume_high = totals.iter().copied().reduce(f64::max);

    if history_runs < MIN_HISTORY_RUNS || history_weeks < MIN_HISTORY_WEEKS {
        return declined_plan(
            &format!(
                "Not enough history to infer a usual week: {history_runs} runs over {history_weeks} \
                 weeks. Stride needs at least {MIN_HISTORY_RUNS} runs over {MIN_HISTORY_WEEKS} \
                 weeks."
            ),
            history_weeks,
            history_runs,
            weekly_volume,
            volume_low,
            volume_high,
            Some(window_start),
            Some(window_end),
        );
    }

    // Per-weekday statistics.
    let mut distances: Vec<Vec<f64>> = vec![Vec::new(); 7];
    let mut paces: Vec<Vec<f64>> = vec![Vec::new(); 7];
    for fact in &runs {
        let weekday = weekday_index(fact.date);
        distances[weekday].push(fact.distance_m);
        if let Some(pace) = fact
            .moving_time_s
            .filter(|moving| *moving > 0.0)
            .map(|moving| moving / (fact.distance_m / 1000.0))
        {
            paces[weekday].push(pace);
        }
    }

    let run_weekdays: Vec<usize> = (0..7)
        .filter(|index| distances[*index].len() >= MIN_WEEKDAY_RUNS)
        .collect();
    let rest_days: Vec<String> = (0..7)
        .filter(|index| distances[*index].is_empty())
        .map(|index| weekday_name(index).to_string())
        .collect();

    let mut usual_by_weekday: BTreeMap<String, f64> = BTreeMap::new();
    let mut median_distance: Vec<Option<f64>> = vec![None; 7];
    let mut median_pace: Vec<Option<f64>> = vec![None; 7];
    for index in &run_weekdays {
        median_distance[*index] = median(&distances[*index]);
        median_pace[*index] = median(&paces[*index]);
        if let Some(distance) = median_distance[*index] {
            usual_by_weekday.insert(weekday_name(*index).to_string(), round_to(distance, 0));
        }
    }

    let long_run_day = long_run_day(&runs, &mondays);
    let quality_day = quality_day(&run_weekdays, &median_pace, &distances);

    let mut sessions: Vec<PlanSession> = Vec::new();
    if end_date >= start_date {
        let days = start_date.days_until(end_date);
        for offset in 0..=days {
            let day = start_date.add_days(offset);
            let weekday = weekday_index(day);
            let Some(distance) = median_distance[weekday] else {
                continue;
            };
            sessions.push(inferred_session(
                day,
                weekday,
                distance,
                median_pace[weekday],
                long_run_day.as_deref(),
                quality_day.as_deref(),
            ));
        }
    }

    InferredPlan {
        source: "inferred".to_string(),
        prescribed: false,
        description: inferred_description(),
        sufficient: true,
        reason: None,
        history_weeks,
        history_runs,
        window_start: Some(window_start.to_string()),
        window_end: Some(window_end.to_string()),
        weekly_volume_m: weekly_volume.map(|value| round_to(value, 0)),
        weekly_volume_low_m: volume_low.map(|value| round_to(value, 0)),
        weekly_volume_high_m: volume_high.map(|value| round_to(value, 0)),
        rest_days,
        long_run_day,
        quality_day,
        usual_by_weekday,
        sessions,
    }
}

/// A plan that declines to infer a pattern, and says why.
#[allow(clippy::too_many_arguments)]
fn declined_plan(
    reason: &str,
    history_weeks: usize,
    history_runs: usize,
    weekly_volume: Option<f64>,
    volume_low: Option<f64>,
    volume_high: Option<f64>,
    window_start: Option<Date>,
    window_end: Option<Date>,
) -> InferredPlan {
    InferredPlan {
        source: "inferred".to_string(),
        prescribed: false,
        description: inferred_description(),
        sufficient: false,
        reason: Some(reason.to_string()),
        history_weeks,
        history_runs,
        window_start: window_start.map(|date| date.to_string()),
        window_end: window_end.map(|date| date.to_string()),
        weekly_volume_m: weekly_volume.map(|value| round_to(value, 0)),
        weekly_volume_low_m: volume_low.map(|value| round_to(value, 0)),
        weekly_volume_high_m: volume_high.map(|value| round_to(value, 0)),
        rest_days: Vec::new(),
        long_run_day: None,
        quality_day: None,
        usual_by_weekday: BTreeMap::new(),
        sessions: Vec::new(),
    }
}

/// The weekday of the longest run of the week, the usual one.
///
/// Every week of the window contributes the weekday of its own longest run;
/// the weekday that comes up most often is the long-run day. A tie is
/// broken by the earlier weekday.
fn long_run_day(runs: &[Fact], mondays: &[Date]) -> Option<String> {
    let mut votes = [0usize; 7];
    for monday in mondays {
        let mut longest: Option<&Fact> = None;
        for fact in runs {
            let week_start = iso_week_start(fact.date);
            if week_start != *monday {
                continue;
            }
            // The first of several equally long runs wins, like Python's
            // `max()`.
            match longest {
                Some(current) if current.distance_m >= fact.distance_m => {}
                _ => longest = Some(fact),
            }
        }
        if let Some(fact) = longest {
            votes[weekday_index(fact.date)] += 1;
        }
    }
    let mut best: Option<usize> = None;
    for index in 0..7 {
        match best {
            Some(current) if votes[current] >= votes[index] => {}
            _ => best = Some(index),
        }
    }
    best.map(|index| weekday_name(index).to_string())
}

/// The weekday the athlete runs fastest on.
///
/// The fastest median pace wins. Two weekdays within five seconds a
/// kilometre are a tie, and the tie is broken by the weekday with more
/// distance behind it — that is the one with the most work at intensity.
fn quality_day(
    run_weekdays: &[usize],
    median_pace: &[Option<f64>],
    distances: &[Vec<f64>],
) -> Option<String> {
    let paced: Vec<usize> = run_weekdays
        .iter()
        .copied()
        .filter(|index| median_pace[*index].is_some())
        .collect();
    if paced.is_empty() {
        return None;
    }
    let fastest = paced
        .iter()
        .filter_map(|index| median_pace[*index])
        .fold(f64::INFINITY, f64::min);
    let candidates: Vec<usize> = paced
        .iter()
        .copied()
        .filter(|index| median_pace[*index].is_some_and(|pace| pace <= fastest + 5.0))
        .collect();
    let mut best: Option<usize> = None;
    for index in candidates {
        let total: f64 = distances[index].iter().copied().sum();
        match best {
            Some(current) => {
                let current_total: f64 = distances[current].iter().copied().sum();
                if total > current_total {
                    best = Some(index);
                }
            }
            None => best = Some(index),
        }
    }
    best.map(|index| weekday_name(index).to_string())
}

/// One dated session of the inferred week.
fn inferred_session(
    day: Date,
    weekday: usize,
    usual_distance_m: f64,
    usual_pace: Option<f64>,
    long_run_day: Option<&str>,
    quality_day: Option<&str>,
) -> PlanSession {
    let name = weekday_name(weekday);
    let (session_type, title) = if long_run_day == Some(name) {
        ("long", "Your usual long run".to_string())
    } else if quality_day == Some(name) {
        ("tempo", "Your usual quality run".to_string())
    } else {
        ("easy", format!("Your usual {name} run"))
    };

    let (pace_low, pace_high) = match usual_pace {
        Some(pace) => (
            Some(round_to(pace * (1.0 - PACE_BAND_TOLERANCE), 0)),
            Some(round_to(pace * (1.0 + PACE_BAND_TOLERANCE), 0)),
        ),
        None => (None, None),
    };

    PlanSession {
        id: 0,
        date: day.to_string(),
        session_type: session_type.to_string(),
        title: Some(title),
        details: None,
        target_distance_m: Some(round_to(usual_distance_m, 0)),
        pace_low_s_km: pace_low,
        pace_high_s_km: pace_high,
    }
}

// ---------------------------------------------------------- baseline verdict

/// Compare a run with the athlete's usual, without prescribing.
///
/// The verdict is *less than usual*, *more than usual* or *on a usual day*.
/// There is no "missed" and no "hit": nobody wrote this plan down, so
/// nothing was missed.
pub fn baseline_verdict(session: &PlanSession, activity: Option<&Activity>) -> BaselineVerdict {
    let usual = session.target_distance_m.filter(|value| *value > 0.0);
    let name = match Date::parse(&session.date) {
        Some(date) => weekday_name(weekday_index(date)),
        None => "day",
    };

    let actual = activity
        .filter(|activity| activity.distance_m > 0.0)
        .map(|activity| activity.distance_m);
    let pace = activity.and_then(pace_of);

    let Some(usual) = usual else {
        return BaselineVerdict {
            verdict: "no_usual_session".to_string(),
            verdict_label: "no usual session".to_string(),
            usual_distance_m: None,
            actual_distance_m: None,
            delta_m: None,
            pace_delta_s_km: None,
            message: "There is no usual session to compare this run with.".to_string(),
        };
    };

    let usual_text = format!("You usually run {} km on a {name}", km(Some(usual)));
    let (verdict, message) = match actual {
        None => (
            "less_than_usual",
            format!("{usual_text}; nothing was logged, which is less than usual."),
        ),
        Some(actual) if actual < usual * (1.0 - USUAL_TOLERANCE) => (
            "less_than_usual",
            format!(
                "{usual_text}; you ran {} km, which is less than usual.",
                km(Some(actual))
            ),
        ),
        Some(actual) if actual > usual * (1.0 + USUAL_TOLERANCE) => (
            "more_than_usual",
            format!(
                "{usual_text}; you ran {} km, which is more than usual.",
                km(Some(actual))
            ),
        ),
        Some(actual) => (
            "on_a_usual_day",
            format!(
                "{usual_text}; you ran {} km, which is on a usual day.",
                km(Some(actual))
            ),
        ),
    };

    let delta = actual.map(|actual| actual - usual);
    let pace_delta = pace.and_then(|pace| {
        let bounds: Vec<f64> = [session.pace_low_s_km, session.pace_high_s_km]
            .into_iter()
            .flatten()
            .collect();
        if bounds.is_empty() {
            None
        } else {
            Some(pace - (bounds.iter().sum::<f64>() / bounds.len() as f64))
        }
    });

    BaselineVerdict {
        verdict: verdict.to_string(),
        verdict_label: verdict_label(verdict).to_string(),
        usual_distance_m: Some(round_to(usual, 0)),
        actual_distance_m: actual.map(|value| round_to(value, 0)),
        delta_m: delta.map(|value| round_to(value, 0)),
        pace_delta_s_km: pace_delta.map(|value| round_to(value, 1)),
        message,
    }
}

/// The words a verdict slug is read as.
fn verdict_label(verdict: &str) -> &'static str {
    match verdict {
        "less_than_usual" => "less than usual",
        "more_than_usual" => "more than usual",
        "on_a_usual_day" => "on a usual day",
        _ => "no usual session",
    }
}

// ---------------------------------------------------------- Garmin workouts

/// Map one scheduled Garmin workout onto a plan session.
///
/// `None` when the workout is not a run on a day — a cycling workout is not
/// a run, and a workout with no date cannot be planned.
pub fn session_from_workout(
    scheduled: &ScheduledWorkout,
    detail: Option<&WorkoutDetail>,
) -> Option<PlanSession> {
    let day = normalise_date(scheduled.date.as_deref().unwrap_or_default())?;
    if !is_run_workout(scheduled.sport.as_deref()) {
        return None;
    }
    let status = scheduled
        .status
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    if status.contains("cancel") || status.contains("delete") {
        return None;
    }

    let detail = detail.cloned().unwrap_or_default();
    // A workout that names no kind of session is unknown, not easy. The
    // scheduled workouts are a prescription, so a kind nobody wrote down
    // cannot be filled in with a guess.
    let mut session_type = detail
        .session_type
        .as_deref()
        .unwrap_or("unknown")
        .trim()
        .to_lowercase();
    if !SESSION_TYPES.contains(&session_type.as_str()) {
        session_type = "unknown".to_string();
    }

    let title = detail
        .title
        .filter(|title| !title.is_empty())
        .or_else(|| scheduled.name.clone().filter(|name| !name.is_empty()));

    Some(PlanSession {
        id: 0,
        date: day,
        session_type,
        title,
        details: detail.details.filter(|text| !text.is_empty()),
        target_distance_m: detail.target_distance_m,
        pace_low_s_km: detail.target_pace_low_s_km,
        pace_high_s_km: detail.target_pace_high_s_km,
    })
}

/// Map Garmin's scheduled workouts onto plan sessions.
///
/// The details are paired to their scheduled workout by `workout_id`.
pub fn sessions_from_workouts(
    scheduled: &[ScheduledWorkout],
    details: &[WorkoutDetail],
) -> Vec<PlanSession> {
    let mut by_id: BTreeMap<String, &WorkoutDetail> = BTreeMap::new();
    for detail in details {
        if let Some(id) = detail.workout_id.as_deref() {
            let id = id.trim();
            if !id.is_empty() {
                by_id.insert(id.to_string(), detail);
            }
        }
    }

    let mut sessions: Vec<PlanSession> = Vec::new();
    for row in scheduled {
        let detail = row
            .workout_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .and_then(|id| by_id.get(id).copied());
        if let Some(session) = session_from_workout(row, detail) {
            sessions.push(session);
        }
    }

    sessions.sort_by(|left, right| {
        left.date.cmp(&right.date).then_with(|| {
            left.title
                .clone()
                .unwrap_or_default()
                .cmp(&right.title.clone().unwrap_or_default())
        })
    });
    sessions
}

/// The plan the athlete wrote down, as Garmin's scheduled workouts.
pub fn workouts_plan(scheduled: &[ScheduledWorkout], details: &[WorkoutDetail]) -> WorkoutsPlan {
    WorkoutsPlan {
        source: "garmin".to_string(),
        prescribed: true,
        description: GARMIN_DESCRIPTION.to_string(),
        sessions: sessions_from_workouts(scheduled, details),
    }
}

// ------------------------------------------------------------- deserialisers

/// Accept a string or a number where a loose string is expected.
///
/// Garmin sends ids and dates as numbers sometimes and as strings other
/// times; the reference turns both into their text form.
fn deserialize_loose_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct LooseVisitor;

    impl<'de> Visitor<'de> for LooseVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a number")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(Some(value.to_string()))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(Some(value.to_string()))
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(Some(value.to_string()))
        }

        fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
            Ok(Some(value.to_string()))
        }

        fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
            Ok(Some(value.to_string()))
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(LooseVisitor)
        }
    }

    deserializer.deserialize_option(LooseVisitor)
}
