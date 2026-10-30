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
pub const VERDICTS: [&str; 6] = [
    "hit",
    "missed_pace",
    "short",
    "missed",
    "substituted",
    "long",
];

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
    /// The day the plan is judged against, `YYYY-MM-DD`.
    ///
    /// A session dated after it has not happened yet: it is neither a hit
    /// nor a miss, and it is not evidence of anything.
    pub today: &'a str,
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

/// One row of the plan response: a session, the activity that answered it and
/// the verdict.
///
/// The reference serves the session and the activity as nested objects and the
/// verdict's fields beside them. This is that shape, in that order, so the
/// router can serialise it without reshaping anything.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionRow {
    /// The planned session.
    pub session: PlanSession,
    /// The activity that answered it, if any.
    pub activity: Option<MatchedActivity>,
    /// The verdict.
    pub verdict: String,
    /// Whether the distance target was met.
    pub distance_hit: bool,
    /// Whether the pace band was met.
    pub pace_band_hit: Option<bool>,
    /// The distance difference from the target, in metres.
    pub distance_delta_m: Option<f64>,
    /// The pace difference from the middle of the band.
    pub pace_delta_s_km: Option<f64>,
    /// The adherence score, out of 100.
    pub adherence_score: Option<f64>,
    /// Why the session went the way it did.
    pub cause: Option<String>,
    /// How sure the cause is.
    pub cause_confidence: Option<f64>,
}

