//! The training load: a series of numbers, and the scenario that comes with it.
//!
//! The load is a **zone-weighted** measure of training. It weighs an
//! activity by how hard it was, relative to the athlete's own heart-rate
//! zone model, weighs the days against each other, and reports two
//! exponentially weighted series: the acute load (the last seven days) and
//! the chronic load (the last forty-two days). Their difference is the
//! *form*, and their quotient is the *acute-to-chronic ratio*.
//!
//! What is worth keeping is the **scenario**: with a complete rest day, the
//! ratio is expected back inside its usual band by about a day; form turns
//! positive after about three. A scenario says what it assumes and lets the
//! reader decide. This module therefore returns **values and a scenario**,
//! and never a verdict: there is no risk band here, no "optimal", no traffic
//! light, and nothing that tells the athlete what to do. The wording belongs
//! to whoever presents the numbers.
//!
//! ## Where the numbers come from
//!
//! Every input is a fact the athlete's own watch reported, and every one of
//! them is named:
//!
//! * the **max heart rate** and the **threshold** (the zone-5 floor) come
//!   from the athlete's **stored zone model** — the watch's own
//!   configuration, read once and stored with the day it was read
//!   ([`crate::zones`]). There is no age, no date of birth and no sex
//!   anywhere in this module: the reference implementation estimated a max
//!   heart rate and a resting heart rate from a birthday, and the load
//!   arithmetic never used the age at all. The watch knows both numbers, and
//!   an estimate standing in for a fact is the defect this port exists to
//!   remove;
//! * the **resting heart rate** is the *measured* one from the daily
//!   summary, per day. When the day itself carries no reading, the most
//!   recent reading **on or before** that day is used, and the assumption is
//!   stated (see [`LoadInputs::resting_hr`]);
//! * the **moving time** and the **average heart rate** of each activity
//!   come from the stored activity.
//!
//! ## The two places this differs from the reference implementation
//!
//! * **Power.** The reference implementation weighs a ride with power data
//!   by its normalised power against an FTP setting. v2 has no FTP concept,
//!   so the heart-rate path is the only one: a ride with no heart rate
//!   carries **no load**, and the day it is on has **no load value** — it
//!   was not a rest day and nothing on it could be weighed. The reference
//!   implementation adds nothing for such an activity and reports the day as
//!   zero; here it is "no value", because absence is not zero.
//! * **A negative intensity.** An activity whose average heart rate is below
//!   the resting heart rate has no effort to weigh. The reference
//!   implementation would subtract a little load for it; here it weighs
//!   zero, because a load below zero is not a thing.
//!
//! The **intensity** is otherwise the reference's, quantised to a whole
//! percent — see [`activity_trimp`]. That is the one place where the
//! reference's numbers depend on an implementation detail, and this port
//! keeps it: the rule for this item is to match the reference's *numbers* and
//! change its *vocabulary*.
//!
//! ## The model, in full
//!
//! * **intensity** (per activity) — the heart-rate *reserve* of the
//!   activity's average heart rate: `(avgHR − RHR) / (maxHR − RHR)`;
//! * **TRIMP** (per activity) — `minutes × intensity × exp(k × intensity)`;
//! * the **threshold** — `hrrThreshold = (LTHR − RHR) / (maxHR − RHR)` and
//!   `trimpThreshold = 60 × hrrThreshold × exp(k × hrrThreshold)`;
//! * the **day's load** — the sum over that day's activities of
//!   `(trimp / trimpThreshold) × 100`, rounded to a whole number;
//! * the **time constants** — `αATL = 1 − exp(−1/7)` and
//!   `αCTL = 1 − exp(−1/42)`;
//! * the **acute and chronic load** — exponentially weighted over calendar
//!   days: `ATL_t = load_t·αATL + ATL_{t−1}·(1 − αATL)` and the same for the
//!   chronic load, seeded with the first day's load;
//! * the **form** — `CTL − ATL`;
//! * the **ratio** — `ATL / CTL`, and zero when the chronic load is zero;
//! * the **weekly impulse** — the sum of the last seven days' loads;
//! * the **monotony** — `(sum of the seven days / 7) / standard deviation of
//!   the seven days`, available only from the first *full* week onward;
//! * the **strain** — the seven-day sum times the monotony.
//!
//! The acute window is **seven** days and the chronic window is **forty-two**
//! days: the reference implementation's windows, kept so the numbers mean
//! what they meant there.
//!
//! ## What "no value" means here
//!
//! A missing input is never a default:
//!
//! * **no stored zone model** → the whole load answers "no value", with the
//!   reason. Never a guessed max heart rate, never an age formula, never
//!   zero;
//! * **no resting heart rate on or before a day** → that day's load is "no
//!   value". Never zero, never a guessed number;
//! * **a missing day** is a missing day, **not a rest day**. Only a day
//!   known to be a rest day is zero load: the caller says which days those
//!   are (a day the watch recorded and the athlete did not train on). A day
//!   the record does not cover is left out of the series, and the weighted
//!   averages decay across the gap — time passes whether the record covers
//!   it or not.
//!
//! ## Rounding
//!
//! The day's load and the weekly impulse are whole numbers, the acute load,
//! the chronic load and the form carry one decimal, the ratio and the
//! monotony carry two, and the strain is a whole number. The rounding is
//! **half-to-even**, the contract's rule (`docs/contract/core-contract.md`
//! §3) — the reference implementation's `round()` rounds halves away from
//! zero, and the contract wins.
//!
//! ## The scenario
//!
//! [`recovery_scenario`] projects the two series forward with **zero load**
//! for seven days — the assumption of a complete rest — and reports how
//! many days the ratio needs to be back inside its band and how many days
//! the form needs to turn positive. The band is the reference
//! implementation's band (`0.8 ≤ A:C ≤ 1.3`) and it is a band: a range the
//! reader can agree or disagree with. The scenario names its assumptions:
//! the coefficient, where the max heart rate came from and when the model
//! was read, and the two windows.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::strings;
use crate::types::{HeartRate, RestingHr};
use crate::units;
use crate::zones::ZoneFloors;

