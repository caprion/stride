//! The fingerprint: the run's own mechanics, as **observations**.
//!
//! The fingerprint is what the run's laps say about the run: how the
//! cadence behaved, how the pace held, how the heart rate drifted from one
//! half of the run to the other, and how far the efficiency factor fell
//! between the first third and the last. Every one of those is a fact about
//! one run, read from the run's own file.
//!
//! **It is not a score.** The reference implementation ended with a
//! `form_score`: a weighted mean of three bounded component scores, on a
//! scale of a hundred, with the weights renormalised around whichever
//! component was missing. That number is not ported, and the reason is the
//! product's own: the app reports values and lets the reader decide. A
//! weighted mean of three arbitrary thresholds is not a fact about a run —
//! it is a grade, and a grade is a verdict. What is worth keeping is the
//! three observations the score was built from, each of which says what it
//! measured.
//!
//! ## What is computed, and how
//!
//! The definitions are the reference implementation's, because they are
//! sound and because the numbers are worth matching:
//!
//! * **the running-only classification** — a lap is walked rather than run
//!   when the **one movement rule** says it was walked: the rule the
//!   ingest classifies its samples with, read here at the lap's own grain
//!   (see [`crate::movement`]). When every lap looks like walking, the
//!   classification is not trustworthy and the original laps are used
//!   instead: analysing nothing would be worse than analysing a run that
//!   was mostly walking;
//! * **the thirds and the halves** — the drift is measured between the first
//!   and the last third of the running laps, and the half-split drift
//!   between the two halves. An average heart rate across a run that
//!   included walking and standing describes nobody; the drift between the
//!   two halves is the number that says something;
//! * **the coefficient of variation** — the sample standard deviation over
//!   the mean, as a percentage, and only from three laps or more: two
//!   numbers have no variation to measure;
//! * **the efficiency factor** — the speed over the heart rate as a
//!   fraction of a maximum heart rate. Speed and not pace: the pace-to-
//!   heart-rate ratio is the reciprocal, and using it would make a *slower*
//!   run look more efficient, which inverts every downstream reading;
//! * **the aerobic decoupling** — how far the efficiency factor fell between
//!   the first third and the last;
//! * **the stride length** — the mean of the laps' own, and a stride longer
//!   than three metres is rejected as implausible.
//!
//! ## Where the inputs come from
//!
//! * the **laps** and the **samples** are the run's own, decoded from its
//!   file on read and never stored: the derived series are recomputable, and
//!   a stored fingerprint could rot;
//! * the **maximum heart rate** the efficiency factor is measured against is
//!   the one from the athlete's **stored zone model** — their watch's own
//!   configuration, read once and stored with the day it was read. It is
//!   never the run's own observed maximum (two runs, two scales) and never
//!   an age formula;
//! * the **zone distribution** is the **shares of the moving time** in the
//!   athlete's own zones, from the same stored model and the same moving
//!   samples the zones route uses, so the two answers cannot disagree.
//!
//! ## "No value" is never a zero
//!
//! A run with no cadence, a run too short to split, a run with no stored
//! zone model: each observation says why, in the core's own words. A metric
//! that cannot be computed is absent, and its absence is named — never a
//! zero, never a band, never a guess.

use serde::{Deserialize, Serialize};

use crate::date::Date;
use crate::movement::{movement_of, Movement};
use crate::strings;
use crate::units::{self, ValueUnit};
use crate::zones::{time_in_zone, ZoneFloors, ZoneSample};

/// The longest stride length that is plausible, in metres.
///
/// A stride longer than this is rejected: the reference implementation
/// rejected it too, because a stride of four metres is a decoding fault and
/// not a stride.
pub const MAX_STRIDE_LENGTH_M: f64 = 3.0;

/// The share of the run's distance below which the last lap is a fragment.
///
/// A last lap that holds less than this much of the run's distance is a
/// partial lap — the run was cut short — and its pace says more about where
/// the watch stopped than about how the run was run. The reference
/// implementation dropped it from the pace variation for the same reason.
const PARTIAL_LAP_SHARE: f64 = 0.15;

/// The fewest laps a coefficient of variation is measured from.
///
/// Two numbers have no variation to measure. The reference implementation's
/// rule, kept.
const MIN_LAPS_FOR_VARIATION: usize = 3;

