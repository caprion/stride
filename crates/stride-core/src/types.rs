//! The domain types: the measurements the app shows, in the units the
//! contract freezes.
//!
//! Two rules from `docs/contract/core-contract.md` shape everything here:
//!
//! * **§1 units.** A distance is metres, a time is seconds, a heart rate is
//!   beats per minute, an HRV is milliseconds. Conversion happens at the
//!   display boundary, never inside the arithmetic.
//! * **§2 no value.** `None` is the only representation of "absent". A missing
//!   heart rate is `null`, never `0`. A night of no sleep *is* zero, though —
//!   that is a measurement, not an absence — so the constructors accept it and
//!   the callers decide what a missing row means.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::units;

/// A length of time, in seconds.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    seconds: f64,
}

impl Default for Duration {
    /// A duration of no time at all — a night of no sleep, which is a
    /// measurement and not an absence.
    fn default() -> Self {
        Self { seconds: 0.0 }
    }
}

impl Duration {
    /// A duration in seconds. `None` when the number cannot be a duration.
    pub fn from_seconds(seconds: f64) -> Option<Self> {
        if seconds.is_finite() && seconds >= 0.0 {
            Some(Self { seconds })
        } else {
            None
        }
    }

    /// A duration in minutes. `None` when the number cannot be a duration.
    pub fn from_minutes(minutes: f64) -> Option<Self> {
        Self::from_seconds(minutes * 60.0)
    }

    /// A duration in hours. `None` when the number cannot be a duration.
    pub fn from_hours(hours: f64) -> Option<Self> {
        Self::from_seconds(hours * 3600.0)
    }

    /// The duration in seconds.
    pub fn seconds(&self) -> f64 {
        self.seconds
    }

    /// The duration in minutes.
    pub fn minutes(&self) -> f64 {
        self.seconds / 60.0
    }

    /// The duration in hours.
    pub fn hours(&self) -> f64 {
        self.seconds / 3600.0
    }

    /// The human rendering: "6 h 10 m", "42 m".
    pub fn to_human(&self) -> String {
        units::render_duration(Some(self.seconds)).unwrap_or_default()
    }
}

/// A length, in metres.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Distance {
    metres: f64,
}

impl Distance {
    /// A distance in metres. `None` when the number cannot be a distance.
    pub fn from_metres(metres: f64) -> Option<Self> {
        if metres.is_finite() && metres >= 0.0 {
            Some(Self { metres })
        } else {
            None
        }
    }

    /// A distance in kilometres.
    pub fn from_kilometres(kilometres: f64) -> Option<Self> {
        Self::from_metres(kilometres * 1000.0)
    }

    /// The distance in metres.
    pub fn metres(&self) -> f64 {
        self.metres
    }

    /// The distance in kilometres.
    pub fn kilometres(&self) -> f64 {
        self.metres / 1000.0
    }

    /// The human rendering: "8.4 km", "5 km".
    pub fn to_human(&self) -> String {
        units::render_distance(Some(self.metres)).unwrap_or_default()
    }
}

/// A heart rate, in beats per minute.
///
/// Contract §3: heart rate is *truncated* to an integer, not rounded — the
/// constructor does it, so no caller can forget.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HeartRate {
    bpm: f64,
}

impl HeartRate {
    /// A heart rate in beats per minute. `None` when the number is outside the
    /// contract's 0–300 range — a value out of range is absent, not zero.
    pub fn from_bpm(bpm: f64) -> Option<Self> {
        if bpm.is_finite() && (0.0..=300.0).contains(&bpm) {
            Some(Self {
                bpm: units::truncate_heart_rate(bpm) as f64,
            })
        } else {
            None
        }
    }

    /// The heart rate in beats per minute.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// The human rendering: "148 bpm".
    pub fn to_human(&self) -> String {
        units::render_heart_rate(Some(self.bpm)).unwrap_or_default()
    }
}

/// A resting heart rate, in beats per minute.
///
/// A resting heart rate is a heart rate that is also the athlete's *usual*
/// one, so it carries the same truncation and the same range.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RestingHr {
    bpm: f64,
}