/// The Banister coefficient — the one modelling constant in this module.
///
/// Banister & Calvert (1982) fitted 1.92 for men and 1.67 for women; 1.92
/// is the canonical value and the convention in TRIMP implementations. The
/// reference implementation took the coefficient from the athlete's sex; it
/// is a **model coefficient, not a personal attribute**, so it belongs here,
/// named and sourced, and not in a settings form that asks the athlete about
/// their body.
///
/// The choice is second-order. The coefficient appears in the formula only as
/// `exp(k · (intensity − threshold))`, so on this athlete's own measured
/// history the two canonical values differ by 3.4 % of the total load and
/// leave the acute-to-chronic ratio and the monotony unchanged. If it ever
/// needs to move, it moves here — a constant becomes a setting without
/// touching the model.
pub const BANISTER_K: f64 = 1.92;

/// The acute window: the seven days the acute load is weighted over.
pub const ACUTE_WINDOW_DAYS: i64 = 7;

/// The chronic window: the forty-two days the chronic load is weighted over.
///
/// Forty-two days is the reference implementation's window.
pub const CHRONIC_WINDOW_DAYS: i64 = 42;

/// The week: the seven days the impulse, the monotony and the strain are
/// measured over.
pub const WEEK_DAYS: i64 = 7;

/// How many days the recovery scenario projects forward.
pub const FORECAST_DAYS: i64 = 7;

/// The lower bound of the acute-to-chronic ratio's band.
pub const RATIO_BAND_LOW: f64 = 0.8;

/// The upper bound of the acute-to-chronic ratio's band.
pub const RATIO_BAND_HIGH: f64 = 1.3;

/// The acute weighting factor, `1 − exp(−1/7)`.
fn alpha_atl() -> f64 {
    1.0 - (-1.0 / ACUTE_WINDOW_DAYS as f64).exp()
}

/// The chronic weighting factor, `1 − exp(−1/42)`.
fn alpha_ctl() -> f64 {
    1.0 - (-1.0 / CHRONIC_WINDOW_DAYS as f64).exp()
}