/// The shortest lap the fingerprint measures, in metres.
///
/// A lap shorter than this is a **fragment**: the watch recorded it, but
/// its pace is mostly noise — the timer's resolution and the GPS's own
/// jitter are a large part of it — and a fragment inside a run wrecks
/// every observation computed across laps. A hundred-metre lap timed at a
/// quarter of a second per kilometre is not a sprint; it is a fragment.
///
/// Three hundred metres is where a lap's pace starts to mean what it
/// says. The laps the fingerprint measures are the ones at least this
/// long, and when a run has none of them the fragments are used as they
/// are: measuring nothing would be worse than measuring a short run.
pub const MIN_LAP_DISTANCE_M: f64 = 300.0;

/// One lap of the run, as the fingerprint reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LapReading {
    /// The lap's running time, in seconds.
    pub duration_s: f64,
    /// The lap's running distance, in metres.
    pub distance_m: f64,
    /// The lap's mean heart rate, in beats per minute.
    pub avg_hr_bpm: Option<f64>,
    /// The lap's mean cadence, in steps per minute.
    pub avg_cadence_spm: Option<f64>,
    /// The lap's mean speed, in metres per second.
    pub avg_speed_ms: Option<f64>,
    /// The lap's stride length, in metres. The files Stride reads do not
    /// carry one, so this is `None` for most laps — and the observation says
    /// so rather than showing a zero.
    pub stride_length_m: Option<f64>,
}

/// The athlete's **stored** zone model, as the fingerprint reads it.
///
/// The model is a dated fact about the watch's own configuration; the api
/// names it and dates it in the answer, and the core uses its floors for
/// the time in zone and its maximum heart rate for the efficiency factor.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FingerprintZoneModel {
    /// The maximum heart rate the model was built on. `None` when the watch
    /// reported none: the efficiency factor has nothing to measure against,
    /// and it says so.
    pub max_heart_rate: Option<f64>,
    /// The model's five floors, in beats per minute.
    pub floors: ZoneFloors,
    /// The day the model was read from the watch.
    pub read_on: Option<Date>,
}

impl FingerprintZoneModel {
    /// Where the model came from, in the core's own words.
    ///
    /// A number that cannot say where its input came from and when it was
    /// read is not inspectable, so the zone distribution says which model
    /// it was measured against and the day that model was read.
    pub fn source(&self) -> Option<String> {
        let read_on = self.read_on?;
        let date = read_on.to_iso();
        strings::render("fingerprint.zones.source", &[("date", &date)])
    }
}

/// Everything the fingerprint reads.
///
/// The input is built by the caller and never crosses the wire: it holds
/// the run's samples, which are not a wire type.
#[derive(Clone, Debug, PartialEq)]
pub struct FingerprintInput {
    /// The run's laps, in the order the file gives them.
    pub laps: Vec<LapReading>,
    /// The run's distance, in metres. The pace variation uses it to tell a
    /// fragment of a last lap from a lap.
    pub run_distance_m: f64,
    /// The athlete's stored zone model, when one has been read.
    pub zone_model: Option<FingerprintZoneModel>,
    /// The run's **moving** samples, for the time in zone. The same filter
    /// the zones route uses, so the two answers cannot disagree.
    pub samples: Vec<ZoneSample>,
}

/// Why an observation has no value.
///
/// Every reason is the core's own sentence: the api passes it on and never
/// writes one of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoValue {
    /// The run's laps carry no cadence.
    NoCadence,
    /// The run's laps carry no heart rate.
    NoHeartRate,
    /// The run's laps carry neither a distance nor a speed, so they have no
    /// pace.
    NoPace,
    /// There are too few laps to measure a variation from.
    TooFewLaps,
    /// There are too few laps to compare one end of the run with the other.
    TooFewLapsToCompare,
    /// The run's file carries no stride length.
    NoStrideLength,
    /// There is no maximum heart rate to measure the efficiency factor
    /// against.
    NoReferenceHeartRate,
    /// No zone model has been read from the watch.
    NoZoneModel,
    /// The stored zone model's floors are not in order, so it cannot be
    /// used.
    UnusableZoneModel,
    /// The run has no samples to read.
    NoSamples,
    /// The run has no time in any of the athlete's zones, so the shares of
    /// the time in the zones are not defined.
    NoTimeInZones,
    /// The run has no file to read: there are no laps and no samples.
    NoFile,
}

