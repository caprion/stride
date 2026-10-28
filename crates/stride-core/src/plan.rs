//! The plan engine: which run answered which planned session, and whether it
//! was any good.
//!
//! The reference is `backend/app/services/plan_engine.py`. The pure part is
//! what lives here: the matching, the verdict, the cause and the adherence
//! roll-up. Reading the plan and the activities out of the database is the
//! store's job, and this module takes them as plain data.
//!
//! # Matching
//!
//! For each planned session, in date order:
//!
//! 1. a run on the session's own date wins;
//! 2. for a `rest` or `strength` session, a non-run on that date wins (for
//!    `strength`, a sport whose name contains "strength");
//! 3. otherwise a run within ±2 days of the session wins, closest date first
//!    and then closest distance to the target;
//! 4. an activity is claimed by at most one session.
//!
//! # Verdicts
//!
//! `hit`, `missed_pace`, `short`, `missed` and `substituted`. A `rest` session
//! with no activity is a `hit`: a logged absence is the intended outcome of a
//! rest prescription.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::model::{Activity, DailyPhysiology, MatchedActivity, PlanSession};
use crate::numeric::{mean, round_to, sum};
use crate::ordered::OrderedMap;

/// The sports the *plan engine* counts as running.
///
/// The reference has two sets and they are not the same:
/// `backend/app/models.py` has eight (it includes walking and hiking) and
/// `backend/app/services/analytics.py` has six. The plan engine imports the
/// second one, so a walk does **not** answer a planned run and a walk is not
/// a run for the "rest or strength" rule either.
///
/// `docs/core-contract.md` §7.1 lists the eight-sport set for the plan engine.
/// That is wrong: the reference and the golden fixtures use the six-sport set.
pub const RUN_SPORTS: [&str; 6] = [
    "running",
    "trail_running",
    "treadmill_running",
    "virtualrun",
    "street_running",
    "track_running",
];

/// Whether a sport type counts as running for the plan engine.
pub fn is_run_sport(sport_type: &str) -> bool {
    RUN_SPORTS.contains(&sport_type)
}

/// The verdicts a session can get.
pub const VERDICTS: [&str; 5] = ["hit", "missed_pace", "short", "missed", "substituted"];

/// The verdict on one planned session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verdict {
    /// The verdict: `hit`, `missed_pace`, `short`, `missed` or `substituted`.
    pub verdict: String,
    /// Whether the distance target was met.
    pub distance_hit: bool,
    /// Whether the pace band was met. `None` when the session has no band.
    pub pace_band_hit: Option<bool>,
    /// The distance difference from the target, in metres.
    pub distance_delta_m: Option<f64>,
    /// The pace difference from the middle of the band, in seconds per
    /// kilometre.
    pub pace_delta_s_km: Option<f64>,
    /// The adherence score, out of 100.
    pub adherence_score: Option<f64>,
    /// Why the session went the way it did. `None` for a clean hit.
    pub cause: Option<String>,
    /// How sure the cause is, between 0 and 1.
    pub cause_confidence: Option<f64>,
}

/// One session, the activity that answered it and the verdict.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionMatch {
    /// The planned session.
    pub session: PlanSession,
    /// The activity that answered it, if any.
    pub activity: Option<MatchedActivity>,
    /// The verdict.
    pub verdict: Verdict,
}

/// Everything the plan engine needs, as plain data.
#[derive(Clone, Debug)]
pub struct PlanInputs<'a> {
    /// The first day of the range.
    pub from: &'a str,
    /// The last day of the range.
    pub to: &'a str,
    /// The planned sessions of the range.
    pub sessions: &'a [PlanSession],
    /// The athlete's activities.
    ///
    /// The reference reads them in row-id order and breaks ties with that
    /// order, so the caller should pass them in row-id order; the matching
    /// sorts them by id anyway.
    pub activities: &'a [Activity],
    /// The athlete's physiology rows.
    pub physiology: &'a [DailyPhysiology],
}

