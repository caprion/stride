//! The ledger — the athlete's own bands, the shape rows, and the two bases.
//!
//! The ledger answers against the athlete's **own recorded history**: the
//! time in their own zones, the longest run, and the repeated work their
//! watch recorded as a workout. It never says what was not done, and it
//! never scores the training (`exploration/stimulus-ledger.md`).
//!
//! ## The kinds and the values
//!
//! * the **kinds** — the athlete's own bands: Z1–Z2 low, Z3 moderate,
//!   Z4–Z5 high. They partition the time the athlete was running;
//! * the **values** — the longest run and the repeated work. They overlap
//!   the kinds and are **never added** to them.
//!
//! ## The two bases (D23)
//!
//! | basis | what it is | measured over |
//! |---|---|---|
//! | the app's own zone time | the time in the athlete's own bands, computed from the run's samples with the model in force that day | the time the athlete was **running** |
//! | the source's own zone minutes | the watch's own record of the run | the **whole run**, stops included |
//!
//! The two are **never added together**, each carries its own coverage, and
//! the **overlap** — the runs where both exist — is where the app can check
//! its own arithmetic against the watch's record. The overlap states the
//! distance between the two measurements and does not say which is right
//! (`exploration/two-bases.md` §2).
//!
//! ## Absence is not zero
//!
//! A band a week held **none** has **no cell** in the weeks matrix, so the
//! absence is drawable as white space. A band with no time in the period but
//! a measured period is a **measured zero** — 0 m, with no date. A period
//! that could not be measured at all is a **no value with a reason**.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::rolling::{PolarisedZone, Window, ZoneTimes};
use crate::strings;
use crate::training_paces::Rule;
use crate::zones::TimeInZone;

/// The rule the kinds are computed by: the zone model in force on the run's
/// day, measured over the time the athlete was running.
pub const RULE_ZONES: Rule = Rule {
    id: "zones.model_in_force",
    citation: "the zone model in force on the run's day",
    string_id: "stimulus.rule.zones",
};

/// The rule the source's own rows are computed by.
pub const RULE_SOURCE_ZONES: Rule = Rule {
    id: "source.zone_minutes",
    citation: "the watch's own record",
    string_id: "stimulus.rule.source_zones",
};

/// The rule the longest run is computed by.
pub const RULE_LONGEST: Rule = Rule {
    id: "ledger.longest_run",
    citation: "the run's own moving time",
    string_id: "stimulus.rule.longest",
};

/// The rule the repeated-work row is computed by.
pub const RULE_REPEATED: Rule = Rule {
    id: "ledger.workout_steps",
    citation: "the watch's own workout steps",
    string_id: "stimulus.rule.repeated",
};

/// The source's own zone minutes: the watch's own record of a run.
///
/// The watch's zones carry **no boundaries** here — the app does not know
/// what heart rate was "zone 3" on the day of that run — so these rows are
/// the source's own numbers and are never rolled up into the athlete's
/// bands.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceZones {
    /// The seconds the source reported in each of its five zones. `None` for
    /// a zone the source did not report.
    pub zone_s: [Option<f64>; 5],
}

impl SourceZones {
    /// The source's own zone minutes, as a whole.
    pub fn total_s(&self) -> f64 {
        self.zone_s
            .iter()
            .filter_map(|seconds| *seconds)
            .filter(|seconds| seconds.is_finite())
            .sum()
    }
}

/// A step of the watch's own workout.
///
/// The names are the watch's own (the FIT lap field `wkt_step_name`, and the
/// `workout_step` message), and the app does not rename them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkoutStep {
    /// A warm-up.
    Warmup,
    /// A cool-down.
    Cooldown,
    /// A recovery step.
    Recovery,
    /// An interval step.
    Interval,
    /// A rest step.
    Rest,
    /// A work step.
    Work,
    /// A step the watch named itself.
    Custom,
}

impl WorkoutStep {
    /// The step from the FIT enum's number.
    ///
    /// The FIT lap field `wkt_step_name` is an enum: 0 warmup, 1 cooldown,
    /// 2 recovery, 3 interval, 4 rest, 5 work, 6 custom.
    pub fn from_fit_code(code: u8) -> Option<WorkoutStep> {
        match code {
            0 => Some(WorkoutStep::Warmup),
            1 => Some(WorkoutStep::Cooldown),
            2 => Some(WorkoutStep::Recovery),
            3 => Some(WorkoutStep::Interval),
            4 => Some(WorkoutStep::Rest),
            5 => Some(WorkoutStep::Work),
            6 => Some(WorkoutStep::Custom),
            _ => None,
        }
    }

    /// The step's name, as the watch writes it.
    pub fn name(&self) -> &'static str {
        match self {
            WorkoutStep::Warmup => "warmup",
            WorkoutStep::Cooldown => "cooldown",
            WorkoutStep::Recovery => "recovery",
            WorkoutStep::Interval => "interval",
            WorkoutStep::Rest => "rest",
            WorkoutStep::Work => "work",
            WorkoutStep::Custom => "custom",
        }
    }
}