impl NoValue {
    /// The reason, in the core's own words.
    pub fn reason(&self) -> Option<String> {
        let id = match self {
            NoValue::NoCadence => "fingerprint.no_cadence",
            NoValue::NoHeartRate => "fingerprint.no_heart_rate",
            NoValue::NoPace => "fingerprint.no_pace",
            NoValue::TooFewLaps => "fingerprint.too_few_laps",
            NoValue::TooFewLapsToCompare => "fingerprint.too_few_laps_to_compare",
            NoValue::NoStrideLength => "fingerprint.no_stride_length",
            NoValue::NoReferenceHeartRate => "fingerprint.no_reference",
            NoValue::NoZoneModel => "zones.no_model",
            NoValue::UnusableZoneModel => "zones.unusable_model",
            NoValue::NoSamples => "fingerprint.no_samples",
            NoValue::NoTimeInZones => "fingerprint.no_time_in_zones",
            NoValue::NoFile => "fingerprint.unavailable",
        };
        strings::text(id).map(|text| text.to_string())
    }
}

/// One observation: a value, or the reason there is none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// The observation's key: `cadence`, `cadence_variability`, and so on.
    /// It is the name the contract uses; the label beside it is the core's
    /// own word.
    pub key: String,
    /// The label, as the core's catalogue writes it.
    pub label: String,
    /// The value, when there is one.
    pub value: Option<f64>,
    /// The value as the core renders it. `None` exactly when `value` is
    /// `None`.
    pub text: Option<String>,
    /// Why there is no value, in the core's words. `Some` exactly when
    /// `value` is `None`.
    pub reason: Option<String>,
}

/// The time in one of the athlete's zones, and its share.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneShare {
    /// The time in the zone, in seconds.
    pub seconds: f64,
    /// The zone's share of the time **in the zones**, as a percentage. The
    /// time outside the zones is not in the denominator: it is reported on
    /// its own.
    pub share_pct: f64,
}

/// The zone distribution of a run.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ZoneDistribution {
    /// The distribution, computed from the athlete's stored model.
    Known {
        /// The time in each of the five zones, and its share.
        zones: [ZoneShare; 5],
        /// The time outside the zones: no heart-rate reading, or a reading
        /// at or below the first floor.
        outside_s: f64,
    },
    /// There is no value, and here is why.
    Unavailable(NoValue),
}

/// The fingerprint of one run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// How many laps the analysis used: the running ones.
    pub laps: usize,
    /// The observations, in the order the app shows them.
    pub observations: Vec<Observation>,
    /// The zone distribution: the shares of the moving time in the
    /// athlete's own zones.
    pub zones: ZoneDistribution,
}