/// The physiology a cause is explained against.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CauseInputs {
    /// The day's own physiology row.
    pub day: Option<DailyPhysiology>,
    /// The mean sleep of the trailing thirty days, in seconds.
    pub sleep_baseline_s: Option<f64>,
    /// The mean HRV of the trailing thirty days, in milliseconds.
    pub hrv_baseline_ms: Option<f64>,
    /// The mean resting heart rate of the trailing thirty days.
    pub resting_hr_baseline: Option<f64>,
    /// This run's cadence variability, in percent.
    pub cadence_cv: Option<f64>,
    /// The mean cadence variability of the previous thirty days, in percent.
    pub cadence_cv_baseline: Option<f64>,
}

/// One week of the adherence roll-up.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdherenceRow {
    /// The ISO week, `YYYY-Www`.
    pub week_key: String,
    /// The Monday the week starts on.
    pub start_date: String,
    /// The phase the week belongs to. The reference has no phase join here.
    pub phase: Option<String>,
    /// The planned distance, in kilometres.
    pub planned_km: f64,
    /// The logged distance, in kilometres.
    pub actual_km: f64,
    /// The number of planned sessions.
    pub planned_sessions: usize,
    /// The number of sessions with an activity.
    pub completed_sessions: usize,
    /// The number of sessions with no activity.
    pub missed_sessions: usize,
    /// The number of sessions that hit the distance and the pace band.
    pub hit_sessions: usize,
    /// The difference between the logged and the planned distance, in
    /// kilometres.
    pub delta_km: f64,
    /// The share of sessions that hit, in percent.
    pub adherence_pct: Option<f64>,
}

/// The adherence roll-up over a range.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adherence {
    /// The share of sessions that hit, in percent.
    pub overall_pct: Option<f64>,
    /// The planned distance, in kilometres.
    pub planned_km: f64,
    /// The logged distance, in kilometres.
    pub actual_km: f64,
    /// The number of planned sessions.
    pub planned_sessions: usize,
    /// The number of sessions with an activity.
    pub completed_sessions: usize,
    /// The number of sessions that hit the distance and the pace band.
    pub hit_sessions: usize,
    /// The number of sessions with no activity.
    pub missed_sessions: usize,
    /// The number of run activities that no session claimed.
    pub extra_sessions: usize,
    /// The number of sessions per verdict.
    pub by_verdict: OrderedMap<usize>,
    /// One row per ISO week.
    pub rows: Vec<AdherenceRow>,
}

/// The average pace of an activity, in seconds per kilometre.
///
/// The plan engine's own `_pace`: a zero distance or a zero moving time means
/// there is no pace.
fn pace_of(activity: &Activity) -> Option<f64> {
    activity.pace_s_per_km()
}

/// Whether the activity looks like an interval session.
///
/// Either three laps of at least 400 metres with a positive duration, or a
/// name that says so.
fn looks_interval(activity: &Activity) -> bool {
    let long_laps = activity
        .laps
        .iter()
        .filter(|lap| {
            lap.distance_m.is_some_and(|distance| distance >= 400.0)
                && lap.duration_s.is_some_and(|duration| duration > 0.0)
        })
        .count();
    if long_laps >= 3 {
        return true;
    }
    let name = activity.name.to_lowercase();
    ["interval", "repeats", "fartlek"]
        .iter()
        .any(|needle| name.contains(needle))
}

/// Whether the activity is much longer than the session targeted.
fn looks_long(activity: &Activity, target: f64) -> bool {
    target != 0.0 && activity.distance_m > 14_000.0 && activity.distance_m > target * 1.25
}