/// One run, as the ledger reads it.
///
/// The two bases arrive separately and are never merged: `zone_time` is the
/// app's own computation over the samples, `source_zones` is the watch's own
/// record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerRun {
    /// The day the run belongs to.
    pub date: Date,
    /// The run's distance, in metres.
    pub distance_m: Option<f64>,
    /// How long the run moved, in seconds.
    pub moving_time_s: Option<f64>,
    /// How long the whole run took, in seconds — the basis of the source's
    /// own zone minutes.
    pub elapsed_time_s: Option<f64>,
    /// The app's own zone time — over the time the athlete was running.
    ///
    /// `None` when no zone model was in force on the run's day: the run
    /// cannot be measured, and that is not a zero.
    pub zone_time: Option<TimeInZone>,
    /// The source's own zone minutes — over the whole run.
    pub source_zones: Option<SourceZones>,
    /// Whether the run carried heart-rate samples at all.
    pub has_heart_rate: bool,
    /// The watch's own workout steps, when the file carried them.
    pub workout_steps: Option<Vec<WorkoutStep>>,
}

impl LedgerRun {
    /// A run with nothing measured on it.
    pub fn new(date: Date) -> Self {
        Self {
            date,
            distance_m: None,
            moving_time_s: None,
            elapsed_time_s: None,
            zone_time: None,
            source_zones: None,
            has_heart_rate: false,
            workout_steps: None,
        }
    }

    /// Whether the watch's own workout steps make this run **repeated
    /// work**.
    ///
    /// The rule is the watch's own statement about its own session: **two or
    /// more of the run's steps are named `interval`**. The app adds no
    /// threshold, and the lap-shape heuristic is **not** used: auto-laps are
    /// equal by construction, so a shape rule cannot tell a workout from a
    /// run the watch simply lapped
    /// (`exploration/stimulus-ledger.md` §1.2, §7).
    pub fn is_repeated_work(&self) -> bool {
        match &self.workout_steps {
            Some(steps) => {
                steps
                    .iter()
                    .filter(|step| **step == WorkoutStep::Interval)
                    .count()
                    >= 2
            }
            None => false,
        }
    }
}

/// An input the ledger is missing, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingInput {
    /// No run was recorded in the period.
    NoRuns,
    /// No run in the period carried heart-rate samples.
    NoHeartRate,
    /// No zone model has been read at all.
    NoZoneModel,
    /// Zone models have been read, but **none of them was in force for the
    /// period's runs**: the runs predate the first stored model.
    ///
    /// This is the case the two bases exist for: the app's own bands cannot
    /// be measured over a period the model does not cover, and the watch's
    /// own zone minutes are the only zone time there is.
    NoZoneModelInForce,
    /// A zone model was in force for the period's runs, but the run carries
    /// no heart rate: there is nothing to measure the bands from.
    NoHeartRateInRun,
    /// No run in the period carries the watch's own workout steps.
    NoWorkoutSteps,
}

impl MissingInput {
    /// The reason, in the core's own words.
    pub fn reason(&self) -> Option<String> {
        let id = match self {
            MissingInput::NoRuns => "stimulus.no_runs",
            MissingInput::NoHeartRate => "rolling.zones.no_samples",
            MissingInput::NoZoneModel => "zones.no_model",
            MissingInput::NoZoneModelInForce => "stimulus.zones.no_model_in_force",
            MissingInput::NoHeartRateInRun => "stimulus.zones.no_heart_rate",
            MissingInput::NoWorkoutSteps => "stimulus.steps.no_data",
        };
        strings::text(id).map(|text| text.to_string())
    }
}

/// The time a run's own zone time held in one band.
///
/// **One rule, in one place.** The kinds' rows, the weeks matrix and the
/// last occurrence all ask the same question — *how long was the athlete in
/// this band* — and they must not be able to answer it differently.
pub fn band_time(zone_time: &TimeInZone, band: PolarisedZone) -> f64 {
    match band {
        PolarisedZone::Low => zone_time.zone1_s + zone_time.zone2_s,
        PolarisedZone::Moderate => zone_time.zone3_s,
        PolarisedZone::High => zone_time.zone4_s + zone_time.zone5_s,
    }
}

/// One kind's row over the period.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct KindRow {
    /// The band: the athlete's own.
    pub band: PolarisedZone,
    /// The time in the band, in seconds.
    ///
    /// `None` when the period could not be measured at all. A band with no
    /// time in a **measured** period is `Some(0.0)` — a measurement, not an
    /// absence.
    pub time_s: Option<f64>,
    /// The share of the measured moving time, in per cent.
    pub share_percent: Option<f64>,
    /// The runs that carried the band.
    pub runs: usize,
}

/// When a kind last happened — searched over the athlete's **whole
/// history**, not over the period.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LastOccurrence {
    /// The band.
    pub band: PolarisedZone,
    /// The last day the band was recorded, at or before the end of the
    /// period.
    pub date: Option<Date>,
    /// The days between that day and the end of the period.
    pub days_before_end: Option<i64>,
}