/// The fingerprint of a run.
///
/// The computation is the reference implementation's, with three changes
/// that the port requires:
///
/// * the **zone distribution** is the athlete's **stored** model's, never a
///   model rebuilt from the run;
/// * the **efficiency factor** is measured against the maximum heart rate
///   from that same stored model, never against the run's own observed
///   maximum and never against an age formula;
/// * there is **no composite score**. The observations are the answer.
///
/// Every observation is either a value or the reason it has none.
pub fn fingerprint(input: &FingerprintInput) -> Fingerprint {
    // The laps the observations are computed over: the ones that were run
    // rather than walked, and the ones long enough to measure.
    let laps = measurable(&running_only(&input.laps));
    let third = laps.len() / 3;
    let (first, last) = if third == 0 {
        (&laps[..0], &laps[..0])
    } else {
        (&laps[..third], &laps[laps.len() - third..])
    };
    let half = laps.len() / 2;
    let (first_half, second_half) = if half == 0 {
        (&laps[..0], &laps[..0])
    } else {
        (&laps[..half], &laps[half..])
    };

    // The cadence of the running laps.
    let cadences = values(laps.iter().map(|lap| lap.avg_cadence_spm));
    let cadence_avg = mean(&cadences);
    let cadence_cv = coefficient_of_variation(&cadences);
    let cadence_first = mean(&values(first.iter().map(|lap| lap.avg_cadence_spm)));
    let cadence_last = mean(&values(last.iter().map(|lap| lap.avg_cadence_spm)));
    let cadence_drift = percentage_change(cadence_first, cadence_last);

    // The heart rate of the running laps.
    let heart_rates = values(laps.iter().map(|lap| lap.avg_hr_bpm));
    let heart_rate_first = mean(&values(first_half.iter().map(|lap| lap.avg_hr_bpm)));
    let heart_rate_second = mean(&values(second_half.iter().map(|lap| lap.avg_hr_bpm)));
    let heart_rate_drift = match (heart_rate_first, heart_rate_second) {
        (Some(first), Some(second)) => Some(second - first),
        _ => None,
    };

    // The pace of the running laps. The pace variation leaves out a last
    // lap that holds only a fragment of the run.
    let pace_laps: &[LapReading] = if let Some(last) = laps.last() {
        if input.run_distance_m > 0.0 && last.distance_m < input.run_distance_m * PARTIAL_LAP_SHARE
        {
            &laps[..laps.len() - 1]
        } else {
            &laps
        }
    } else {
        &laps
    };
    let paces = values(pace_laps.iter().map(pace_of));
    let pace_cv = coefficient_of_variation(&paces);
    let pace_first = mean(&values(first_half.iter().map(pace_of)));
    let pace_second = mean(&values(second_half.iter().map(pace_of)));
    let pace_drift = match (pace_first, pace_second) {
        (Some(first), Some(second)) => Some(second - first),
        _ => None,
    };

    // The efficiency factor, and the fall between the first third and the
    // last.
    let reference = input
        .zone_model
        .and_then(|model| model.max_heart_rate)
        .filter(|max| max.is_finite() && *max > 0.0);
    let efficiency = reference.and_then(|reference| efficiency_of(&laps, reference));
    let efficiency_first = reference.and_then(|reference| efficiency_of(first, reference));
    let efficiency_last = reference.and_then(|reference| efficiency_of(last, reference));
    let decoupling = match (efficiency_first, efficiency_last) {
        (Some(first), Some(last)) if first > 0.0 => Some((first - last) / first * 100.0),
        _ => None,
    };

    // The stride length: the laps' own, and only a plausible one.
    let strides = values(laps.iter().map(|lap| lap.stride_length_m))
        .into_iter()
        .filter(|stride| *stride <= MAX_STRIDE_LENGTH_M)
        .collect::<Vec<f64>>();
    let stride_length = mean(&strides).map(|metres| metres * 100.0);

    let observations = vec![
        observation(
            "cadence",
            "fingerprint.label.cadence",
            cadence_avg,
            units::render_cadence,
            if cadence_avg.is_none() {
                Some(NoValue::NoCadence)
            } else {
                None
            },
        ),
        observation(
            "cadence_variability",
            "fingerprint.label.cadence_variability",
            cadence_cv,
            units::render_percent,
            variation_reason(&cadences, NoValue::NoCadence),
        ),
        observation(
            "efficiency",
            "fingerprint.label.efficiency",
            efficiency,
            units::render_efficiency,
            efficiency_reason(reference, &heart_rates),
        ),
        observation(
            "pace_variability",
            "fingerprint.label.pace_variability",
            pace_cv,
            units::render_percent,
            variation_reason(&paces, NoValue::NoPace),
        ),
        observation(
            "cadence_drift",
            "fingerprint.label.cadence_drift",
            cadence_drift,
            |value| units::render_difference(value, ValueUnit::Percent),
            drift_reason(third, &cadences, NoValue::NoCadence),
        ),
        observation(
            "heart_rate_drift",
            "fingerprint.label.heart_rate_drift",
            heart_rate_drift,
            |value| units::render_difference(value, ValueUnit::HeartRate),
            drift_reason(half, &heart_rates, NoValue::NoHeartRate),
        ),
        observation(
            "pace_drift",
            "fingerprint.label.pace_drift",
            pace_drift,
            |value| units::render_difference(value, ValueUnit::Pace),
            drift_reason(half, &paces, NoValue::NoPace),
        ),
        observation(
            "decoupling",
            "fingerprint.label.decoupling",
            decoupling,
            units::render_percent,
            decoupling_reason(third, reference, &heart_rates),
        ),
        observation(
            "stride_length",
            "fingerprint.label.stride_length",
            stride_length,
            units::render_length_cm,
            if stride_length.is_none() {
                Some(NoValue::NoStrideLength)
            } else {
                None
            },
        ),
    ];

    Fingerprint {
        laps: laps.len(),
        observations,
        zones: zone_distribution(input),
    }
}