/// One activity, as the load model reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityLoad {
    /// The athlete-local day the activity belongs to.
    pub date: Date,
    /// How long the activity moved, in seconds.
    pub moving_time_s: f64,
    /// The activity's average heart rate. `None` when the activity carries
    /// no heart rate — a ride without a chest strap, for instance.
    pub average_hr: Option<HeartRate>,
}

/// A measured resting heart rate, on the day it was measured.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RestingHrReading {
    /// The athlete-local day the reading belongs to.
    pub date: Date,
    /// The measured resting heart rate.
    pub resting_hr: RestingHr,
}

/// The stored heart-rate zone model, as the load reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadZoneModel {
    /// The maximum heart rate the model was built on. `None` when the watch
    /// did not report one: the load has no maximum to weigh against, and it
    /// says so rather than guessing one.
    pub max_heart_rate: Option<f64>,
    /// The model's five floors. The **zone-5 floor** is the threshold — the
    /// lactate threshold heart rate the model was built around.
    pub floors: ZoneFloors,
    /// The day the model was read from the watch.
    pub read_on: Date,
}

/// One day of the series, as the record covers it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DayInput {
    /// The day.
    pub date: Date,
    /// Whether the day is known to be a **rest day**: the watch recorded the
    /// day and there was no training on it. A rest day's load is zero.
    ///
    /// A day that is not a rest day and carries no activity has **no load**:
    /// absence is not zero. A day the record does not cover at all is left
    /// out of the series, and the weighted averages decay across the gap.
    pub rest_day: bool,
}

/// Everything the load model reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadInputs {
    /// The days of the series, oldest first: one entry per calendar day the
    /// record covers.
    pub days: Vec<DayInput>,
    /// The activities of the series, in any order. They are joined to the
    /// days by their date.
    pub activities: Vec<ActivityLoad>,
    /// The measured resting heart rates, in any order.
    ///
    /// A day with no reading of its own takes the **most recent reading on
    /// or before that day**: the assumption is that the resting heart rate
    /// has not moved much since it was measured, and the scenario says so.
    /// When there is no reading on or before the day at all, that day's load
    /// is "no value" — never zero, never a guess.
    pub resting_hr: Vec<RestingHrReading>,
    /// The stored zone model, when there is one.
    pub zone_model: Option<LoadZoneModel>,
}

/// Why there is no load.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoLoadReason {
    /// No zone model has been read from the watch yet.
    NoZoneModel,
    /// The stored zone model carries no maximum heart rate.
    NoMaxHeartRate,
    /// The stored zone model's floors cannot be used: they are not finite, or
    /// they do not increase.
    UnusableZoneModel,
    /// There is no history to weigh.
    NoHistory,
    /// The day carries an activity that cannot be weighed: no resting heart
    /// rate was read on or before that day.
    NoRestingHr,
    /// Nothing on the day could be weighed: no activity carried a heart
    /// rate, and the day is not a known rest day.
    NothingWeighable,
    /// The stored model's threshold is not above the resting heart rate of
    /// that day, so the day's scale cannot be built.
    UnusableThreshold,
}

/// The load answer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LoadAnswer {
    /// The load, with the assumptions it was made under.
    Known(LoadReport),
    /// There is no value, and here is why.
    Unavailable(NoLoadReason),
}

/// One day's load.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DailyLoad {
    /// The day.
    pub date: Date,
    /// The day's load, rounded to a whole number. `None` when the day has no
    /// load: it is not covered by the record, or nothing on it could be
    /// weighed.
    pub load: Option<f64>,
    /// Why the day has no load. `Some` exactly when `load` is `None`.
    pub reason: Option<NoLoadReason>,
}

impl DailyLoad {
    /// The day's load, as the core renders it: "54".
    ///
    /// The load is **dimensionless** — there is no unit to write — so the
    /// number is the value. The precision is the load's own: the day's
    /// load is a whole number.
    pub fn load_text(&self) -> Option<String> {
        self.load.map(units::format_whole)
    }
}

