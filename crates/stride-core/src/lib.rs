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
//! * [`strings`] — the typed string catalogue and the lint that holds it to
//!   `docs/contract/voice.md`.
//! * [`rest`] — the "Rest" line: a count and a direction, never a verdict.
//! * [`direction`] — the engine: a vital and its band become one of exactly
//!   three words, with the arithmetic and the direction of travel beside it.
//! * [`eddington`] — the Eddington number: consistency over peaks, counted
//!   from the daily distance in kilometres or miles.
//! * [`pattern`] — what usually happened, with its sample size, never from
//!   fewer than three comparable days.
//! * [`plan`] — the plan engine: the inferred baseline, the week, the
//!   adherence, the results and the distance comparison.
//! * [`statement`] — the sixth kind: a factual sentence about the data — the
//!   distance comparison, the Progress headline, the Compare basis.
//! * [`story`] — the one sentence every run carries: descriptive, never
//!   evaluative, and read from the same primitives the charts read.
//! * [`zones`] — the heart-rate zone model and the time-in-zone computation,
//!   from the athlete's stored configuration, never a run's observed maximum.
//! * [`fingerprint`] — the run's own mechanics: the cadence, the pace and
//!   the heart rate as they moved across the run, and the shares of the
//!   moving time in the athlete's own zones. Observations, never a score.

//! * [`streaks`] — the streaks: the current run of days (or weeks) and the
//!   longest one, with the days they cover.
//! * [`rolling`] — the rolling view: a window of days against the window
//!   before it, the split of the training across the athlete's zones, the
//!   weekday and daytime splits, the distance breakdown, and the monthly
//!   and yearly tables.
//! * [`profile`] — the athlete's shape over a window: seven facets, as
//!   values with a direction of travel, never as a shape.

//! * [`heatmap`] — the grid over the stored positions: a count of runs per
//!   cell, computed on the device, never a route.
//! * [`load`] — the training load: the daily series, the acute and chronic
//!   loads, the form, the ratio, and the recovery scenario — a scenario
//!   with its assumption attached, never a verdict.
//!
//! The voice, in one line: facts first, no judgements. The app informs; it
//! never instructs.

pub mod band;
pub mod date;
pub mod direction;
pub mod eddington;
pub mod fingerprint;

pub mod heatmap;
pub mod load;
pub mod pattern;
pub mod plan;
pub mod profile;
pub mod rest;
pub mod rolling;
pub mod statement;
pub mod story;
pub mod streaks;
pub mod strings;
pub mod types;
pub mod units;
pub mod zones;

pub use band::{band_for, Band};
pub use date::Date;
pub use direction::{day_directions, DayDirections, Direction, Travel, VitalDirection};
pub use eddington::{
    days_needed_for_next, eddington, DailyDistance, Eddington, EddingtonUnit, Period,
};
pub use fingerprint::{
    fingerprint, running_only, Fingerprint, FingerprintInput, FingerprintZoneModel, LapReading,
    NoValue, Observation, ZoneDistribution, ZoneShare, MAX_STRIDE_LENGTH_M, WALKING_CADENCE_SPM,
    WALKING_SPEED_MPS,
};

pub use heatmap::{bounds_of, cell_of, cell_size_m, heatmap, Bounds, Cell, Heatmap, HeatmapCell};
pub use load::{
    load_report, recovery_scenario, ActivityLoad, DailyLoad, DayInput, LoadAnswer, LoadAssumptions,
    LoadInputs, LoadReport, LoadState, LoadZoneModel, NoLoadReason, ProjectedDay, RecoveryScenario,
    RestingHrReading, WeekLoad, BANISTER_K,
};
pub use pattern::{patterns, HistoryDay, Pattern, PatternKind};
pub use plan::{
    adherence_of, adherence_of_at, baseline_from_history, baseline_from_history_at,
    baseline_not_enough_history, day_label, distance_comparison, results_of, results_of_at,
    session_label, source_banner, today_of, week_of, week_start, Adherence, PlanBlock, PlanSession,
    PlanSource, RunFact, SessionKind, SessionResult, SessionStatus,
};
pub use profile::{facets, Facet, FacetKey, FacetUnit};
pub use rest::RestSummary;
pub use rolling::{
    compare_windows, daytime_split, distance_breakdown, monthly_totals, polarised_shares,
    totals_in, weekday_split, yearly_totals, Daytime, DaytimeCuts, DaytimeTotals, DistanceBucket,
    Movement, PeriodTotals, PolarisedZone, RollingActivity, RollingWindow, WeekdayTotals,
    WindowComparison, WindowTotals, ZoneTimes,
};
pub use statement::{
    compare_basis, distance_comparison_line, progress_headline, CompareBasis, PaceAtHr,
    ProgressHeadline,
};
pub use story::{story, RunFacts, Split};
pub use streaks::{streaks, Streak, Streaks};
pub use strings::StringKind;
pub use types::{
    BetterSide, Distance, Duration, Effort, HeartRate, Hrv, RestingHr, Sleep, VitalKind, Vitals,
    WellnessDay,
};
pub use units::{
    render_cadence, render_difference, render_distance, render_duration, render_efficiency,
    render_elevation, render_heart_rate, render_heart_rate_range, render_heart_rate_range_edges,
    render_length_cm, render_pace, render_percent, render_share, render_version, render_zone_label,
    ValueUnit,
};
pub use zones::{model_source, time_in_zone, zone_edges, TimeInZone, ZoneFloors, ZoneSample};