impl From<&SessionMatch> for SessionRow {
    fn from(entry: &SessionMatch) -> Self {
        Self {
            session: entry.session.clone(),
            activity: entry.activity.clone(),
            verdict: entry.verdict.verdict.clone(),
            distance_hit: entry.verdict.distance_hit,
            pace_band_hit: entry.verdict.pace_band_hit,
            distance_delta_m: entry.verdict.distance_delta_m,
            pace_delta_s_km: entry.verdict.pace_delta_s_km,
            adherence_score: entry.verdict.adherence_score,
            cause: entry.verdict.cause.clone(),
            cause_confidence: entry.verdict.cause_confidence,
        }
    }
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
/// Laps alone are not evidence: a watch's default auto-lap splits every
/// steady run into kilometre laps. What marks intervals is the *pace*
/// alternating. Over the laps of 200 metres or more, the spread between the
/// fastest and slowest lap is at least 15% of the median lap, and splitting
/// at the middle of that spread, there are at least two fast laps, two slow
/// laps and three switches between them (a warm-up and a cool-down around a
/// steady run make only two). A name that says so also counts.
fn looks_interval(activity: &Activity) -> bool {
    let name = activity.name.to_lowercase();
    if ["interval", "repeats", "fartlek"]
        .iter()
        .any(|needle| name.contains(needle))
    {
        return true;
    }
    let paces: Vec<f64> = activity
        .laps
        .iter()
        .filter_map(|lap| match (lap.distance_m, lap.duration_s) {
            (Some(distance), Some(duration)) if distance >= 200.0 && duration > 0.0 => {
                Some(duration / (distance / 1000.0))
            }
            _ => None,
        })
        .collect();
    if paces.len() < 4 {
        return false;
    }
    let Some(mid) = crate::numeric::median(&paces) else {
        return false;
    };
    let low = paces.iter().copied().fold(f64::INFINITY, f64::min);
    let high = paces.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if mid <= 0.0 || (high - low) / mid < 0.15 {
        return false;
    }
    let split = (low + high) / 2.0;
    let fast: Vec<bool> = paces.iter().map(|pace| *pace < split).collect();
    let fast_laps = fast.iter().filter(|is_fast| **is_fast).count();
    let slow_laps = fast.len() - fast_laps;
    let switches = fast.windows(2).filter(|pair| pair[0] != pair[1]).count();
    fast_laps >= 2 && slow_laps >= 2 && switches >= 3
}

/// Has this session not happened yet?
///
/// The reference: the session's date against the day the plan is judged
/// against.
fn is_future(session: &PlanSession, today: &str) -> bool {
    match (Date::parse(&session.date), Date::parse(today)) {
        (Some(date), Some(today)) => date > today,
        // An unparseable day is not evidence of anything: it is treated as
        // due, exactly as the reference treats a date it cannot read.
        _ => false,
    }
}

/// Whether the activity is much longer than the session targeted.
fn looks_long(activity: &Activity, target: f64) -> bool {
    target != 0.0 && activity.distance_m > 14_000.0 && activity.distance_m > target * 1.25
}

/// The verdict of a session that has not happened yet.
///
/// It is neither a hit nor a miss, it does not count in the adherence
/// denominator, and it is not evidence of anything.
pub const NOT_YET_DUE: &str = "not_yet";

/// The verdict on one session.
///
/// `today` is the day the plan is judged against. A session dated after it
/// has not happened yet: it is neither a hit nor a miss, it does not count in
/// the adherence denominator, and it is not evidence of anything. Reporting it
/// as `missed` says the athlete failed to do something they have not been
/// given the chance to do yet.
pub fn verdict_for(session: &PlanSession, activity: Option<&Activity>, today: &str) -> Verdict {
    if is_future(session, today) {
        return Verdict {
            verdict: NOT_YET_DUE.to_string(),
            distance_hit: false,
            pace_band_hit: None,
            distance_delta_m: None,
            pace_delta_s_km: None,
            adherence_score: None,
            cause: None,
            cause_confidence: None,
        };
    }
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
            target == 0.0 || actual >= target * 0.95 || (actual - target).abs() <= actual * 0.05
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
        // Far longer than planned is its own verdict, not a hit: a 9.6 km run
        // against a 3 km shakeout did not answer the session.
        Some(_)
            if target != 0.0
                && session.session_type != "long"
                && actual.is_some_and(|value| value > target * 1.3 && value - target >= 2000.0) =>
        {
            "long"
        }
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
        round_to(
            0.5 * distance_score + 0.4 * pace_score + 0.1 * type_score,
            1,
        )
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

/// The verdict on one session, chosen by the kind of plan it belongs to.
///
/// A **prescribed** plan — one somebody wrote down — is judged `hit`,
/// `missed` and the rest. A **baseline** — a plan inferred from the
/// athlete's own habits — is judged `less_than_usual`, `more_than_usual` and
/// `on_a_usual_day`, and it never says `missed`: nobody wrote it down, so
/// nothing was missed.
///
/// A session that has not happened yet is `not_yet` either way.
pub fn verdict_for_source(
    session: &PlanSession,
    activity: Option<&Activity>,
    today: &str,
    prescribed: bool,
) -> Verdict {
    if is_future(session, today) {
        return verdict_for(session, activity, today);
    }
    if prescribed {
        return verdict_for(session, activity, today);
    }
    let baseline = crate::plan_sources::baseline_verdict(session, activity);
    Verdict {
        verdict: baseline.verdict,
        distance_hit: false,
        pace_band_hit: None,
        distance_delta_m: baseline.delta_m,
        pace_delta_s_km: baseline.pace_delta_s_km,
        adherence_score: None,
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
    if verdict == "hit" || verdict == "not_yet" {
        // A hit needs no explanation, and a session that has not happened yet
        // has nothing to explain: it is not evidence of anything.
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
    if verdict == "long" {
        // Nothing physiological explains running further than planned; the
        // point worth making is the distance itself.
        return (
            Some("Ran well past the planned distance; keep the next few days easy.".to_string()),
            Some(0.0),
        );
    }
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
            Some("No measured factor explains this one.".to_string()),
            Some(0.0),
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
        if chosen.is_none()
            && (session.session_type == "rest" || session.session_type == "strength")
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
                })
                .copied()
                .collect();
            chosen = wanted.first().copied();
        }

        // 3. Otherwise a run within two days.
        if chosen.is_none() && session.session_type != "rest" && session.session_type != "strength"
        {
            let near: Vec<&Activity> = candidates
                .iter()
                .filter(|activity| {
                    is_run_sport(&activity.sport_type)
                        && day_difference(&activity.local_date, &session.date)
                            .is_some_and(|days| days.abs() <= 2)
                })
                .copied()
                .collect();
            if let Some(activity) = min_by(&near, |activity| {
                let days = day_difference(&activity.local_date, &session.date)
                    .unwrap_or(0)
                    .abs();
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

        let mut verdict = verdict_for(session, chosen, inputs.today);
        if session.session_type == "rest" && chosen.is_none() && verdict.verdict != "not_yet" {
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
        let (cause, confidence) =
            explain_cause(session, chosen, physiology.as_ref(), &verdict.verdict);
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
            // The reference answers an empty range with no extra sessions
            // either, and the two must agree.
            extra_sessions: 0,
            by_verdict: OrderedMap::new(),
            rows: Vec::new(),
        };
    }

    let mut weeks: Vec<(String, AdherenceRow)> = Vec::new();
    for entry in matches {
        // A session that has not happened yet is not evidence of anything: it
        // does not count as planned, as completed, as missed or as hit.
        if entry.verdict.verdict == "not_yet" {
            continue;
        }
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
                    row.completed_sessions as f64 / row.planned_sessions as f64 * 100.0,
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
    // The sessions that were due: the ones that have happened, or should
    // have. A session that has not happened yet is not in the denominator.
    let due = matches
        .iter()
        .filter(|entry| entry.verdict.verdict != "not_yet")
        .count();
    if due == 0 {
        // Nothing was due: there is no adherence to report, and a percentage
        // of nothing is not a number.
        return Adherence {
            overall_pct: None,
            planned_km: 0.0,
            actual_km: 0.0,
            planned_sessions: 0,
            completed_sessions: 0,
            hit_sessions: 0,
            missed_sessions: 0,
            extra_sessions: 0,
            by_verdict: OrderedMap::new(),
            rows: Vec::new(),
        };
    }

    // Adherence is showing up: the share of due sessions that got a run. How
    // each run went is the verdict's job, not this percentage's.
    let completed_due = matches
        .iter()
        .filter(|entry| entry.verdict.verdict != "not_yet" && entry.activity.is_some())
        .count();
    Adherence {
        overall_pct: Some(round_to(completed_due as f64 / due as f64 * 100.0, 1)),
        planned_km: round_to(
            sum(&rows.iter().map(|row| row.planned_km).collect::<Vec<_>>()),
            2,
        ),
        actual_km: round_to(
            sum(&rows.iter().map(|row| row.actual_km).collect::<Vec<_>>()),
            2,
        ),
        planned_sessions: due,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Lap;

    fn session(session_type: &str, target: Option<f64>) -> PlanSession {
        PlanSession {
            id: 1,
            date: "2026-03-02".to_string(),
            session_type: session_type.to_string(),
            title: None,
            details: None,
            target_distance_m: target,
            pace_low_s_km: None,
            pace_high_s_km: None,
        }
    }

    fn activity(id: i64, sport: &str, distance: f64, day: &str) -> Activity {
        Activity {
            id,
            name: format!("run {id}"),
            local_date: day.to_string(),
            start_date: day.to_string(),
            sport_type: sport.to_string(),
            source: "garmin".to_string(),
            distance_m: distance,
            moving_time_s: Some(distance / 3.0),
            ..Activity::default()
        }
    }

    /// A session that has not happened yet is neither a hit nor a miss.
    #[test]
    fn a_session_that_has_not_happened_yet_is_not_yet() {
        let mut session = session("easy", Some(1000.0));
        session.date = "2026-03-20".to_string();

        let verdict = verdict_for(&session, None, "2026-03-19");
        assert_eq!(verdict.verdict, NOT_YET_DUE);
        assert!(verdict.verdict != "hit");
        assert!(verdict.verdict != "missed");
        assert!(!verdict.distance_hit);
        assert_eq!(verdict.pace_band_hit, None);
        assert_eq!(verdict.distance_delta_m, None);
        assert_eq!(verdict.pace_delta_s_km, None);
        assert_eq!(verdict.adherence_score, None);
        assert_eq!(verdict.cause, None);

        // The same session on the day itself is due, and it is missed.
        let due = verdict_for(&session, None, "2026-03-20");
        assert_eq!(due.verdict, "missed");
    }

    /// The adherence does not count a session that has not happened yet.
    #[test]
    fn the_adherence_does_not_count_a_session_that_has_not_happened_yet() {
        let mut past = session("easy", Some(1000.0));
        past.date = "2026-03-02".to_string();
        let mut future = session("easy", Some(1000.0));
        future.id = 2;
        future.date = "2026-03-20".to_string();
        let sessions = vec![past, future];
        let activities = vec![activity(1, "running", 1000.0, "2026-03-02")];
        let inputs = PlanInputs {
            today: "2026-03-10",
            from: "2026-03-01",
            to: "2026-03-21",
            sessions: &sessions,
            activities: &activities,
            physiology: &[],
        };

        let matches = match_sessions(&inputs);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].verdict.verdict, "hit");
        assert_eq!(matches[1].verdict.verdict, NOT_YET_DUE);

        let adherence = adherence_for(&inputs);
        assert_eq!(
            adherence.planned_sessions, 1,
            "the session that has not happened is not a planned session yet"
        );
        assert_eq!(adherence.hit_sessions, 1);
        assert_eq!(adherence.missed_sessions, 0);
        assert_eq!(
            adherence.overall_pct,
            Some(100.0),
            "the percentage is of the sessions that were due"
        );
    }

    /// A baseline never says "missed": nobody wrote it down.
    #[test]
    fn a_baseline_never_says_missed() {
        let mut session = session("easy", Some(1000.0));
        session.date = "2026-03-02".to_string();
        let verdict = verdict_for_source(&session, None, "2026-03-10", false);
        assert_eq!(verdict.verdict, "less_than_usual");
        assert!(verdict.verdict != "missed");
        // A prescribed plan, on the same session, says "missed".
        let prescribed = verdict_for_source(&session, None, "2026-03-10", true);
        assert_eq!(prescribed.verdict, "missed");
    }

    #[test]
    fn the_plan_uses_the_analytics_run_sports_not_the_model_ones() {
        // The reference has two sets and they disagree. The plan engine uses
        // the narrow one, so a walk does not answer a planned run.
        assert!(is_run_sport("running"));
        assert!(is_run_sport("treadmill_running"));
        assert!(!is_run_sport("walking"));
        assert!(!is_run_sport("hiking"));
        assert!(crate::model::is_run_sport("walking"));
    }

    #[test]
    fn a_walk_does_not_answer_a_planned_run() {
        let sessions = vec![session("easy", Some(1000.0))];
        let activities = vec![activity(1, "walking", 1000.0, "2026-03-02")];
        let inputs = PlanInputs {
            today: "2026-03-20",
            from: "2026-03-01",
            to: "2026-03-03",
            sessions: &sessions,
            activities: &activities,
            physiology: &[],
        };
        let matches = match_sessions(&inputs);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].verdict.verdict, "missed");
        assert!(matches[0].activity.is_none());
    }

    #[test]
    fn a_rest_session_with_no_activity_is_a_hit() {
        let sessions = vec![session("rest", None)];
        let inputs = PlanInputs {
            today: "2026-03-20",
            from: "2026-03-01",
            to: "2026-03-03",
            sessions: &sessions,
            activities: &[],
            physiology: &[],
        };
        let matches = match_sessions(&inputs);
        assert_eq!(matches[0].verdict.verdict, "hit");
        assert!(matches[0].verdict.distance_hit);
        assert_eq!(matches[0].verdict.pace_band_hit, None);
        assert_eq!(matches[0].verdict.adherence_score, Some(100.0));
    }

    #[test]
    fn a_run_on_a_rest_day_is_a_substitution() {
        let sessions = vec![session("rest", None)];
        let activities = vec![activity(1, "running", 1000.0, "2026-03-02")];
        let inputs = PlanInputs {
            today: "2026-03-20",
            from: "2026-03-01",
            to: "2026-03-03",
            sessions: &sessions,
            activities: &activities,
            physiology: &[],
        };
        let matches = match_sessions(&inputs);
        assert_eq!(matches[0].verdict.verdict, "substituted");
        // The distance and the pace both score 100; only the type scores zero.
        assert_eq!(matches[0].verdict.adherence_score, Some(90.0));
    }

    fn judged(planned: PlanSession, activity: Activity) -> String {
        let inputs = PlanInputs {
            today: "2026-03-20",
            from: "2026-03-01",
            to: "2026-03-03",
            sessions: &[planned],
            activities: &[activity],
            physiology: &[],
        };
        match_sessions(&inputs)[0].verdict.verdict.clone()
    }

    fn laps(paces_s_km: &[f64]) -> Vec<Lap> {
        paces_s_km
            .iter()
            .enumerate()
            .map(|(index, pace)| Lap {
                lap_index: index as i64 + 1,
                distance_m: Some(1000.0),
                duration_s: Some(*pace),
                ..Lap::default()
            })
            .collect()
    }

    #[test]
    fn a_steady_tempo_with_kilometre_auto_laps_is_a_hit() {
        // Every steady run with a default 1 km auto-lap has three or more
        // long laps. That alone must never read as intervals.
        let planned = session("tempo", Some(8000.0));
        let mut activity = activity(1, "running", 8000.0, "2026-03-02");
        activity.laps = laps(&[361.0, 358.0, 359.0, 362.0, 360.0, 357.0, 360.0, 359.0]);
        assert_eq!(judged(planned, activity), "hit");
    }

    #[test]
    fn a_tempo_run_as_alternating_fast_and_slow_laps_is_a_substitution() {
        let planned = session("tempo", Some(8000.0));
        let mut activity = activity(1, "running", 8000.0, "2026-03-02");
        activity.laps = laps(&[400.0, 300.0, 420.0, 300.0, 420.0, 300.0, 420.0, 400.0]);
        assert_eq!(judged(planned, activity), "substituted");
    }

    #[test]
    fn a_run_far_longer_than_a_short_session_is_long_not_hit() {
        let planned = session("easy", Some(3000.0));
        let activity = activity(1, "running", 9600.0, "2026-03-02");
        assert_eq!(judged(planned, activity), "long");
    }

    #[test]
    fn a_long_run_that_goes_further_is_still_a_hit() {
        let planned = session("long", Some(16000.0));
        let activity = activity(1, "running", 22000.0, "2026-03-02");
        assert_eq!(judged(planned, activity), "hit");
    }

    #[test]
    fn the_adherence_roll_up_counts_the_verdicts() {
        let sessions = vec![
            session("easy", Some(1000.0)),
            PlanSession {
                id: 2,
                date: "2026-03-03".to_string(),
                ..session("easy", Some(1000.0))
            },
        ];
        let activities = vec![activity(1, "running", 1000.0, "2026-03-02")];
        let inputs = PlanInputs {
            today: "2026-03-20",
            from: "2026-03-01",
            to: "2026-03-05",
            sessions: &sessions,
            activities: &activities,
            physiology: &[],
        };
        let summary = adherence_for(&inputs);
        assert_eq!(summary.planned_sessions, 2);
        assert_eq!(summary.completed_sessions, 1);
        assert_eq!(summary.overall_pct, Some(50.0));
        assert_eq!(summary.planned_km, 2.0);
        assert_eq!(summary.actual_km, 1.0);
        assert_eq!(summary.rows.len(), 1);
        assert_eq!(summary.rows[0].week_key, "2026-W10");
    }
}

/// The plan week a session belongs to.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WeekTarget {
    /// The week's row id.
    #[serde(default)]
    pub id: i64,
    /// The week's number in its phase.
    #[serde(default)]
    pub week_number: i64,
    /// The week's target distance, in kilometres.
    #[serde(default)]
    pub target_km: Option<f64>,
    /// The Monday the week starts on.
    #[serde(default)]
    pub start_date: Option<String>,
}