impl RestingHr {
    /// A resting heart rate in beats per minute. `None` when the number is
    /// outside the contract's 0–300 range.
    pub fn from_bpm(bpm: f64) -> Option<Self> {
        if bpm.is_finite() && (0.0..=300.0).contains(&bpm) {
            Some(Self {
                bpm: units::truncate_heart_rate(bpm) as f64,
            })
        } else {
            None
        }
    }

    /// The resting heart rate in beats per minute.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// The human rendering: "63 bpm".
    pub fn to_human(&self) -> String {
        units::render_heart_rate(Some(self.bpm)).unwrap_or_default()
    }
}

/// A heart-rate variability, in milliseconds.
///
/// HRV is not truncated: the reference reads it to one decimal, and the
/// deviation from the usual is a whole number of milliseconds, not a whole
/// number of truncated milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Hrv {
    ms: f64,
}

impl Hrv {
    /// An HRV in milliseconds. `None` when the number cannot be one: a zero or
    /// negative HRV is an absent reading, not a reading of zero.
    pub fn from_ms(ms: f64) -> Option<Self> {
        if ms.is_finite() && ms > 0.0 {
            Some(Self { ms })
        } else {
            None
        }
    }

    /// The HRV in milliseconds.
    pub fn ms(&self) -> f64 {
        self.ms
    }
}

/// A night of sleep: the duration, and the stage mix when the watch reports
/// one.
///
/// The stage mix is optional all the way down: a night with no stage data
/// renders the duration chart only — the app never draws an empty stage chart
/// to fill a hole.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Sleep {
    total: Duration,
    deep: Option<Duration>,
    rem: Option<Duration>,
    light: Option<Duration>,
    awake: Option<Duration>,
}

impl Sleep {
    /// A night of sleep with a duration and no stage mix.
    pub fn new(total: Duration) -> Self {
        Self {
            total,
            deep: None,
            rem: None,
            light: None,
            awake: None,
        }
    }

    /// A night of sleep with a duration and a stage mix.
    pub fn with_stages(
        total: Duration,
        deep: Option<Duration>,
        rem: Option<Duration>,
        light: Option<Duration>,
        awake: Option<Duration>,
    ) -> Self {
        Self {
            total,
            deep,
            rem,
            light,
            awake,
        }
    }

    /// The total sleep.
    pub fn total(&self) -> Duration {
        self.total
    }

    /// The deep sleep.
    pub fn deep(&self) -> Option<Duration> {
        self.deep
    }

    /// The REM sleep.
    pub fn rem(&self) -> Option<Duration> {
        self.rem
    }

    /// The light sleep.
    pub fn light(&self) -> Option<Duration> {
        self.light
    }

    /// The time awake.
    pub fn awake(&self) -> Option<Duration> {
        self.awake
    }

    /// Whether the watch reported a stage mix at all.
    pub fn has_stages(&self) -> bool {
        self.deep.is_some() || self.rem.is_some() || self.light.is_some() || self.awake.is_some()
    }

    /// The human rendering of the total: "6 h 10 m".
    pub fn to_human(&self) -> String {
        self.total.to_human()
    }
}

/// The athlete's own rating of how hard a session was.
///
/// The Borg CR10 scale: 0 is nothing at all, 10 is maximal. It is the
/// athlete's number about themselves — the app never produces one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Effort {
    rpe: u8,
}

impl Effort {
    /// A rating of effort. `None` when the number is outside the CR10 scale.
    pub fn from_rpe(rpe: u8) -> Option<Self> {
        if rpe <= 10 {
            Some(Self { rpe })
        } else {
            None
        }
    }

    /// The rating, 0 to 10.
    pub fn rpe(&self) -> u8 {
        self.rpe
    }
}

/// Which side of a measurement is the better side.
///
/// This is a fact about the measurement, not an opinion about the athlete:
/// more sleep is better, a higher HRV is better, a lower resting heart rate
/// is better.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetterSide {
    /// A larger number is the better side.
    Higher,
    /// A smaller number is the better side.
    Lower,
}

/// The three vitals the app shows, on one day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vitals {
    /// Last night's sleep.
    pub sleep: Option<Sleep>,
    /// The morning's heart-rate variability.
    pub hrv: Option<Hrv>,
    /// The day's resting heart rate.
    pub resting_hr: Option<RestingHr>,
}