/// The state of the two series at a day.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadState {
    /// The day.
    pub date: Date,
    /// The acute load: the seven-day weighted average.
    pub acute: f64,
    /// The chronic load: the forty-two-day weighted average.
    pub chronic: f64,
    /// The form: the chronic load minus the acute load.
    pub form: f64,
    /// The acute-to-chronic ratio. Zero when the chronic load is zero.
    pub ratio: f64,
}

impl LoadState {
    /// The acute load, as the core renders it: "45.2".
    pub fn acute_text(&self) -> String {
        units::format_decimal(self.acute, 1)
    }

    /// The chronic load, as the core renders it: "51.8".
    pub fn chronic_text(&self) -> String {
        units::format_decimal(self.chronic, 1)
    }

    /// The form, as the core renders it: "6.6".
    ///
    /// The form can be negative — the acute load above the chronic one —
    /// and the sign is the number's own.
    pub fn form_text(&self) -> String {
        units::format_decimal(self.form, 1)
    }

    /// The acute-to-chronic ratio, as the core renders it: "0.87".
    pub fn ratio_text(&self) -> String {
        units::format_decimal(self.ratio, 2)
    }
}

/// The seven-day values.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeekLoad {
    /// The sum of the seven days' loads.
    pub impulse: f64,
    /// The monotony: the mean of the seven days over their standard
    /// deviation.
    pub monotony: f64,
    /// The strain: the seven-day sum times the monotony.
    pub strain: f64,
}

impl WeekLoad {
    /// The weekly impulse, as the core renders it: "320".
    pub fn impulse_text(&self) -> String {
        units::format_whole(self.impulse)
    }

    /// The monotony, as the core renders it: "1.42".
    pub fn monotony_text(&self) -> String {
        units::format_decimal(self.monotony, 2)
    }

    /// The strain, as the core renders it: "454".
    pub fn strain_text(&self) -> String {
        units::format_whole(self.strain)
    }
}

/// The assumptions every number of a load answer rests on.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadAssumptions {
    /// The Banister coefficient.
    pub banister_k: f64,
    /// The maximum heart rate the model was built on.
    pub max_heart_rate_bpm: f64,
    /// The threshold heart rate: the zone-5 floor of the stored model.
    pub threshold_hr_bpm: f64,
    /// The day the zone model was read.
    pub zone_model_read_on: Date,
    /// The acute window, in days.
    pub acute_window_days: i64,
    /// The chronic window, in days.
    pub chronic_window_days: i64,
    /// How many days the recovery scenario projects forward, and over which
    /// the assumption of no training is made.
    pub forecast_days: i64,
}

impl LoadAssumptions {
    /// The assumptions as a statement: the assumption of the projection, the
    /// coefficient, where the max heart rate came from and when the model
    /// was read, and the two windows.
    ///
    /// **The day the model was read is passed in, not rendered here.** The
    /// core does not know how a screen writes a date — the api does, and the
    /// api's own rendering is the one the product shows — so the caller
    /// hands the sentence the day it wants written and the core puts it in
    /// its own words. A third date format is not introduced here.
    pub fn statement(&self, zone_model_read_on: &str) -> String {
        let forecast = self.forecast_days.to_string();
        let k = format!("{:.2}", self.banister_k);
        let max_hr = units::render_heart_rate(Some(self.max_heart_rate_bpm)).unwrap_or_default();
        let date = zone_model_read_on.to_string();
        let acute = self.acute_window_days.to_string();
        let chronic = self.chronic_window_days.to_string();
        strings::render(
            "statement.load.assumptions",
            &[
                ("forecast", &forecast),
                ("k", &k),
                ("max_hr", &max_hr),
                ("date", &date),
                ("acute", &acute),
                ("chronic", &chronic),
            ],
        )
        .unwrap_or_default()
    }
}

/// The load of a series of days.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadReport {
    /// The days, oldest first.
    pub days: Vec<DailyLoad>,
    /// The state of the two series at each day, oldest first.
    pub states: Vec<LoadState>,
    /// The seven-day values at the last day of the series. `None` when the
    /// last seven days are not all there with a load: a week with a missing
    /// day has no impulse, no monotony and no strain, because absence is not
    /// zero.
    pub week: Option<WeekLoad>,
    /// The assumptions the numbers were made under.
    pub assumptions: LoadAssumptions,
}