/// The week as the plan response reports it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WeekSummary {
    /// The week's row id.
    #[serde(default)]
    pub id: i64,
    /// The week's number in its phase.
    #[serde(default)]
    pub week_number: i64,
    /// The week's target distance, in kilometres.
    #[serde(default)]
    pub target_km: Option<f64>,
}

/// Everything today's recommendation is derived from.
#[derive(Clone, Debug, Default)]
pub struct TodayInput<'a> {
    /// The day, `YYYY-MM-DD`.
    pub day: &'a str,
    /// The session planned for that day, if there is one.
    pub session: Option<&'a PlanSession>,
    /// The week the session belongs to, with the start date the
    /// week-to-date volume is measured from.
    pub week: Option<WeekTarget>,
    /// Garmin's own training readiness for the day.
    pub training_readiness: Option<f64>,
    /// The distance logged in the run activities from the week's start to the
    /// day, in metres.
    pub logged_distance_m: f64,
}

/// What to do today.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TodayPlan {
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The session planned for the day.
    pub session: Option<PlanSession>,
    /// The week the session belongs to.
    pub week: Option<WeekSummary>,
    /// Garmin's own training readiness for the day.
    pub readiness: Option<f64>,
    /// What to do.
    pub recommendation: String,
    /// Why.
    pub rationale: String,
    /// Whether a plan exists for the day.
    pub has_plan: bool,
}