/// What the period covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    /// The runs in the period.
    pub runs: usize,
    /// The runs that carried heart-rate samples.
    pub with_heart_rate: usize,
    /// The runs the app could measure — a zone model was in force **and the
    /// run carried a heart rate**.
    ///
    /// A run with no heart rate carries no measurement: the time in a band
    /// would be a zero where there is nothing to measure.
    pub measured: usize,
    /// The runs that carried the source's own zone minutes.
    pub with_source_zones: usize,
}

/// One week of the app's own matrix: the time in each of the athlete's
/// bands.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppWeek {
    /// The Monday the week begins on.
    pub week_start: Date,
    /// The time in the low, moderate and high bands, in seconds.
    ///
    /// `None` where the week held none — the absence is drawn as white
    /// space, not as a zero.
    pub bands: [Option<f64>; 3],
}

/// One week of the source's own matrix: the time in each of the watch's
/// zones.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceWeek {
    /// The Monday the week begins on.
    pub week_start: Date,
    /// The time in the watch's five zones, in seconds. `None` where the week
    /// held none.
    pub zones: [Option<f64>; 5],
}

/// The app's own zone time: over the time the athlete was running.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppBasis {
    /// The kinds' rows over the period.
    pub kinds: Vec<KindRow>,
    /// The weeks matrix.
    pub weeks: Vec<AppWeek>,
    /// The coverage of this basis.
    pub coverage: Coverage,
    /// The moving time the bands were measured over, in seconds.
    pub moving_s: f64,
}

/// The source's own zone minutes: over the whole run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceBasis {
    /// The watch's own zones' rows over the period. There is **no roll-up**:
    /// the athlete's bands are the model's grouping, and applying it to the
    /// watch's own numbers would import boundaries the app does not hold.
    pub zones: Vec<SourceZoneRow>,
    /// The weeks matrix.
    pub weeks: Vec<SourceWeek>,
    /// The coverage of this basis.
    pub coverage: Coverage,
}

/// One of the watch's own zones' row over the period.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceZoneRow {
    /// The watch's zone, one through five.
    pub zone: usize,
    /// The time in it, in seconds.
    pub time_s: Option<f64>,
    /// The share of the source's own zone minutes, in per cent.
    pub share_percent: Option<f64>,
    /// The runs that carried it.
    pub runs: usize,
}

/// Where both bases exist: the runs the app could measure **and** the runs
/// the watch recorded its own zone minutes for.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Overlap {
    /// The first day of the overlap.
    pub from: Date,
    /// The last day of the overlap.
    pub to: Date,
    /// The runs in the overlap.
    pub runs: usize,
    /// The app's own zone time over the overlap, in seconds — over the time
    /// the athlete was running.
    pub app_moving_s: f64,
    /// The source's own zone minutes over the overlap, in seconds — over the
    /// whole run.
    pub source_elapsed_s: f64,
    /// The difference between the two, in seconds: the source's own minutes
    /// minus the app's own.
    ///
    /// A difference between two measurements over two spans. It is not an
    /// error, and the app does not say which of the two is right.
    pub difference_s: f64,
}

/// The two bases, kept apart, and the overlap between them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bases {
    /// The app's own zone time.
    pub app: Option<AppBasis>,
    /// The source's own zone minutes.
    pub source: Option<SourceBasis>,
    /// Where both exist.
    pub overlap: Option<Overlap>,
}

/// The longest run in the period.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LongestRun {
    /// The day of the run.
    pub date: Date,
    /// The run's distance, in metres.
    pub distance_m: Option<f64>,
    /// How long the run moved, in seconds.
    pub moving_time_s: f64,
    /// The share of the period's moving time, in per cent.
    pub share_percent: Option<f64>,
}

/// The repeated work: the runs the watch recorded as workouts with interval
/// steps.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RepeatedWork {
    /// The time in those runs, in seconds.
    pub time_s: f64,
    /// The runs.
    pub runs: usize,
    /// The last day one of them was run.
    pub last_on: Option<Date>,
}

/// The ledger.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    /// The period the ledger answers over.
    pub window: Window,
    /// The kinds' rows, in the product's order.
    pub kinds: Vec<KindRow>,
    /// When each kind last happened, over the athlete's history.
    pub last_occurrence: Vec<LastOccurrence>,
    /// The coverage of the period.
    pub coverage: Coverage,
    /// The longest run in the period.
    pub longest: Option<LongestRun>,
    /// The repeated work in the period.
    pub repeated: Option<RepeatedWork>,
    /// Why the repeated-work row has no value, when it has none.
    pub repeated_missing: Option<MissingInput>,
    /// The two bases and their overlap.
    pub bases: Bases,
}

impl Ledger {
    /// The kind's row, when the ledger holds one.
    pub fn kind(&self, band: PolarisedZone) -> Option<&KindRow> {
        self.kinds.iter().find(|row| row.band == band)
    }