impl LoadReport {
    /// The state at the last day of the series.
    pub fn latest_state(&self) -> Option<&LoadState> {
        self.states.last()
    }
}

/// One projected day of the recovery scenario.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectedDay {
    /// The day of the projection: one is the day after the series ends.
    pub day: u32,
    /// The projected form.
    pub form: f64,
    /// The projected acute-to-chronic ratio.
    pub ratio: f64,
}

impl ProjectedDay {
    /// The projected form, as the core renders it: "6.4".
    pub fn form_text(&self) -> String {
        units::format_decimal(self.form, 1)
    }

    /// The projected acute-to-chronic ratio, as the core renders it:
    /// "0.85".
    pub fn ratio_text(&self) -> String {
        units::format_decimal(self.ratio, 2)
    }
}

/// The recovery scenario: what a run of complete rest days is expected to do.
///
/// It is a scenario and not a verdict. It says what it assumes — a complete
/// rest day, the Banister coefficient, the zone model the max heart rate
/// came from, the two windows — and it leaves the reading to the athlete.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecoveryScenario {
    /// How many complete rest days the ratio is expected to need to be back
    /// inside its band. `Some(0)` when it is inside the band now. `None`
    /// when seven rest days do not bring it back.
    pub days_to_ratio_band: Option<u32>,
    /// How many days the form is expected to need to turn positive.
    /// `Some(0)` when it is positive now. `None` when seven rest days do not
    /// turn it positive.
    pub days_to_positive_form: Option<u32>,
    /// The projection itself, one entry per projected day.
    pub projection: Vec<ProjectedDay>,
    /// The assumptions the projection rests on.
    pub assumptions: LoadAssumptions,
}

impl RecoveryScenario {
    /// The scenario as a statement, with its assumption attached.
    ///
    /// The wording is the product's: values and a direction, no verdict and
    /// no instruction. The assumption travels with it — a scenario that does
    /// not say what it assumed is a verdict in disguise.
    ///
    /// The day the zone model was read is **passed in**: see
    /// [`LoadAssumptions::statement`].
    pub fn statement(&self, zone_model_read_on: &str) -> String {
        let mut parts: Vec<String> = Vec::new();
        for part in self.scenario_parts() {
            parts.push(part);
        }
        parts.push(self.assumptions.statement(zone_model_read_on));
        parts.join(" · ")
    }

    /// The scenario on its own, without the assumption.
    ///
    /// The same words, without the assumption attached: what the rest days
    /// are expected to do. The assumption is its own sentence
    /// ([`LoadAssumptions::statement`]), so a screen can show the two
    /// apart and neither is read without the other.
    pub fn scenario_text(&self) -> String {
        self.scenario_parts().join(" · ")
    }

    /// The two sentences of the scenario: the ratio and the form.
    fn scenario_parts(&self) -> Vec<String> {
        let band = ratio_band_text();
        let ratio = match self.days_to_ratio_band {
            Some(0) => strings::render("statement.load.scenario.ratio.now", &[("band", &band)]),
            Some(days) => {
                let days = day_count(days);
                strings::render(
                    "statement.load.scenario.ratio.days",
                    &[("band", &band), ("days", &days)],
                )
            }
            None => {
                let days = FORECAST_DAYS.to_string();
                strings::render(
                    "statement.load.scenario.ratio.undetermined",
                    &[("band", &band), ("days", &days)],
                )
            }
        };
        let form = match self.days_to_positive_form {
            Some(0) => strings::render("statement.load.scenario.form.now", &[]),
            Some(days) => {
                let days = day_count(days);
                strings::render("statement.load.scenario.form.days", &[("days", &days)])
            }
            None => {
                let days = FORECAST_DAYS.to_string();
                strings::render(
                    "statement.load.scenario.form.undetermined",
                    &[("days", &days)],
                )
            }
        };
        let mut parts: Vec<String> = Vec::new();
        for part in [ratio, form].into_iter().flatten() {
            parts.push(part);
        }
        parts
    }
}