/// Today's recommendation.
///
/// The reference reads the clock and the database; everything it then *decides*
/// is a function of the day, the session, the week's target, the day's
/// training readiness and the distance logged so far. That is what this is:
/// the decision, without the clock and without the database.
///
/// Two signals shorten the session: a training readiness below 40, and a
/// week-to-date running volume that is behind the week's target.
pub fn today_plan(input: &TodayInput<'_>) -> TodayPlan {
    let session = input.session;
    let Some(session) = session else {
        return TodayPlan {
            date: input.day.to_string(),
            session: None,
            week: None,
            readiness: input.training_readiness,
            recommendation:
                "No plan is defined for today; add or import a plan to get a recommendation."
                    .to_string(),
            rationale: "There is no planned session to compare or adjust.".to_string(),
            has_plan: false,
        };
    };

    let low = input
        .training_readiness
        .is_some_and(|readiness| readiness < 40.0);

    let mut behind = false;
    if let Some(week) = &input.week {
        if let (Some(target_km), Some(start_date)) = (week.target_km, week.start_date.as_deref()) {
            if target_km != 0.0 {
                // The days from the week's start to the day, and never less
                // than one.
                let elapsed = crate::date::Date::parse(input.day)
                    .zip(crate::date::Date::parse(start_date))
                    .map(|(day, start)| (1 - day.days_until(start)).max(1))
                    .unwrap_or(1);
                let expected = target_km * (elapsed as f64 / 7.0).min(1.0) * 0.75;
                behind = (input.logged_distance_m / 1000.0) < expected;
            }
        }
    }

    let adjust = low || behind;
    let why = if low {
        "readiness is low"
    } else {
        "this week's running volume is behind target"
    };
    let recommendation = if adjust {
        format!("Shorten today's session or swap quality for an easy run because {why}.")
    } else {
        let title = session
            .title
            .clone()
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| session.session_type.clone());
        format!("Complete the planned {title} session.")
    };

    let mut reasons: Vec<&str> = Vec::new();
    if low {
        reasons.push("readiness is below 40");
    }
    if behind {
        reasons.push("week-to-date running volume is behind its target");
    }
    let rationale = if reasons.is_empty() {
        "No low-readiness or behind-volume signal was available.".to_string()
    } else {
        reasons.join("; ")
    };

    TodayPlan {
        date: input.day.to_string(),
        session: Some(session.clone()),
        week: input.week.as_ref().map(|week| WeekSummary {
            id: week.id,
            week_number: week.week_number,
            target_km: week.target_km,
        }),
        readiness: input.training_readiness,
        recommendation,
        rationale,
        has_plan: true,
    }
}