/// The running laps: the ones that were run rather than walked.
///
/// A lap is walked when its cadence is well below running **and** its pace
/// is slow. Both conditions must hold, so a short-strided runner is not
/// mistaken for a walker, and a lap with no speed at all is judged on its
/// cadence alone.
///
/// When every lap looks like walking, the classification is not
/// trustworthy — a run of very short strides would be thrown away — and the
/// original laps are used instead. Analysing nothing would be worse than
/// analysing a run that was mostly walking.
pub fn running_only(laps: &[LapReading]) -> Vec<LapReading> {
    if laps.is_empty() {
        return Vec::new();
    }
    let running: Vec<LapReading> = laps
        .iter()
        .copied()
        .filter(|lap| !was_walked(lap))
        .collect();
    if running.is_empty() {
        laps.to_vec()
    } else {
        running
    }
}

/// The laps the fingerprint measures: the ones long enough to mean
/// something.
///
/// A fragment — a lap under [`MIN_LAP_DISTANCE_M`] — carries a pace that is
/// mostly noise, and one of them inside the run is enough to turn a drift
/// of a few seconds per kilometre into a figure nobody ran. So they are
/// left out, and when a run is nothing but fragments the fragments are
/// used: a short run still has something to say.
pub fn measurable(laps: &[LapReading]) -> Vec<LapReading> {
    if laps.is_empty() {
        return Vec::new();
    }
    let measured: Vec<LapReading> = laps
        .iter()
        .copied()
        .filter(|lap| lap.distance_m >= MIN_LAP_DISTANCE_M)
        .collect();
    if measured.is_empty() {
        laps.to_vec()
    } else {
        measured
    }
}

/// Whether a lap was walked rather than run.
///
/// **The one movement rule**, read at the lap's grain: the lap's own mean
/// speed and mean cadence, and the same thresholds the ingest classifies
/// its samples with (see [`crate::movement`]). A lap that was standing
/// still is not a lap that was walked — it is not running either, and it
/// is not what the fingerprint excludes.
fn was_walked(lap: &LapReading) -> bool {
    movement_of(lap.avg_speed_ms, lap.avg_cadence_spm) == Movement::Walk
}

/// The pace of a lap, in seconds per kilometre.
///
/// The lap's own distance and time are used when it has both; its speed is
/// the fallback, and a lap with neither has no pace.
fn pace_of(lap: &LapReading) -> Option<f64> {
    if lap.duration_s.is_finite()
        && lap.duration_s > 0.0
        && lap.distance_m.is_finite()
        && lap.distance_m > 0.0
    {
        return Some(lap.duration_s * 1000.0 / lap.distance_m);
    }
    let speed = lap.avg_speed_ms?;
    if !speed.is_finite() || speed <= 0.0 {
        return None;
    }
    Some(1000.0 / speed)
}

/// The efficiency factor of a group of laps: the speed over the heart rate
/// as a fraction of the maximum.
///
/// The laps are the ones that carry **both** a pace and a heart rate, and
/// the pace and the heart rate are their means — the reference
/// implementation's rule, kept.
fn efficiency_of(laps: &[LapReading], reference: f64) -> Option<f64> {
    let paired: Vec<&LapReading> = laps
        .iter()
        .filter(|lap| {
            pace_of(lap).is_some()
                && lap
                    .avg_hr_bpm
                    .map(|hr| hr.is_finite() && hr > 0.0)
                    .unwrap_or(false)
        })
        .collect();
    if paired.is_empty() {
        return None;
    }
    let paces = values(paired.iter().map(|lap| pace_of(lap)));
    let heart_rates = values(paired.iter().map(|lap| lap.avg_hr_bpm));
    let pace = mean(&paces)?;
    let heart_rate = mean(&heart_rates)?;
    if pace <= 0.0 || heart_rate <= 0.0 || reference <= 0.0 {
        return None;
    }
    let speed = 1000.0 / pace;
    Some(speed / (heart_rate / reference))
}

/// The values of a series, keeping the ones that are measurements.
///
/// A value that is absent, or that is not a positive finite number, is not
/// a measurement: it is a missing reading, and it is left out of the mean
/// rather than counted as a zero.
fn values(series: impl Iterator<Item = Option<f64>>) -> Vec<f64> {
    series
        .flatten()
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect()
}