/// The verdict on one session.
pub fn verdict_for(session: &PlanSession, activity: Option<&Activity>) -> Verdict {
    let target = session
        .target_distance_m
        .filter(|value| *value != 0.0)
        .unwrap_or(0.0);
    let actual = activity.map(|activity| activity.distance_m);
    let delta = match actual {
        Some(actual) if target != 0.0 => Some(actual - target),
        _ => None,
    };
    let pace = activity.and_then(pace_of);

    let band = if session.pace_low_s_km.is_some() || session.pace_high_s_km.is_some() {
        Some(match pace {
            Some(pace) => {
                session.pace_low_s_km.map_or(true, |low| pace >= low)
                    && session.pace_high_s_km.map_or(true, |high| pace <= high)
            }
            None => false,
        })
    } else {
        None
    };

    let distance_hit = match actual {
        Some(actual) => {
            target == 0.0
                || actual >= target * 0.95
                || (actual - target).abs() <= actual * 0.05
        }
        None => false,
    };

    let verdict = match activity {
        None => "missed",
        Some(activity)
            if (session.session_type == "rest" || session.session_type == "strength")
                && is_run_sport(&activity.sport_type) =>
        {
            "substituted"
        }
        Some(activity) if session.session_type == "tempo" && looks_interval(activity) => {
            "substituted"
        }
        Some(activity) if session.session_type == "interval" && looks_long(activity, target) => {
            "substituted"
        }
        Some(_) if target != 0.0 && actual.is_some_and(|value| value < target * 0.8) => "short",
        _ if distance_hit && band == Some(false) => "missed_pace",
        _ if distance_hit => "hit",
        _ => "short",
    }
    .to_string();

    let distance_score = match actual {
        Some(actual) if target != 0.0 => (actual / target * 100.0).clamp(0.0, 100.0),
        _ => 100.0,
    };
    let pace_score = match band {
        None => 100.0,
        Some(true) => 100.0,
        Some(false) => {
            let edge = session
                .pace_low_s_km
                .filter(|value| *value != 0.0)
                .or_else(|| session.pace_high_s_km.filter(|value| *value != 0.0))
                .unwrap_or(0.0);
            let pace = pace.unwrap_or(0.0);
            (100.0 - ((pace - edge).abs() / 60.0 * 10.0).min(100.0)).max(0.0)
        }
    };
    let type_score = if verdict == "substituted" { 0.0 } else { 100.0 };
    let adherence_score = activity.map(|_| {
        round_to(0.5 * distance_score + 0.4 * pace_score + 0.1 * type_score, 1)
    });

    let pace_delta_s_km = match (pace, session.pace_low_s_km, session.pace_high_s_km) {
        (Some(pace), low, high) => {
            let bounds: Vec<f64> = [low, high].into_iter().flatten().collect();
            if bounds.is_empty() {
                None
            } else {
                Some(pace - mean(&bounds).unwrap_or(0.0))
            }
        }
        _ => None,
    };

    Verdict {
        verdict,
        distance_hit: activity.is_some() && distance_hit,
        pace_band_hit: band,
        distance_delta_m: delta,
        pace_delta_s_km,
        adherence_score,
        cause: None,
        cause_confidence: None,
    }
}