#[cfg(test)]
mod today_tests {
    use super::*;

    fn session(title: Option<&str>) -> PlanSession {
        PlanSession {
            id: 1,
            date: "2026-03-04".to_string(),
            session_type: "easy".to_string(),
            title: title.map(str::to_string),
            details: None,
            target_distance_m: Some(400.0),
            pace_low_s_km: None,
            pace_high_s_km: None,
        }
    }

    fn week(target_km: Option<f64>, start_date: &str) -> WeekTarget {
        WeekTarget {
            id: 1,
            week_number: 1,
            target_km,
            start_date: Some(start_date.to_string()),
        }
    }

    #[test]
    fn a_day_without_a_plan_says_so() {
        let plan = today_plan(&TodayInput {
            day: "2026-03-04",
            session: None,
            week: None,
            training_readiness: Some(40.0),
            logged_distance_m: 0.0,
        });
        assert!(!plan.has_plan);
        assert_eq!(plan.session, None);
        assert_eq!(
            plan.recommendation,
            "No plan is defined for today; add or import a plan to get a recommendation."
        );
        assert_eq!(
            plan.rationale,
            "There is no planned session to compare or adjust."
        );
        assert_eq!(plan.readiness, Some(40.0));
    }

    #[test]
    fn a_planned_day_is_completed_as_planned() {
        let plan = today_plan(&TodayInput {
            day: "2026-03-04",
            session: Some(&session(Some("Easy 400"))),
            week: Some(week(Some(25.0), "2026-03-02")),
            training_readiness: Some(80.0),
            logged_distance_m: 10_000.0,
        });
        assert!(plan.has_plan);
        assert_eq!(
            plan.recommendation,
            "Complete the planned Easy 400 session."
        );
        assert_eq!(
            plan.rationale,
            "No low-readiness or behind-volume signal was available."
        );
    }

