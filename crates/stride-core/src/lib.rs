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

pub mod comparison;
pub mod date;
pub mod fingerprint;
pub mod model;
pub mod numeric;
pub mod ordered;
pub mod plan;
pub mod readiness;

pub use comparison::{
    compare, profile_for, Comparison, Parameter, ParameterComparison, ProfilePoint, Winner,
    PROFILE_POINTS,
};
pub use date::Date;
pub use fingerprint::{athlete_zones, fingerprint, max_hr_reference, zone_model, FingerprintInput};
pub use model::{
    Activity, DailyPhysiology, Fingerprint, Lap, MatchedActivity, PlanSession, Stream, Zone,
    ZoneTime,
};
pub use ordered::OrderedMap;
pub use plan::{
    adherence, explain_cause, is_run_sport, match_sessions, verdict_for, Adherence, AdherenceRow,
    CauseInputs, PlanInputs, SessionMatch, Verdict, RUN_SPORTS as PLAN_RUN_SPORTS,
};
pub use readiness::{baselines, readiness, Baselines, Readiness, ReadinessDriver};