    /// The last occurrence of a kind, when the ledger holds one.
    pub fn last_occurrence_of(&self, band: PolarisedZone) -> Option<&LastOccurrence> {
        self.last_occurrence.iter().find(|row| row.band == band)
    }
}

/// The ledger, over a period and the athlete's history.
///
/// The `runs` are the athlete's **whole retained history**: the period's
/// rows are read from the runs inside it, and the last occurrence of each
/// kind is searched over everything at or before the end of the period.
pub fn ledger(runs: &[LedgerRun], window: Window) -> Ledger {
    let in_window: Vec<&LedgerRun> = runs
        .iter()
        .filter(|run| window.contains(run.date))
        .collect();

    let coverage = Coverage {
        runs: in_window.len(),
        with_heart_rate: in_window.iter().filter(|run| run.has_heart_rate).count(),
        measured: in_window
            .iter()
            .filter(|run| run.zone_time.is_some())
            .count(),
        with_source_zones: in_window
            .iter()
            .filter(|run| run.source_zones.is_some())
            .count(),
    };

    let app = app_basis(&in_window, window, coverage);
    let source = source_basis(&in_window, window, coverage);
    let kinds = app
        .as_ref()
        .map(|basis| basis.kinds.clone())
        .unwrap_or_default();
    let overlap = overlap_of(&in_window);

    // The last occurrence of each kind, searched over the history: every run
    // at or before the end of the period.
    let last_occurrence = last_occurrences(runs, window);

    // The longest run: the greatest moving time, and the **earlier** run when
    // two are equally long, so the answer is deterministic.
    let moving_total: f64 = in_window
        .iter()
        .filter_map(|run| run.moving_time_s)
        .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
        .sum();
    let longest = in_window
        .iter()
        .filter(|run| run.moving_time_s.is_some())
        .min_by(|a, b| {
            let left = a.moving_time_s.unwrap_or_default();
            let right = b.moving_time_s.unwrap_or_default();
            right
                .partial_cmp(&left)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.date.cmp(&b.date))
        })
        .map(|run| LongestRun {
            date: run.date,
            distance_m: run.distance_m,
            moving_time_s: run.moving_time_s.unwrap_or_default(),
            share_percent: if moving_total > 0.0 {
                Some(run.moving_time_s.unwrap_or_default() / moving_total * 100.0)
            } else {
                None
            },
        });

    // The repeated work: the watch's own workout steps, and nothing else.
    let repeated_runs: Vec<&LedgerRun> = in_window
        .iter()
        .copied()
        .filter(|run| run.is_repeated_work())
        .collect();
    let repeated_missing = if in_window.is_empty() {
        Some(MissingInput::NoRuns)
    } else if repeated_runs.is_empty() {
        Some(MissingInput::NoWorkoutSteps)
    } else {
        None
    };
    let repeated = if repeated_runs.is_empty() {
        None
    } else {
        let time_s: f64 = repeated_runs
            .iter()
            .filter_map(|run| run.moving_time_s)
            .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
            .sum();
        let last_on = repeated_runs.iter().map(|run| run.date).max();
        Some(RepeatedWork {
            time_s,
            runs: repeated_runs.len(),
            last_on,
        })
    };

    Ledger {
        window,
        kinds,
        last_occurrence,
        coverage,
        longest,
        repeated,
        repeated_missing,
        bases: Bases {
            app,
            source,
            overlap,
        },
    }
}

/// The app's own basis: the time in the athlete's own bands, over the time
/// the athlete was running.
fn app_basis(in_window: &[&LedgerRun], window: Window, coverage: Coverage) -> Option<AppBasis> {
    if in_window.is_empty() {
        return None;
    }
    let measured: Vec<&&LedgerRun> = in_window
        .iter()
        .filter(|run| run.zone_time.is_some())
        .collect();
    if measured.is_empty() {
        return None;
    }

    let mut times = ZoneTimes::default();
    let mut moving_s = 0.0;
    for run in &measured {
        if let Some(zone_time) = run.zone_time {
            times.zone1_s += zone_time.zone1_s;
            times.zone2_s += zone_time.zone2_s;
            times.zone3_s += zone_time.zone3_s;
            times.zone4_s += zone_time.zone4_s;
            times.zone5_s += zone_time.zone5_s;
            moving_s += zone_time.moving_s();
        }
    }

    let kinds = PolarisedZone::all()
        .iter()
        .map(|band| {
            let time_s = times.band_s(*band);
            let runs = measured
                .iter()
                .filter(|run| {
                    run.zone_time
                        .map(|zone_time| band_time(&zone_time, *band) > 0.0)
                        .unwrap_or(false)
                })
                .count();
            KindRow {
                band: *band,
                time_s: Some(time_s),
                share_percent: if moving_s > 0.0 {
                    Some(time_s / moving_s * 100.0)
                } else {
                    None
                },
                runs,
            }
        })
        .collect();

    let weeks = app_weeks(in_window, window);

    Some(AppBasis {
        kinds,
        weeks,
        coverage,
        moving_s,
    })
}

