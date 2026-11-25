//! Stride v2 — the domain core.
//!
//! Pure by contract: **no I/O, no clock, no randomness**. Every
//! time-dependent value — the day a reading belongs to, the history a band is
//! drawn from — is injected by the caller. That is what makes the golden
//! corpus reproducible and the tests meaningful.
//!

pub mod band;

pub mod challenges;
pub mod date;
pub mod direction;
pub mod eddington;
pub mod fingerprint;

pub mod heatmap;
pub mod heart_rates;
pub mod load;
pub mod movement;
pub mod pace;
pub mod pattern;
pub mod plan;
pub mod plan_bases;
pub mod plan_week;
pub mod profile;
pub mod rest;
pub mod rolling;
pub mod statement;
pub mod stimulus;
pub mod story;
pub mod streaks;
pub mod strings;
pub mod threshold;
pub mod training_paces;
pub mod types;
pub mod units;
pub mod vo2;
pub mod year;

pub mod update;
pub mod zones;

pub use band::{band_for, Band};
pub use challenges::{
    challenge_grid, challenge_met, streak_of, Challenge, ChallengeKind, ChallengeWeek, WeekMeasure,
};
pub use date::Date;
pub use direction::{day_directions, DayDirections, Direction, Travel, VitalDirection};
pub use eddington::{
    days_needed_for_next, eddington, DailyDistance, Eddington, EddingtonUnit, Period,
};
pub use fingerprint::{
    fingerprint, running_only, Fingerprint, FingerprintInput, FingerprintZoneModel, LapReading,
    NoValue, Observation, ZoneDistribution, ZoneShare, MAX_STRIDE_LENGTH_M,
};

pub use heatmap::{bounds_of, cell_of, cell_size_m, heatmap, Bounds, Cell, Heatmap, HeatmapCell};
pub use heart_rates::{
    compare, derive, history, in_force, select, source_line, Basis, BandRange, Bands,
    BoundaryDifference, Comparison, DeclaredRate, Declaration, Derived, HeartRateKind,
    HeartRateSource, MissingReason, SelectedBands, Supported, WatchBands, LADDER,
};
pub use load::{
    load_report, recovery_scenario, ActivityLoad, DailyLoad, DayInput, LoadAnswer, LoadAssumptions,
    LoadInputs, LoadReport, LoadState, LoadZoneModel, NoLoadReason, ProjectedDay, RecoveryScenario,
    RestingHrReading, WeekLoad, BANISTER_K,
};
pub use pace::{pace_bands, PaceBand, PaceBands, PacePoint, PaceRun, BAND_WIDTH_S_PER_KM};
pub use pattern::{patterns, HistoryDay, Pattern, PatternKind};
pub use plan::{
    adherence_of, adherence_of_at, baseline_from_history, baseline_from_history_at,
    baseline_not_enough_history, day_label, distance_comparison, results_of, results_of_at,
    session_label, source_banner, today_of, week_of, week_start, Adherence, PlanBase, PlanBlock,
    PlanSession, PlanSource, RunFact, SessionKind, SessionResult, SessionStatus,
};
pub use plan_bases::{
    adopt, catalogue, programme, programmes, read_week, read_week_at, AdoptedPlan, AdoptedSession,
    AdoptedWeek, AdoptionInput, AdoptionRefusal, Catalogue, DateWords, EntryReading, GoalRace,
    MileageRange, Pacing, PacingReading, PacingSource, PlanReading, Programme, RaceOption,
    RaceWeekShift, SessionReading, TemplateDay, TemplateWeek, WeekReading, CATALOGUE_VERSION,
};
pub use plan_week::{
    compare_week, SessionComparison, WeekComparison, WeekFact, WeekState, WeekTotals, WeekWindow,
};
pub use profile::{facets, Facet, FacetKey, FacetUnit};
pub use rest::RestSummary;
pub use rolling::{
    compare_windows, daytime_split, distance_breakdown, monthly_totals, polarised_shares,
    totals_in, weekday_split, yearly_totals, Daytime, DaytimeCuts, DaytimeTotals, DistanceBucket,
    Movement, PeriodTotals, PolarisedZone, RollingActivity, WeekdayTotals, Window,
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
pub use update::{
    check_draft, payload, Check, Fact, Finding, ObservationFact, ObservationInput, Payload,
    Quantity, Report, SplitFact, SplitInput, UpdateInput, ZoneFact, ZoneInput, VERDICTS,
};
pub use year::{
    days_of, months_of, totals_of, weeks_of, what_changed, year_window, DayFacts, MonthFacts,
    WeekFacts, YearRun, YearTotals, YEAR_MONTHS,
};
pub use zones::{
    model_source, models_source, time_in_zone, zone_edges, TimeInZone, ZoneFloors, ZoneSample,
};