/// Which vital a number belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VitalKind {
    /// Sleep — the duration of last night.
    Sleep,
    /// Heart-rate variability.
    Hrv,
    /// Resting heart rate.
    RestingHr,
}

impl VitalKind {
    /// The three vitals, in the order the app shows them.
    pub const ALL: [VitalKind; 3] = [VitalKind::Sleep, VitalKind::Hrv, VitalKind::RestingHr];

    /// The label the app shows.
    pub fn label(&self) -> &'static str {
        match self {
            VitalKind::Sleep => "Sleep",
            VitalKind::Hrv => "HRV",
            VitalKind::RestingHr => "Resting HR",
        }
    }

    /// The unit the value is written in.
    pub fn unit(&self) -> &'static str {
        match self {
            VitalKind::Sleep => "h m",
            VitalKind::Hrv => "ms",
            VitalKind::RestingHr => "bpm",
        }
    }

    /// The name used inside a sentence: "sleep", "HRV", "resting HR".
    ///
    /// The label is the label ("Resting HR"); this is the name of the vital
    /// as it reads in the middle of a line ("your resting HR has been …").
    pub fn phrase(&self) -> &'static str {
        match self {
            VitalKind::Sleep => "sleep",
            VitalKind::Hrv => "HRV",
            VitalKind::RestingHr => "resting HR",
        }
    }

    /// Which side of this measurement is the better side.
    pub fn better_side(&self) -> BetterSide {
        match self {
            // More sleep is better; a higher HRV is better; a lower resting
            // heart rate is better.
            VitalKind::Sleep | VitalKind::Hrv => BetterSide::Higher,
            VitalKind::RestingHr => BetterSide::Lower,
        }
    }

    /// The measured value of this vital, in the vital's own scale.
    pub fn value(&self, vitals: &Vitals) -> Option<f64> {
        match self {
            VitalKind::Sleep => vitals.sleep.map(|sleep| sleep.total().seconds()),
            VitalKind::Hrv => vitals.hrv.map(|hrv| hrv.ms()),
            VitalKind::RestingHr => vitals.resting_hr.map(|resting| resting.bpm()),
        }
    }
}

/// One day of wellness: the vitals, and the day they belong to.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WellnessDay {
    /// The athlete-local day.
    pub date: Date,
    /// The three vitals.
    pub vitals: Vitals,
}

impl WellnessDay {
    /// A day of wellness.
    pub fn new(date: Date, vitals: Vitals) -> Self {
        Self { date, vitals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn duration(seconds: f64) -> Result<Duration, &'static str> {
        Duration::from_seconds(seconds).ok_or("not a duration")
    }

    #[test]
    fn a_value_that_cannot_exist_is_absent_not_zero() {
        assert!(Duration::from_seconds(f64::NAN).is_none());
        assert!(Duration::from_seconds(-1.0).is_none());
        assert!(Distance::from_metres(-0.5).is_none());
        assert!(Hrv::from_ms(0.0).is_none());
        assert!(Hrv::from_ms(-3.0).is_none());
        assert!(HeartRate::from_bpm(301.0).is_none());
        assert!(RestingHr::from_bpm(-1.0).is_none());
        assert!(Effort::from_rpe(11).is_none());
    }

    #[test]
    fn a_night_of_no_sleep_is_a_measurement_not_an_absence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sleep = Sleep::new(duration(0.0)?);
        assert_eq!(sleep.total().seconds(), 0.0);
        assert_eq!(sleep.to_human(), "0 m");
        assert!(!sleep.has_stages());
        Ok(())
    }

    #[test]
    fn a_heart_rate_is_truncated_not_rounded() -> Result<(), Box<dyn std::error::Error>> {
        let rate = HeartRate::from_bpm(144.9).ok_or("not a heart rate")?;
        assert_eq!(rate.bpm(), 144.0);
        let resting = RestingHr::from_bpm(63.9).ok_or("not a resting heart rate")?;
        assert_eq!(resting.bpm(), 63.0);
        Ok(())
    }

    #[test]
    fn the_better_side_is_a_fact_about_the_measurement() {
        assert_eq!(VitalKind::Sleep.better_side(), BetterSide::Higher);
        assert_eq!(VitalKind::Hrv.better_side(), BetterSide::Higher);
        assert_eq!(VitalKind::RestingHr.better_side(), BetterSide::Lower);
    }
}