/// The app's own weeks matrix: per week, per band.
///
/// A band a week held **none** has no cell — the absence is drawn as white
/// space.
fn app_weeks(in_window: &[&LedgerRun], window: Window) -> Vec<AppWeek> {
    week_starts(window)
        .into_iter()
        .map(|week_start| {
            let week_end = week_start.add_days(6);
            let mut low = 0.0;
            let mut moderate = 0.0;
            let mut high = 0.0;
            for run in in_window {
                if run.date < week_start || run.date > week_end {
                    continue;
                }
                if let Some(zone_time) = run.zone_time {
                    low += zone_time.zone1_s + zone_time.zone2_s;
                    moderate += zone_time.zone3_s;
                    high += zone_time.zone4_s + zone_time.zone5_s;
                }
            }
            AppWeek {
                week_start,
                bands: [cell(low), cell(moderate), cell(high)],
            }
        })
        .collect()
}

/// The source's own basis: the watch's own zone minutes, over the whole run.
fn source_basis(
    in_window: &[&LedgerRun],
    window: Window,
    coverage: Coverage,
) -> Option<SourceBasis> {
    let recorded: Vec<&&LedgerRun> = in_window
        .iter()
        .filter(|run| run.source_zones.is_some())
        .collect();
    if recorded.is_empty() {
        return None;
    }

    let mut per_zone = [0.0_f64; 5];
    for run in &recorded {
        if let Some(source) = run.source_zones {
            for (index, seconds) in source.zone_s.iter().enumerate() {
                if let Some(seconds) = seconds {
                    per_zone[index] += seconds;
                }
            }
        }
    }
    let total: f64 = per_zone.iter().sum();

    let zones = per_zone
        .iter()
        .enumerate()
        .map(|(index, time_s)| SourceZoneRow {
            zone: index + 1,
            time_s: Some(*time_s),
            share_percent: if total > 0.0 {
                Some(time_s / total * 100.0)
            } else {
                None
            },
            runs: recorded
                .iter()
                .filter(|run| {
                    run.source_zones
                        .map(|source| {
                            source.zone_s[index]
                                .map(|seconds| seconds > 0.0)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
                .count(),
        })
        .collect();

    let weeks = source_weeks(in_window, window);

    Some(SourceBasis {
        zones,
        weeks,
        coverage,
    })
}

/// The source's own weeks matrix: per week, per watch zone.
fn source_weeks(in_window: &[&LedgerRun], window: Window) -> Vec<SourceWeek> {
    week_starts(window)
        .into_iter()
        .map(|week_start| {
            let week_end = week_start.add_days(6);
            let mut per_zone = [0.0_f64; 5];
            for run in in_window {
                if run.date < week_start || run.date > week_end {
                    continue;
                }
                if let Some(source) = run.source_zones {
                    for (index, seconds) in source.zone_s.iter().enumerate() {
                        if let Some(seconds) = seconds {
                            per_zone[index] += seconds;
                        }
                    }
                }
            }
            SourceWeek {
                week_start,
                zones: per_zone.map(cell),
            }
        })
        .collect()
}

/// The overlap: the runs the app could measure and the watch recorded its own
/// zone minutes for.
///
/// The two totals are **different measurements over different spans** — the
/// time the athlete was running and the whole run — so the gap between them
/// is stated and nothing more is said about it.
fn overlap_of(in_window: &[&LedgerRun]) -> Option<Overlap> {
    let both: Vec<&&LedgerRun> = in_window
        .iter()
        .filter(|run| run.zone_time.is_some() && run.source_zones.is_some())
        .collect();
    if both.is_empty() {
        return None;
    }
    let app_moving_s: f64 = both
        .iter()
        .filter_map(|run| run.zone_time.map(|zone_time| zone_time.moving_s()))
        .sum();
    let source_elapsed_s: f64 = both
        .iter()
        .filter_map(|run| run.source_zones.map(|source| source.total_s()))
        .sum();
    let from = both.iter().map(|run| run.date).min()?;
    let to = both.iter().map(|run| run.date).max()?;
    Some(Overlap {
        from,
        to,
        runs: both.len(),
        app_moving_s,
        source_elapsed_s,
        difference_s: source_elapsed_s - app_moving_s,
    })
}

/// When each kind last happened, over the athlete's whole history.
fn last_occurrences(runs: &[LedgerRun], window: Window) -> Vec<LastOccurrence> {
    PolarisedZone::all()
        .iter()
        .map(|band| {
            let date = runs
                .iter()
                .filter(|run| run.date <= window.to())
                .filter(|run| {
                    run.zone_time
                        .map(|zone_time| band_time(&zone_time, *band) > 0.0)
                        .unwrap_or(false)
                })
                .map(|run| run.date)
                .max();
            LastOccurrence {
                band: *band,
                date,
                days_before_end: date.map(|date| window.to().days_since(date)),
            }
        })
        .collect()
}

/// The Mondays of the weeks the period covers.
fn week_starts(window: Window) -> Vec<Date> {
    let mut starts = Vec::new();
    let mut current = crate::plan::week_start(window.from());
    while current <= window.to() {
        starts.push(current);
        current = current.add_days(7);
    }
    starts
}

/// A cell of the weeks matrix: `None` where the week held none.
fn cell(seconds: f64) -> Option<f64> {
    if seconds.is_finite() && seconds > 0.0 {
        Some(seconds)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::zones::ZoneFloors;

    fn date(year: i32, month: u32, day: u32) -> Date {
        Date::from_ymd(year, month, day).expect("a date")
    }

    /// A run with the app's own zone time: `low` seconds in Z1–Z2,
    /// `moderate` in Z3 and `high` in Z4–Z5.
    fn run(day: Date, moving_s: f64, low: f64, moderate: f64, high: f64) -> LedgerRun {
        LedgerRun {
            date: day,
            distance_m: Some(moving_s / 300.0 * 1_000.0),
            moving_time_s: Some(moving_s),
            elapsed_time_s: Some(moving_s + 180.0),
            zone_time: Some(TimeInZone {
                zone1_s: low * 0.5,
                zone2_s: low * 0.5,
                zone3_s: moderate,
                zone4_s: high * 0.8,
                zone5_s: high * 0.2,
                outside_s: 0.0,
                stopped_s: 0.0,
            }),
            source_zones: None,
            has_heart_rate: true,
            workout_steps: None,
        }
    }

    /// The watch's own zone minutes for a run.
    fn with_source(mut ledger_run: LedgerRun, zone_s: [Option<f64>; 5]) -> LedgerRun {
        ledger_run.source_zones = Some(SourceZones { zone_s });
        ledger_run
    }

    /// The canonical window: twelve calendar weeks through 24 November 2026.
    fn window() -> Window {
        Window::calendar_weeks(12, date(2026, 11, 24))
    }

    #[test]
    fn the_kinds_are_the_athletes_own_bands_and_they_partition_the_moving_time() {
        let runs = vec![
            run(date(2026, 11, 2), 1_800.0, 1_500.0, 240.0, 60.0),
            run(date(2026, 11, 4), 1_200.0, 1_000.0, 200.0, 0.0),
        ];
        let ledger = ledger(&runs, window());

        let low = ledger.kind(PolarisedZone::Low).expect("the low band");
        assert_eq!(low.time_s, Some(2_500.0));
        assert_eq!(low.runs, 2);
        let moderate = ledger
            .kind(PolarisedZone::Moderate)
            .expect("the moderate band");
        assert_eq!(moderate.time_s, Some(440.0));
        assert_eq!(moderate.runs, 2);
        let high = ledger.kind(PolarisedZone::High).expect("the high band");
        assert_eq!(high.time_s, Some(60.0));
        assert_eq!(high.runs, 1);

        // The shares are over the measured moving time.
        let app = ledger.bases.app.as_ref().expect("the app's basis");
        assert_eq!(app.moving_s, 3_000.0);
        let share = low.share_percent.expect("a share");
        assert!((share - 83.33).abs() < 0.1);
    }

    #[test]
    fn a_band_with_no_time_in_a_measured_period_is_a_measured_zero() {
        let runs = vec![run(date(2026, 11, 2), 1_800.0, 1_800.0, 0.0, 0.0)];
        let ledger = ledger(&runs, window());
        let high = ledger.kind(PolarisedZone::High).expect("the high band");
        // Not `None`: the period was measured, and the band held no time.
        assert_eq!(high.time_s, Some(0.0));
        assert_eq!(high.runs, 0);
    }

    #[test]
    fn a_period_with_no_measurable_run_has_no_value_and_a_reason() {
        let runs = vec![LedgerRun::new(date(2026, 11, 2))];
        let ledger = ledger(&runs, window());
        assert!(ledger.bases.app.is_none());
        assert!(ledger.kinds.is_empty());
        assert_eq!(
            MissingInput::NoZoneModel.reason(),
            Some(
                "not measured — your watch's zones were not read, and the watch's own zone \
                 minutes are the only zone time here"
                    .to_string()
            )
        );
    }

    #[test]
    fn the_reasons_name_what_is_missing_and_never_what_to_do_about_it() {
        // The four situations the app's own bands can be missing in are four
        // different facts, and each has its own words.
        assert_eq!(
            MissingInput::NoHeartRate.reason(),
            Some(
                "No heart rate recorded — none of the runs in this period recorded a heart rate"
                    .to_string()
            )
        );
        assert_eq!(
            MissingInput::NoZoneModel.reason(),
            Some(
                "not measured — your watch's zones were not read, and the watch's own zone \
                 minutes are the only zone time here"
                    .to_string()
            )
        );
        assert_eq!(
            MissingInput::NoZoneModelInForce.reason(),
            Some(
                "not measured — your watch's zones were not read for these runs; the watch's own zone minutes are the only zone time here"
                    .to_string()
            )
        );
        assert_eq!(
            MissingInput::NoHeartRateInRun.reason(),
            Some("not measured — the run carries no heart rate".to_string())
        );
    }

    #[test]
    fn the_band_of_a_run_s_time_is_one_rule_everywhere() {
        let time = TimeInZone {
            zone1_s: 100.0,
            zone2_s: 200.0,
            zone3_s: 300.0,
            zone4_s: 400.0,
            zone5_s: 500.0,
            outside_s: 0.0,
            stopped_s: 0.0,
        };
        assert_eq!(band_time(&time, PolarisedZone::Low), 300.0);
        assert_eq!(band_time(&time, PolarisedZone::Moderate), 300.0);
        assert_eq!(band_time(&time, PolarisedZone::High), 900.0);
    }

    #[test]
    fn the_weeks_matrix_leaves_a_band_the_week_held_none_empty() {
        let runs = vec![
            run(date(2026, 11, 2), 1_800.0, 1_500.0, 300.0, 0.0),
            run(date(2026, 11, 9), 1_800.0, 1_800.0, 0.0, 0.0),
        ];
        let ledger = ledger(&runs, window());
        let weeks = &ledger.bases.app.as_ref().expect("the app's basis").weeks;

        let first = weeks
            .iter()
            .find(|week| week.week_start == date(2026, 11, 2))
            .expect("the week");
        assert_eq!(first.bands[0], Some(1_500.0));
        assert_eq!(first.bands[1], Some(300.0));
        // The high band: the week held none, so the cell is empty.
        assert_eq!(first.bands[2], None);

        let second = weeks
            .iter()
            .find(|week| week.week_start == date(2026, 11, 9))
            .expect("the week");
        assert_eq!(second.bands[0], Some(1_800.0));
        assert_eq!(second.bands[1], None);
    }

    #[test]
    fn the_last_occurrence_is_searched_over_the_whole_history() {
        let runs = vec![
            // Before the window: the last run in the high band.
            run(date(2026, 10, 20), 3_600.0, 3_000.0, 500.0, 100.0),
            run(date(2026, 11, 2), 1_800.0, 1_800.0, 0.0, 0.0),
        ];
        let ledger = ledger(&runs, window());
        let last = ledger
            .last_occurrence_of(PolarisedZone::High)
            .expect("the last occurrence");
        // 20 October, which is before the window's end and inside it.
        assert_eq!(last.date, Some(date(2026, 10, 20)));
        assert_eq!(last.days_before_end, Some(35));

        let low = ledger
            .last_occurrence_of(PolarisedZone::Low)
            .expect("the last occurrence");
        assert_eq!(low.date, Some(date(2026, 11, 2)));
    }

    #[test]
    fn the_coverage_counts_the_runs_each_basis_carries() {
        let runs = vec![
            run(date(2026, 11, 2), 1_800.0, 1_800.0, 0.0, 0.0),
            with_source(
                run(date(2026, 11, 4), 1_200.0, 1_200.0, 0.0, 0.0),
                [Some(600.0), Some(600.0), None, None, None],
            ),
            LedgerRun::new(date(2026, 11, 6)),
        ];
        let ledger = ledger(&runs, window());
        assert_eq!(ledger.coverage.runs, 3);
        assert_eq!(ledger.coverage.with_heart_rate, 2);
        assert_eq!(ledger.coverage.measured, 2);
        assert_eq!(ledger.coverage.with_source_zones, 1);
    }

    #[test]
    fn the_longest_run_is_the_one_that_moved_for_the_longest_and_the_earlier_wins_a_tie() {
        let runs = vec![
            run(date(2026, 11, 2), 3_600.0, 3_600.0, 0.0, 0.0),
            run(date(2026, 11, 4), 3_600.0, 3_600.0, 0.0, 0.0),
            run(date(2026, 11, 6), 1_800.0, 1_800.0, 0.0, 0.0),
        ];
        let ledger = ledger(&runs, window());
        let longest = ledger.longest.expect("a longest run");
        assert_eq!(longest.date, date(2026, 11, 2));
        assert_eq!(longest.moving_time_s, 3_600.0);
    }

    #[test]
    fn the_repeated_work_row_is_the_watch_s_own_workout_steps() {
        let mut workout = run(date(2026, 11, 2), 3_600.0, 3_000.0, 500.0, 100.0);
        workout.workout_steps = Some(vec![
            WorkoutStep::Warmup,
            WorkoutStep::Interval,
            WorkoutStep::Interval,
            WorkoutStep::Cooldown,
        ]);
        let mut one_interval = run(date(2026, 11, 4), 1_800.0, 1_800.0, 0.0, 0.0);
        one_interval.workout_steps = Some(vec![WorkoutStep::Warmup, WorkoutStep::Interval]);
        let plain = run(date(2026, 11, 6), 1_800.0, 1_800.0, 0.0, 0.0);

        let ledger = ledger(&[workout, one_interval, plain], window());
        let repeated = ledger.repeated.expect("the repeated work");
        assert_eq!(repeated.runs, 1);
        assert_eq!(repeated.time_s, 3_600.0);
        assert_eq!(repeated.last_on, Some(date(2026, 11, 2)));
    }

    #[test]
    fn with_no_workout_steps_the_repeated_work_row_has_no_value_and_a_reason() {
        let runs = vec![run(date(2026, 11, 2), 1_800.0, 1_800.0, 0.0, 0.0)];
        let ledger = ledger(&runs, window());
        assert!(ledger.repeated.is_none());
        assert_eq!(ledger.repeated_missing, Some(MissingInput::NoWorkoutSteps));
        assert!(MissingInput::NoWorkoutSteps
            .reason()
            .unwrap_or_default()
            .contains("workout steps"));
    }

    #[test]
    fn the_two_bases_are_kept_apart_and_never_added() {
        let runs = vec![
            with_source(
                run(date(2026, 11, 2), 3_000.0, 2_000.0, 800.0, 200.0),
                [
                    Some(1_000.0),
                    Some(1_000.0),
                    Some(800.0),
                    Some(200.0),
                    Some(0.0),
                ],
            ),
            with_source(
                run(date(2026, 11, 4), 1_800.0, 1_800.0, 0.0, 0.0),
                [Some(800.0), Some(800.0), None, None, None],
            ),
        ];
        let ledger = ledger(&runs, window());

        // The app's own basis: over the time the athlete was running.
        let app = ledger.bases.app.as_ref().expect("the app's basis");
        assert_eq!(app.moving_s, 4_800.0);
        // The source's own basis: over the whole run.
        let source = ledger.bases.source.as_ref().expect("the source's basis");
        let total: f64 = source
            .zones
            .iter()
            .map(|row| row.time_s.expect("a time"))
            .sum();
        assert_eq!(total, 4_600.0);
        // The two are not added anywhere: they are two structures.
        assert_ne!(app.moving_s, total);
    }

    #[test]
    fn the_overlap_is_where_both_exist_and_states_the_distance_between_them() {
        let runs = vec![
            with_source(
                run(date(2026, 11, 2), 3_000.0, 2_000.0, 800.0, 200.0),
                [Some(1_500.0), Some(1_500.0), Some(500.0), Some(500.0), None],
            ),
            // A run with no zone model in force: not in the overlap.
            {
                let mut no_model = LedgerRun::new(date(2026, 11, 4));
                no_model.source_zones = Some(SourceZones {
                    zone_s: [Some(600.0), Some(600.0), None, None, None],
                });
                no_model
            },
        ];
        let ledger = ledger(&runs, window());
        let overlap = ledger.bases.overlap.expect("an overlap");
        assert_eq!(overlap.runs, 1);
        assert_eq!(overlap.from, date(2026, 11, 2));
        assert_eq!(overlap.to, date(2026, 11, 2));
        // The app counted the time the athlete was running; the watch
        // counted the whole run.
        assert_eq!(overlap.app_moving_s, 3_000.0);
        assert_eq!(overlap.source_elapsed_s, 4_000.0);
        assert_eq!(overlap.difference_s, 1_000.0);
    }

    #[test]
    fn with_no_model_and_no_source_minutes_there_is_no_overlap() {
        let runs = vec![run(date(2026, 11, 2), 1_800.0, 1_800.0, 0.0, 0.0)];
        let ledger = ledger(&runs, window());
        assert!(ledger.bases.overlap.is_none());
        assert!(ledger.bases.source.is_none());
    }

    #[test]
    fn the_source_s_own_zones_are_never_rolled_up_into_the_athletes_bands() {
        let runs = vec![with_source(
            run(date(2026, 11, 2), 3_000.0, 2_000.0, 800.0, 200.0),
            [Some(1_500.0), Some(1_500.0), Some(500.0), Some(500.0), None],
        )];
        let ledger = ledger(&runs, window());
        let source = ledger.bases.source.as_ref().expect("the source's basis");
        // Five rows, the watch's own zones.
        assert_eq!(source.zones.len(), 5);
        assert_eq!(source.zones[0].zone, 1);
        assert_eq!(source.zones[4].zone, 5);
        assert_eq!(source.zones[0].time_s, Some(1_500.0));
    }

    #[test]
    fn a_window_with_no_runs_holds_no_rows() {
        let ledger = ledger(&[], window());
        assert!(ledger.kinds.is_empty());
        assert!(ledger.longest.is_none());
        assert!(ledger.repeated.is_none());
        assert_eq!(ledger.repeated_missing, Some(MissingInput::NoRuns));
    }

    #[test]
    fn the_zone_floors_are_the_app_s_own_and_the_rows_use_them() {
        // The floors are not the ledger's business — it reads the time in
        // zone the app computed — but the type it reads is the app's.
        let floors = ZoneFloors {
            zone1: 115.0,
            zone2: 128.0,
            zone3: 143.0,
            zone4: 159.0,
            zone5: 165.0,
        };
        assert!(floors.is_usable());
    }
}