/// The band of the acute-to-chronic ratio, as it is written: "0.8 to 1.3".
///
/// **A band is a range, and it is named as one.** It is never a verdict
/// and never a risk: the reader decides what a ratio inside it means.
pub fn ratio_band_text() -> String {
    format!("{RATIO_BAND_LOW} to {RATIO_BAND_HIGH}")
}

/// A number of days, as it is written: "1 day", "3 days".
fn day_count(days: u32) -> String {
    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}

/// The load of a series of days.
///
/// The answer is never a guessed number: the [`LoadAnswer::Unavailable`] arm
/// says which input was missing, and the [`LoadAnswer::Known`] arm carries
/// the assumptions the numbers were made under.
pub fn load_report(inputs: &LoadInputs) -> LoadAnswer {
    let model = match inputs.zone_model {
        Some(model) => model,
        None => return LoadAnswer::Unavailable(NoLoadReason::NoZoneModel),
    };
    if !model.floors.is_usable() {
        return LoadAnswer::Unavailable(NoLoadReason::UnusableZoneModel);
    }
    let max_heart_rate = match model.max_heart_rate {
        Some(max_heart_rate) if max_heart_rate.is_finite() && max_heart_rate > 0.0 => {
            max_heart_rate
        }
        _ => return LoadAnswer::Unavailable(NoLoadReason::NoMaxHeartRate),
    };
    if inputs.days.is_empty() {
        return LoadAnswer::Unavailable(NoLoadReason::NoHistory);
    }

    let threshold = model.floors.zone5;
    let resting_hr = sorted_resting_hr(inputs);
    let activities = activities_by_day(inputs);

    let mut days: Vec<DailyLoad> = Vec::with_capacity(inputs.days.len());
    let mut states: Vec<LoadState> = Vec::with_capacity(inputs.days.len());
    let mut acute = 0.0;
    let mut chronic = 0.0;
    let mut previous: Option<Date> = None;

    for day in &inputs.days {
        let day_activities = activities.get(&day.date).map(Vec::as_slice).unwrap_or(&[]);
        let (load, reason) = day_load(day, day_activities, &resting_hr, threshold, max_heart_rate);
        let value = load.unwrap_or(0.0);

        // The averages are over **calendar days**: a day the record does not
        // cover is a day that passes, and the averages decay across it. The
        // first day seeds both series.
        let gap = match previous {
            None => 1,
            Some(previous) => day.date.days_since(previous).clamp(1, 1_000_000),
        };
        if previous.is_none() {
            acute = value;
            chronic = value;
        } else {
            let alpha_atl = alpha_atl();
            let alpha_ctl = alpha_ctl();
            acute = value * alpha_atl + acute * (1.0 - alpha_atl).powi(gap as i32);
            chronic = value * alpha_ctl + chronic * (1.0 - alpha_ctl).powi(gap as i32);
        }
        previous = Some(day.date);

        states.push(LoadState {
            date: day.date,
            acute: units::round_half_even(acute, 1),
            chronic: units::round_half_even(chronic, 1),
            form: units::round_half_even(chronic - acute, 1),
            ratio: units::round_half_even(ratio_of(acute, chronic), 2),
        });
        days.push(DailyLoad {
            date: day.date,
            load,
            reason,
        });
    }

    let assumptions = LoadAssumptions {
        banister_k: BANISTER_K,
        max_heart_rate_bpm: max_heart_rate,
        threshold_hr_bpm: threshold,
        zone_model_read_on: model.read_on,
        acute_window_days: ACUTE_WINDOW_DAYS,
        chronic_window_days: CHRONIC_WINDOW_DAYS,
        forecast_days: FORECAST_DAYS,
    };

    LoadAnswer::Known(LoadReport {
        week: week_values(&days),
        days,
        states,
        assumptions,
    })
}

/// The acute-to-chronic ratio, and zero when the chronic load is zero.
fn ratio_of(acute: f64, chronic: f64) -> f64 {
    if chronic == 0.0 {
        0.0
    } else {
        acute / chronic
    }
}