    #[test]
    fn a_readiness_below_forty_shortens_the_session() {
        let plan = today_plan(&TodayInput {
            day: "2026-03-04",
            session: Some(&session(None)),
            week: Some(week(Some(25.0), "2026-03-02")),
            training_readiness: Some(39.0),
            logged_distance_m: 10_000.0,
        });
        assert_eq!(
            plan.recommendation,
            "Shorten today's session or swap quality for an easy run because readiness is low."
        );
        assert_eq!(plan.rationale, "readiness is below 40");
        // Without a title the session's type is used.
        assert!(plan.recommendation.contains("readiness"));
    }

    #[test]
    fn a_week_behind_its_target_shortens_the_session() {
        let plan = today_plan(&TodayInput {
            day: "2026-03-04",
            session: Some(&session(Some("Easy 400"))),
            week: Some(week(Some(40.0), "2026-03-02")),
            training_readiness: Some(80.0),
            logged_distance_m: 1_000.0,
        });
        assert_eq!(
            plan.recommendation,
            "Shorten today's session or swap quality for an easy run because this week's running volume is behind target."
        );
        assert_eq!(
            plan.rationale,
            "week-to-date running volume is behind its target"
        );
    }

    #[test]
    fn both_signals_are_reported_together() {
        let plan = today_plan(&TodayInput {
            day: "2026-03-04",
            session: Some(&session(Some("Easy 400"))),
            week: Some(week(Some(40.0), "2026-03-02")),
            training_readiness: Some(20.0),
            logged_distance_m: 1_000.0,
        });
        assert_eq!(
            plan.rationale,
            "readiness is below 40; week-to-date running volume is behind its target"
        );
    }
}

