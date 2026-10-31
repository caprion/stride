//! Stride v2 — the domain core.
//!
//! Pure by contract: **no I/O, no clock, no randomness**. Every
//! time-dependent value — the day a reading belongs to, the history a band is
//! drawn from — is injected by the caller. That is what makes the golden
//! corpus reproducible and the tests meaningful.
//!
//! * [`units`] — the contract's units, its rounding table and the human
//!   rendering ("6 h 10 m", not "6.166 h"). Every rounding helper in the
//!   crate lives here.
//! * [`types`] — the domain types: [`types::Distance`], [`types::Duration`],
//!   [`types::HeartRate`], [`types::Hrv`], [`types::Sleep`],
//!   [`types::RestingHr`], [`types::Effort`]. "No value" is `None`, never `0`.
//! * [`date`] — a civil date, with no clock and no timezone.
//! * [`band`] — "usual": the median of the last 30 nights, and the band around
//!   it, median ± 1.5 × MAD.
//!
//! The voice, in one line: facts first, no judgements. The app informs; it
//! never instructs.

pub mod band;
pub mod date;
pub mod types;
pub mod units;

pub use band::{band_for, Band};
pub use date::Date;
pub use types::{
    BetterSide, Distance, Duration, Effort, HeartRate, Hrv, RestingHr, Sleep, VitalKind, Vitals,
    WellnessDay,
};