/// The day's load, and the reason when there is none.
///
/// The rules, in one place:
///
/// * an activity with **no heart rate** is not a heart-rate activity: v2 has
///   no FTP model, so it carries no load and does not spoil the day. A day
///   whose only activity is such a ride has **no load** — it was not a rest
///   day, and nothing on it could be weighed;
/// * an activity with a heart rate that cannot be weighed — no resting
///   heart rate on or before its day — leaves the **whole day** without a
///   load: half a day is not the day;
/// * a day with no activity is zero only when it is a **known rest day**.
fn day_load(
    day: &DayInput,
    activities: &[&ActivityLoad],
    resting_hr: &[(Date, f64)],
    threshold: f64,
    max_heart_rate: f64,
) -> (Option<f64>, Option<NoLoadReason>) {
    if activities.is_empty() {
        return if day.rest_day {
            (Some(0.0), None)
        } else {
            (None, Some(NoLoadReason::NothingWeighable))
        };
    }
    let mut total = 0.0;
    let mut weighed = false;
    for activity in activities {
        let heart_rate = match activity.average_hr {
            Some(heart_rate) => heart_rate,
            None => continue,
        };
        let resting = match resting_hr_on_or_before(resting_hr, activity.date) {
            Some(resting) => resting,
            None => return (None, Some(NoLoadReason::NoRestingHr)),
        };
        let contribution = match activity_trimp(
            activity,
            heart_rate.bpm(),
            resting,
            threshold,
            max_heart_rate,
        ) {
            Some(contribution) => contribution,
            None => return (None, Some(NoLoadReason::UnusableThreshold)),
        };
        total += contribution;
        weighed = true;
    }
    if !weighed {
        return (None, Some(NoLoadReason::NothingWeighable));
    }
    (Some(units::round_half_even(total, 0)), None)
}

/// The TRIMP of one activity, as a share of the day's threshold.
///
/// `None` when the threshold cannot be built — the stored model's threshold
/// is not above the resting heart rate of the day.
///
/// The **intensity is quantised to a whole percent** of the reserve, because
/// the reference implementation's is: its
/// `ActivityIntensity::calculateHeartRateBased` returns an integer
/// percentage, and that quantisation is part of its numbers — on a
/// fourteen-day synthetic history it moved three of thirteen day loads by
/// one unit. This port keeps the reference's numbers, so it keeps the
/// quantisation; the rounding itself is half-to-even, the contract's rule.
fn activity_trimp(
    activity: &ActivityLoad,
    heart_rate: f64,
    resting_hr: f64,
    threshold: f64,
    max_heart_rate: f64,
) -> Option<f64> {
    let reserve = max_heart_rate - resting_hr;
    if reserve <= 0.0 {
        return None;
    }
    let intensity = (heart_rate - resting_hr) / reserve;
    let intensity = units::round_half_even(intensity * 100.0, 0) / 100.0;
    if intensity <= 0.0 {
        // The average heart rate was at or below the resting heart rate:
        // there is no effort to weigh, and no negative load.
        return Some(0.0);
    }
    let minutes = activity.moving_time_s / 60.0;
    if !minutes.is_finite() || minutes <= 0.0 {
        return Some(0.0);
    }
    let trimp = minutes * intensity * (BANISTER_K * intensity).exp();
    let hrr_threshold = (threshold - resting_hr) / reserve;
    if hrr_threshold <= 0.0 {
        return None;
    }
    let trimp_threshold = 60.0 * hrr_threshold * (BANISTER_K * hrr_threshold).exp();
    if trimp_threshold <= 0.0 || !trimp_threshold.is_finite() {
        return None;
    }
    Some(trimp / trimp_threshold * 100.0)
}

/// The resting heart rates, oldest first.
fn sorted_resting_hr(inputs: &LoadInputs) -> Vec<(Date, f64)> {
    let mut readings: Vec<(Date, f64)> = inputs
        .resting_hr
        .iter()
        .map(|reading| (reading.date, reading.resting_hr.bpm()))
        .collect();
    readings.sort_by_key(|(date, _)| *date);
    readings
}