/// One week of the plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekPlan {
    /// The Monday the week starts on.
    pub start_date: String,
    /// The Sunday the week ends on.
    pub end_date: String,
    /// The sessions of the week.
    pub items: Vec<PlanSession>,
}

/// The plan of the week a day falls in.
///
/// The sessions are the ones of the active block; this keeps the ones of the
/// week and the week's dates.
pub fn week_plan(day: &str, sessions: &[PlanSession]) -> Option<WeekPlan> {
    let date = crate::date::Date::parse(day)?;
    let (start, end) = date.week_range();
    let mut items: Vec<PlanSession> = sessions
        .iter()
        .filter(|session| {
            session.date.as_str() >= start.to_string().as_str()
                && session.date.as_str() <= end.to_string().as_str()
        })
        .cloned()
        .collect();
    // The week strip reads in date order.
    items.sort_by(|left, right| {
        left.date
            .cmp(&right.date)
            .then_with(|| left.id.cmp(&right.id))
    });
    Some(WeekPlan {
        start_date: start.to_string(),
        end_date: end.to_string(),
        items,
    })
}

#[cfg(test)]
mod week_tests {
    use super::*;

    #[test]
    fn a_week_runs_from_monday_to_sunday_and_sorts_the_sessions() {
        let sessions = vec![
            PlanSession {
                id: 2,
                date: "2026-03-08".to_string(),
                session_type: "easy".to_string(),
                ..PlanSession::default()
            },
            PlanSession {
                id: 1,
                date: "2026-03-03".to_string(),
                session_type: "tempo".to_string(),
                ..PlanSession::default()
            },
            PlanSession {
                id: 3,
                date: "2026-03-10".to_string(),
                session_type: "rest".to_string(),
                ..PlanSession::default()
            },
        ];
        let week = week_plan("2026-03-04", &sessions).expect("a week");
        assert_eq!(week.start_date, "2026-03-02");
        assert_eq!(week.end_date, "2026-03-08");
        assert_eq!(week.items.len(), 2);
        assert_eq!(week.items[0].date, "2026-03-03");
        assert_eq!(week.items[1].date, "2026-03-08");
    }
}
