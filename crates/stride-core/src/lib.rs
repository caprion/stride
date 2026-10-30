//! Stride's domain logic: the running fingerprint, the readiness score, the
//! plan-vs-actual verdicts and the run comparison.
//!
//! Everything in this crate is a **pure function over plain data**. There is no
//! database, no I/O and no HTTP, which is exactly what makes the domain
//! checkable against the golden fixtures without a database behind it.
//!
//! The reference implementation is the Python service under `backend/app/`:
//!
//! * `services/fingerprint.py` — the fingerprint and the zone model
//! * `services/readiness.py`   — the readiness score and its seven drivers
//! * `services/plan_engine.py` — the session matching and the verdicts
//! * `services/comparison.py`  — the fifteen-parameter run comparison
//!
//! The golden files in `backend/tests/golden/golden/` are the authority on what
//! "the same number" means; `docs/core-contract.md` is the authority on units,
//! rounding and the representation of an absent value.
//!
//! # Rounding
//!
//! The reference rounds in a handful of places, and the contract pins each of
//! them. [`numeric::round_to`] reproduces Python's `round()` — correct decimal
//! rounding, ties to even — so the port rounds in the same place with the same
//! result.
//!
//! # Absent values
//!
//! `Option<f64>` is `null`. There is no sentinel: a missing heart rate is
//! `None`, never `0.0`.
//!
//! # What the router needs
//!
//! Every struct here serialises to the reference's shape, with the
//! reference's key order, so `stride-api` can serve the responses without
//! reshaping anything:
//!
//! * [`model::Fingerprint`] is the fingerprint payload;
//! * [`readiness::Readiness`] is the readiness payload;
//! * [`plan::SessionMatch`] is one planned session and its verdict, and
//!   [`plan::Adherence`] is the adherence roll-up;
//! * [`comparison::values_for`] is the comparison's `values` object and
//!   [`comparison::PARAMETERS`] is its parameter matrix.

pub mod comparison;
pub mod date;
pub mod fingerprint;
pub mod insights;
pub mod model;
pub mod numeric;
pub mod ordered;
pub mod plan;
pub mod plan_sources;
pub mod readiness;

pub use comparison::{
    candidates, compare, duration_label, profile_for, run_payload, value_for, values_for,
    Candidate, Comparison, Parameter, ParameterComparison, ProfilePoint, RunPayload, Winner,
    CANDIDATE_SPORTS, PROFILE_POINTS,
};
pub use date::Date;
pub use fingerprint::{
    athlete_zones, default_zone_model, fingerprint, fingerprint_metric, fingerprint_trend,
    max_hr_reference, metric, zone_model, zone_shares, FingerprintInput, FingerprintItem, Metric,
    StoredZone, Trend, TrendInput, TrendPoint, ZoneShare, METRICS,
};
pub use model::{
    Activity, DailyPhysiology, Fingerprint, Lap, MatchedActivity, PlanSession, Stream, Zone,
    ZoneTime,
};
pub use ordered::OrderedMap;
pub use plan::{
    adherence, explain_cause, is_run_sport, match_sessions, today_plan, verdict_for,
    verdict_for_source, week_plan, Adherence, AdherenceRow, CauseInputs, PlanInputs, SessionMatch,
    SessionRow, TodayInput, TodayPlan, Verdict, WeekPlan, WeekSummary, WeekTarget, NOT_YET_DUE,
    RUN_SPORTS as PLAN_RUN_SPORTS,
};
pub use plan_sources::{
    baseline_verdict, infer_plan, normalise_date, session_from_workout, sessions_from_workouts,
    weekday_index, weekday_name, workouts_plan, BaselineVerdict, InferredPlan, ScheduledWorkout,
    WorkoutDetail, WorkoutsPlan,
};
pub use readiness::{
    baselines, readiness, readiness_range, Baselines, CoreError, Readiness, ReadinessDriver,
};