/// The most recent resting heart rate read on or before a day.
fn resting_hr_on_or_before(readings: &[(Date, f64)], date: Date) -> Option<f64> {
    readings
        .iter()
        .rev()
        .find_map(|(read_on, bpm)| (*read_on <= date).then_some(*bpm))
}

/// The activities, grouped by their day.
fn activities_by_day(inputs: &LoadInputs) -> std::collections::BTreeMap<Date, Vec<&ActivityLoad>> {
    let mut grouped: std::collections::BTreeMap<Date, Vec<&ActivityLoad>> =
        std::collections::BTreeMap::new();
    for activity in &inputs.activities {
        grouped.entry(activity.date).or_default().push(activity);
    }
    grouped
}

/// The seven-day values at the last day of the series.
///
/// `None` when the last seven days are not all there with a load: a week
/// with a missing day has no impulse, no monotony and no strain. Absence is
/// not zero, and a week that was not measured is not a week of zeroes.
fn week_values(days: &[DailyLoad]) -> Option<WeekLoad> {
    let last = days.last()?;
    let mut loads: Vec<f64> = Vec::with_capacity(WEEK_DAYS as usize);
    for offset in (0..WEEK_DAYS).rev() {
        let date = last.date.add_days(-offset);
        let load = days
            .iter()
            .find(|day| day.date == date)
            .and_then(|day| day.load)?;
        loads.push(load);
    }
    let sum: f64 = loads.iter().sum();
    let mean = sum / WEEK_DAYS as f64;
    let variance: f64 =
        loads.iter().map(|load| (load - mean).powi(2)).sum::<f64>() / WEEK_DAYS as f64;
    let deviation = variance.sqrt();
    // A week whose seven loads are all the same has no variation to
    // measure: the reference implementation reports a monotony of zero
    // there, and so does this port.
    let monotony = if deviation > 0.0 {
        mean / deviation
    } else {
        0.0
    };
    Some(WeekLoad {
        impulse: units::round_half_even(sum, 0),
        monotony: units::round_half_even(monotony, 2),
        strain: units::round_half_even(sum * monotony, 0),
    })
}

/// The recovery scenario of a load report.
///
/// The projection is the reference implementation's: both series decay with
/// **zero load** for seven days — the assumption of a complete rest — and
/// the ratio is inside its band at `0.8 ≤ A:C ≤ 1.3` while the form is
/// positive when `CTL − ATL > 0`.
///
/// The projection starts from the **rounded** acute and chronic loads, the
/// way the reference implementation does.
pub fn recovery_scenario(report: &LoadReport) -> Option<RecoveryScenario> {
    let state = report.latest_state()?;
    let mut acute = state.acute;
    let mut chronic = state.chronic;

    let mut projection: Vec<ProjectedDay> = Vec::with_capacity(FORECAST_DAYS as usize);
    let mut days_to_ratio_band = if inside_band(state.ratio) {
        Some(0)
    } else {
        None
    };
    let mut days_to_positive_form = if state.form > 0.0 { Some(0) } else { None };

    for day in 1..=FORECAST_DAYS {
        acute *= 1.0 - alpha_atl();
        chronic *= 1.0 - alpha_ctl();
        let form = units::round_half_even(chronic - acute, 1);
        let ratio = units::round_half_even(ratio_of(acute, chronic), 2);
        if days_to_ratio_band.is_none() && inside_band(ratio) {
            days_to_ratio_band = Some(day as u32);
        }
        if days_to_positive_form.is_none() && form > 0.0 {
            days_to_positive_form = Some(day as u32);
        }
        projection.push(ProjectedDay {
            day: day as u32,
            form,
            ratio,
        });
    }

    Some(RecoveryScenario {
        days_to_ratio_band,
        days_to_positive_form,
        projection,
        assumptions: report.assumptions,
    })
}

/// Whether a ratio is inside the band.
fn inside_band(ratio: f64) -> bool {
    (RATIO_BAND_LOW..=RATIO_BAND_HIGH).contains(&ratio)
}