/// Why a session went the way it did.
///
/// A cause is only produced for a session that did not go to plan; a `hit` has
/// no cause at all.
pub fn explain_cause(
    session: &PlanSession,
    activity: Option<&Activity>,
    physiology: Option<&CauseInputs>,
    verdict: &str,
) -> (Option<String>, Option<f64>) {
    if verdict == "hit" {
        return (None, None);
    }
    let activity = match activity {
        Some(activity) => activity,
        None => {
            return (
                Some("No activity was logged for the planned session.".to_string()),
                Some(0.0),
            )
        }
    };
    let physiology = match physiology {
        Some(physiology) => physiology,
        None => {
            return (
                Some(
                    "Physiology data is unavailable, so the reason for this result is unknown."
                        .to_string(),
                ),
                Some(0.0),
            )
        }
    };

    let day = physiology.day.as_ref();
    let metric = |field: &str| -> Option<f64> {
        match day {
            Some(day) => match field {
                "sleep_s" => day.sleep_s,
                "hrv_ms" => day.hrv_ms,
                "resting_hr" => day.resting_hr,
                _ => None,
            },
            None => None,
        }
    };

    let mut factors: Vec<(&str, f64)> = Vec::new();
    let sleep_s = metric("sleep_s");
    if let (Some(sleep_s), Some(baseline)) = (sleep_s, physiology.sleep_baseline_s) {
        if sleep_s < baseline - 3600.0 {
            factors.push(("short sleep", 0.85));
        }
    }
    let hrv = metric("hrv_ms");
    if let (Some(hrv), Some(baseline)) = (hrv, physiology.hrv_baseline_ms) {
        if baseline != 0.0 && hrv < baseline * 0.9 {
            factors.push(("elevated fatigue", 0.7));
        }
    }
    let resting_hr = metric("resting_hr");
    if let (Some(resting_hr), Some(baseline)) = (resting_hr, physiology.resting_hr_baseline) {
        if resting_hr > baseline + 4.0 {
            factors.push(("illness or overreaching", 0.6));
        }
    }
    if let (Some(cv), Some(baseline)) = (physiology.cadence_cv, physiology.cadence_cv_baseline) {
        if cv > baseline {
            factors.push(("mechanical inefficiency", 0.55));
        }
    }
    if session.session_type == "race" {
        if let (Some(pace), Some(low)) = (pace_of(activity), session.pace_low_s_km) {
            let high = session.pace_high_s_km.unwrap_or(low);
            if !(low <= pace && pace <= high) {
                factors.push(("intended race-week load", 0.55));
            }
        }
    }

    if !factors.is_empty() {
        factors.sort_by(|left, right| {
            right
                .1
                .partial_cmp(&left.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top: Vec<&str> = factors.iter().take(2).map(|(name, _)| *name).collect();
        let confidence = factors
            .iter()
            .take(2)
            .map(|(_, confidence)| *confidence)
            .fold(f64::NEG_INFINITY, f64::max);
        return (
            Some(format!("Likely influenced by {}.", top.join(" and "))),
            Some(confidence),
        );
    }

    if physiology.sleep_baseline_s.is_some()
        || physiology.hrv_baseline_ms.is_some()
        || physiology.resting_hr_baseline.is_some()
        || physiology.cadence_cv_baseline.is_some()
    {
        return (
            Some(
                "No measured factor explains the result; it may reflect a fitness gap or an off-plan effort."
                    .to_string(),
            ),
            Some(0.2),
        );
    }

    if metric("sleep_s").is_some() || metric("hrv_ms").is_some() || metric("resting_hr").is_some() {
        return (
            Some(
                "Physiology is available, but there is not enough baseline data to explain this result."
                    .to_string(),
            ),
            Some(0.1),
        );
    }

    (
        Some(
            "Physiology data is unavailable, so the result is unexplained and may reflect fitness or an off-plan effort."
                .to_string(),
        ),
        Some(0.1),
    )
}

/// The days between two ISO dates, in days.
fn day_difference(left: &str, right: &str) -> Option<i64> {
    match (Date::parse(left), Date::parse(right)) {
        (Some(left), Some(right)) => Some(left.days_until(right)),
        _ => None,
    }
}

/// Match the planned sessions of a range against the athlete's activities.
pub fn match_sessions(inputs: &PlanInputs<'_>) -> Vec<SessionMatch> {
    let mut sessions: Vec<&PlanSession> = inputs
        .sessions
        .iter()
        .filter(|session| {
            session.date.as_str() >= inputs.from && session.date.as_str() <= inputs.to
        })
        .collect();
    sessions.sort_by(|left, right| {
        left.date
            .cmp(&right.date)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut activities: Vec<&Activity> = inputs
        .activities
        .iter()
        .filter(|activity| {
            let date = &activity.local_date;
            let from = Date::parse(inputs.from)
                .map(|date| date.add_days(-2).to_string())
                .unwrap_or_else(|| inputs.from.to_string());
            let to = Date::parse(inputs.to)
                .map(|date| date.add_days(2).to_string())
                .unwrap_or_else(|| inputs.to.to_string());
            date.as_str() >= from.as_str() && date.as_str() <= to.as_str()
        })
        .collect();
    // The reference reads the candidates with a range predicate on
    // `local_date`, which SQLite answers through the `local_date` index — so
    // the candidates arrive in date order, and the first of several equally
    // good ones wins. The port reproduces that order.
    activities.sort_by(|left, right| {
        left.local_date
            .cmp(&right.local_date)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut claimed: Vec<i64> = Vec::new();
    let mut matches = Vec::new();

    for session in sessions {
        let target = Date::parse(&session.date);
        let candidates: Vec<&Activity> = activities
            .iter()
            .filter(|activity| !claimed.contains(&activity.id) && !activity.local_date.is_empty())
            .copied()
            .collect();

        let mut chosen: Option<&Activity> = None;

        // 1. A run on the session's own date.
        let same: Vec<&Activity> = candidates
            .iter()
            .filter(|activity| {
                activity.local_date == session.date && is_run_sport(&activity.sport_type)
            })
            .copied()
            .collect();
        if let Some(activity) = min_by(&same, |activity| {
            let days = target
                .and_then(|target| day_difference(activity.start_date(), &target.to_string()))
                .unwrap_or(0);
            days.abs()
        }) {
            chosen = Some(activity);
        }

        // 2. A rest or strength session takes the non-run of that date.
        if chosen.is_none() && (session.session_type == "rest" || session.session_type == "strength")
        {
            let wanted: Vec<&Activity> = candidates
                .iter()
                .filter(|activity| {
                    activity.local_date == session.date
                        && if session.session_type == "strength" {
                            activity.sport_type.to_lowercase().contains("strength")
                        } else {
                            !is_run_sport(&activity.sport_type)
                        }
                }).copied()
                .collect();
            chosen = wanted.first().copied();
        }

        // 3. Otherwise a run within two days.
        if chosen.is_none()
            && session.session_type != "rest"
            && session.session_type != "strength"
        {
            let near: Vec<&Activity> = candidates
                .iter()
                .filter(|activity| {
                    is_run_sport(&activity.sport_type)
                        && day_difference(&activity.local_date, &session.date)
                            .is_some_and(|days| days.abs() <= 2)
                }).copied()
                .collect();
            if let Some(activity) = min_by(&near, |activity| {
                let days = day_difference(&activity.local_date, &session.date).unwrap_or(0).abs();
                let target = session.target_distance_m.unwrap_or(0.0);
                let distance = if target != 0.0 {
                    (activity.distance_m - target).abs()
                } else {
                    0.0
                };
                (days, distance)
            }) {
                chosen = Some(activity);
            }
        }

        if let Some(activity) = chosen {
            claimed.push(activity.id);
        }

        let mut verdict = verdict_for(session, chosen);
        if session.session_type == "rest" && chosen.is_none() {
            // A logged absence is the intended outcome of a rest prescription.
            verdict = Verdict {
                verdict: "hit".to_string(),
                distance_hit: true,
                pace_band_hit: None,
                distance_delta_m: None,
                pace_delta_s_km: None,
                adherence_score: Some(100.0),
                cause: None,
                cause_confidence: None,
            };
        }

        let physiology = cause_inputs(inputs, session, chosen);
        let (cause, confidence) = explain_cause(
            session,
            chosen,
            physiology.as_ref(),
            &verdict.verdict,
        );
        verdict.cause = cause;
        verdict.cause_confidence = confidence;

        matches.push(SessionMatch {
            session: (*session).clone(),
            activity: chosen.map(|activity| MatchedActivity {
                id: activity.id,
                date: activity.local_date.clone(),
                name: activity.name.clone(),
                sport_type: activity.sport_type.clone(),
                distance_m: Some(activity.distance_m),
                pace_s_km: pace_of(activity),
            }),
            verdict,
        });
    }

    matches
}

/// The physiology a cause is explained against, for one session.
fn cause_inputs(
    inputs: &PlanInputs<'_>,
    session: &PlanSession,
    chosen: Option<&Activity>,
) -> Option<CauseInputs> {
    let day = inputs
        .physiology
        .iter()
        .find(|row| row.date == session.date)
        .cloned();

    let window_start = Date::parse(&session.date)
        .map(|date| date.add_days(-30).to_string())
        .unwrap_or_else(|| session.date.clone());
    let history: Vec<&DailyPhysiology> = inputs
        .physiology
        .iter()
        .filter(|row| row.date >= window_start && row.date < session.date)
        .collect();

    let mean_metric = |pick: fn(&DailyPhysiology) -> Option<f64>| -> Option<f64> {
        let values: Vec<f64> = history.iter().filter_map(|row| pick(row)).collect();
        mean(&values)
    };

    let mut cause = CauseInputs {
        day: day.clone(),
        sleep_baseline_s: mean_metric(|row| row.sleep_s),
        hrv_baseline_ms: mean_metric(|row| row.hrv_ms),
        resting_hr_baseline: mean_metric(|row| row.resting_hr),
        cadence_cv: None,
        cadence_cv_baseline: None,
    };

    if let Some(activity) = chosen {
        cause.cadence_cv = activity
            .fingerprint
            .as_ref()
            .and_then(|fingerprint| fingerprint.cadence_cv);
        let prior: Vec<f64> = inputs
            .activities
            .iter()
            .filter(|activity| {
                activity.local_date >= window_start && activity.local_date < session.date
            })
            .filter_map(|activity| activity.fingerprint.as_ref())
            .filter_map(|fingerprint| fingerprint.cadence_cv)
            .collect();
        cause.cadence_cv_baseline = mean(&prior);
    }

    if day.is_some() || cause.cadence_cv.is_some() {
        Some(cause)
    } else {
        None
    }
}

/// The element with the smallest key, the first one winning a tie.
///
/// The reference's `min()` keeps the first of several equal elements, so a
/// tie is not broken by the key order.
fn min_by<'a, T, K: PartialOrd>(items: &'a [T], key: impl Fn(&T) -> K) -> Option<&'a T> {
    let mut best: Option<&'a T> = None;
    for item in items {
        best = match best {
            Some(current) if key(current) <= key(item) => Some(current),
            _ => Some(item),
        };
    }
    best
}

/// The run activities of a range that no session claimed.
pub fn extra_activities<'a>(inputs: &'a PlanInputs<'a>) -> Vec<&'a Activity> {
    let mut sessions: Vec<&PlanSession> = inputs
        .sessions
        .iter()
        .filter(|session| {
            session.date.as_str() >= inputs.from && session.date.as_str() <= inputs.to
        })
        .collect();
    sessions.sort_by(|left, right| left.date.cmp(&right.date).then(left.id.cmp(&right.id)));

    let mut acts: Vec<&Activity> = inputs
        .activities
        .iter()
        .filter(|activity| {
            activity.local_date.as_str() >= inputs.from
                && activity.local_date.as_str() <= inputs.to
                && is_run_sport(&activity.sport_type)
        })
        .collect();
    // The reference reads the bonus activities with a predicate on the sport
    // type, which SQLite answers through the `sport_type` index — so they
    // arrive in sport order. The port reproduces that order, because the
    // first of several equally close activities wins.
    acts.sort_by(|left, right| {
        left.sport_type
            .cmp(&right.sport_type)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut claimed: Vec<i64> = Vec::new();
    for session in &sessions {
        let options: Vec<&Activity> = acts
            .iter()
            .filter(|activity| {
                !claimed.contains(&activity.id)
                    && day_difference(&activity.local_date, &session.date)
                        .is_some_and(|days| days.abs() <= 2)
            })
            .copied()
            .collect();
        if let Some(chosen) = min_by(&options, |activity| {
            day_difference(&activity.local_date, &session.date)
                .unwrap_or(0)
                .abs()
        }) {
            claimed.push(chosen.id);
        }
    }

    acts.into_iter()
        .filter(|activity| !claimed.contains(&activity.id))
        .collect()
}

/// Roll the matched sessions up into the adherence summary.
pub fn adherence(matches: &[SessionMatch], extra_sessions: usize) -> Adherence {
    if matches.is_empty() {
        return Adherence {
            overall_pct: None,
            planned_km: 0.0,
            actual_km: 0.0,
            planned_sessions: 0,
            completed_sessions: 0,
            hit_sessions: 0,
            missed_sessions: 0,
            extra_sessions,
            by_verdict: OrderedMap::new(),
            rows: Vec::new(),
        };
    }

    let mut weeks: Vec<(String, AdherenceRow)> = Vec::new();
    for entry in matches {
        let session = &entry.session;
        let date = match Date::parse(&session.date) {
            Some(date) => date,
            None => continue,
        };
        let week_key = date.iso_week_key();
        let index = match weeks.iter().position(|(key, _)| key == &week_key) {
            Some(index) => index,
            None => {
                weeks.push((
                    week_key.clone(),
                    AdherenceRow {
                        week_key: week_key.clone(),
                        start_date: date.iso_week_start().to_string(),
                        phase: None,
                        planned_km: 0.0,
                        actual_km: 0.0,
                        planned_sessions: 0,
                        completed_sessions: 0,
                        missed_sessions: 0,
                        hit_sessions: 0,
                        delta_km: 0.0,
                        adherence_pct: None,
                    },
                ));
                weeks.len() - 1
            }
        };
        let row = &mut weeks[index].1;
        row.planned_sessions += 1;
        if let Some(target) = session.target_distance_m {
            if target != 0.0 {
                row.planned_km += target / 1000.0;
            }
        }
        match &entry.activity {
            Some(activity) => {
                row.completed_sessions += 1;
                row.actual_km += activity.distance_m.unwrap_or(0.0) / 1000.0;
            }
            None => row.missed_sessions += 1,
        }
        if entry.verdict.verdict == "hit" || entry.verdict.verdict == "missed_pace" {
            row.hit_sessions += 1;
        }
    }

    let rows: Vec<AdherenceRow> = weeks
        .into_iter()
        .map(|(_, mut row)| {
            row.delta_km = round_to(row.actual_km - row.planned_km, 2);
            row.adherence_pct = if row.planned_sessions == 0 {
                None
            } else {
                Some(round_to(
                    row.hit_sessions as f64 / row.planned_sessions as f64 * 100.0,
                    1,
                ))
            };
            row
        })
        .collect();

    let mut by_verdict = OrderedMap::new();
    for verdict in VERDICTS {
        let count = matches
            .iter()
            .filter(|entry| entry.verdict.verdict == verdict)
            .count();
        by_verdict.push(verdict.to_string(), count);
    }

    let hit_or_missed_pace = matches
        .iter()
        .filter(|entry| entry.verdict.verdict == "hit" || entry.verdict.verdict == "missed_pace")
        .count();

    Adherence {
        overall_pct: Some(round_to(
            hit_or_missed_pace as f64 / matches.len() as f64 * 100.0,
            1,
        )),
        planned_km: round_to(
            sum(
                &rows
                    .iter()
                    .map(|row| row.planned_km)
                    .collect::<Vec<_>>(),
            ),
            2,
        ),
        actual_km: round_to(
            sum(
                &rows
                    .iter()
                    .map(|row| row.actual_km)
                    .collect::<Vec<_>>(),
            ),
            2,
        ),
        planned_sessions: matches.len(),
        completed_sessions: matches
            .iter()
            .filter(|entry| entry.activity.is_some())
            .count(),
        hit_sessions: hit_or_missed_pace,
        missed_sessions: matches
            .iter()
            .filter(|entry| entry.verdict.verdict == "missed")
            .count(),
        extra_sessions,
        by_verdict,
        rows,
    }
}

/// Match the sessions of a range and roll them up in one go.
pub fn adherence_for(inputs: &PlanInputs<'_>) -> Adherence {
    let matches = match_sessions(inputs);
    let extra = extra_activities(inputs).len();
    adherence(&matches, extra)
}