/// The mean of a series of measurements.
fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    mean.is_finite().then_some(mean)
}

/// The coefficient of variation: the sample standard deviation over the
/// mean, as a percentage.
///
/// `None` from fewer than three values, or from a mean that is not
/// positive: two numbers have no variation to measure, and a mean of zero
/// has no variation to measure it against.
fn coefficient_of_variation(values: &[f64]) -> Option<f64> {
    if values.len() < MIN_LAPS_FOR_VARIATION {
        return None;
    }
    let mean = mean(values)?;
    if mean <= 0.0 {
        return None;
    }
    let sum: f64 = values.iter().map(|value| (value - mean).powi(2)).sum();
    let deviation = (sum / (values.len() as f64 - 1.0)).sqrt();
    Some(deviation / mean * 100.0)
}

/// The change between two measurements, as a percentage of the first.
///
/// `None` when there is no first measurement, or when the first is not
/// positive: a percentage without a baseline is not a percentage.
fn percentage_change(first: Option<f64>, last: Option<f64>) -> Option<f64> {
    match (first, last) {
        (Some(first), Some(last)) if first > 0.0 => Some((last - first) / first * 100.0),
        _ => None,
    }
}

/// The reason a variation has no value.
///
/// A series with no readings at all is missing the measurement itself; a
/// series with one or two readings is missing enough laps to measure a
/// variation from.
fn variation_reason(values: &[f64], missing: NoValue) -> Option<NoValue> {
    if values.is_empty() {
        return Some(missing);
    }
    if values.len() < MIN_LAPS_FOR_VARIATION {
        return Some(NoValue::TooFewLaps);
    }
    None
}

/// The reason a drift has no value.
///
/// Too few laps to compare one end of the run with the other comes first:
/// it is the more fundamental of the two, and it is true of every reading
/// of the run.
fn drift_reason(parts: usize, values: &[f64], missing: NoValue) -> Option<NoValue> {
    if parts == 0 {
        return Some(NoValue::TooFewLapsToCompare);
    }
    if values.is_empty() {
        return Some(missing);
    }
    None
}

/// The reason the efficiency factor has no value.
fn efficiency_reason(reference: Option<f64>, heart_rates: &[f64]) -> Option<NoValue> {
    if reference.is_none() {
        return Some(NoValue::NoReferenceHeartRate);
    }
    if heart_rates.is_empty() {
        return Some(NoValue::NoHeartRate);
    }
    None
}

/// The reason the decoupling has no value.
fn decoupling_reason(third: usize, reference: Option<f64>, heart_rates: &[f64]) -> Option<NoValue> {
    if third == 0 {
        return Some(NoValue::TooFewLapsToCompare);
    }
    efficiency_reason(reference, heart_rates)
}

/// One observation, with its value or its reason.
fn observation(
    key: &'static str,
    label_id: &str,
    value: Option<f64>,
    render: fn(Option<f64>) -> Option<String>,
    reason: Option<NoValue>,
) -> Observation {
    Observation {
        key: key.to_string(),
        label: strings::text(label_id).unwrap_or_default().to_string(),
        value,
        text: render(value),
        reason: reason.and_then(|reason| reason.reason()),
    }
}

/// The zone distribution of a run, from the athlete's stored model.
///
/// The shares are of the time **in the zones**: the time outside them — no
/// heart-rate reading, or a reading at or below the first floor — is
/// reported on its own, because it is not part of any zone.
fn zone_distribution(input: &FingerprintInput) -> ZoneDistribution {
    let Some(model) = input.zone_model else {
        return ZoneDistribution::Unavailable(NoValue::NoZoneModel);
    };
    let Some(time) = time_in_zone(model.floors, &input.samples) else {
        return ZoneDistribution::Unavailable(NoValue::UnusableZoneModel);
    };
    let times = time.per_zone();
    let in_zone: f64 = times.iter().sum();
    if in_zone <= 0.0 {
        return ZoneDistribution::Unavailable(NoValue::NoTimeInZones);
    }
    let zones = times.map(|seconds| ZoneShare {
        seconds,
        share_pct: seconds / in_zone * 100.0,
    });
    ZoneDistribution::Known {
        zones,
        outside_s: time.outside_s,
    }
}
