//! Stride v2 — the domain core.
//!
//! Pure by contract: **no I/O, no clock, no randomness**. Every
//! time-dependent value — the day a reading belongs to, the history a band is
//! drawn from — is injected by the caller. That is what makes the golden
//! corpus reproducible and the tests meaningful.
//!
//! The voice, in one line: facts first, no judgements. The app informs; it
//! never instructs.

pub mod date;
pub mod types;
pub mod units;

pub use date::Date;
pub use types::{
    BetterSide, Distance, Duration, Effort, HeartRate, Hrv, RestingHr, Sleep, Vitals, VitalKind,
    WellnessDay,
};
