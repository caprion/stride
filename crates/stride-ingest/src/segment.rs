//! Run / walk / stand segmentation for a FIT record stream.
//!
//! A record owns the interval that follows it, up to the next record. A hole
//! in the stream is therefore charged to the record before the hole. The
//! thresholds below are the frozen rules from `docs/contract/core-contract.md`.

use serde::{Deserialize, Serialize};

/// How the athlete was moving in one sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SegmentKind {
    /// The sample belongs to a running stretch.
    Run,
    /// The sample belongs to a walking stretch.
    Walk,
    /// The sample belongs to a standing-still stretch.
    Stand,
}

impl SegmentKind {
    /// The kind's name, as the contract spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Walk => "walk",
            Self::Stand => "stand",
        }
    }
}

/// One record's measurements, before it is classified.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// The sample's position in the record stream.
    pub index: usize,
    /// Seconds since the first record.
    pub elapsed_s: f64,
    /// Cumulative distance, in metres.
    pub distance_m: Option<f64>,
    /// Speed, in metres per second.
    pub speed_ms: Option<f64>,
    /// Cadence exactly as stored in the FIT file.
    pub stored_cadence: Option<f64>,
    /// Heart rate, in beats per minute.
    pub heart_rate_bpm: Option<f64>,
    /// Altitude, in metres.
    pub altitude_m: Option<f64>,
}

/// A sample together with its classification.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassifiedSample {
    /// The sample's measurements.
    pub sample: Sample,
    /// How the sample was classified.
    pub kind: SegmentKind,
}

/// One contiguous stretch of samples of the same kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    /// How the athlete was moving.
    pub kind: SegmentKind,
    /// Elapsed time of the first sample in the stretch.
    pub start_s: f64,
    /// Elapsed time of the **last** sample in the stretch.
    pub end_s: f64,
    /// Number of samples in the stretch.
    pub samples: usize,
    /// `end_s - start_s`.
    pub duration_s: f64,
}

/// The time and distance totals produced by segmentation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SegmentTotals {
    /// Seconds spent running.
    pub running_time_s: f64,
    /// Seconds spent walking.
    pub walking_time_s: f64,
    /// Seconds spent standing still.
    pub stopped_time_s: f64,
    /// Metres covered while running.
    pub running_distance_m: f64,
}

/// The result of segmenting a record stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Segmentation {
    /// Consecutive stretches of one kind.
    pub segments: Vec<Segment>,
    /// Every sample, in file order, with its classification.
    pub samples: Vec<ClassifiedSample>,
    /// The totals charged to each kind.
    pub totals: SegmentTotals,
}

/// Classify one sample from its own speed and its own stored cadence.
///
/// The stored cadence is deliberately not corrected before classification:
/// a watch that stores running cadence halved writes about `80`, and that
/// value is what distinguishes a run from a walk here.
pub fn classify_sample(speed_ms: Option<f64>, stored_cadence: Option<f64>) -> SegmentKind {
    if speed_ms.is_some_and(|speed| speed < 0.30) {
        return SegmentKind::Stand;
    }
    if stored_cadence.is_some_and(|cadence| cadence == 0.0) {
        return SegmentKind::Stand;
    }
    if stored_cadence.is_some_and(|cadence| cadence < 60.0)
        && speed_ms.map_or(true, |speed| speed < 2.2)
    {
        return SegmentKind::Walk;
    }
    if speed_ms.is_some_and(|speed| speed < 1.55) {
        return SegmentKind::Walk;
    }
    SegmentKind::Run
}

/// Split a record stream into run / walk / stand stretches and totals.
///
/// Sample `i` owns the interval from its elapsed time to the next sample's
/// elapsed time. The final sample owns no interval. Positive distance
/// increments are charged to the sample on their left, which is what makes a
/// paused watch's hole part of the preceding running sample.
pub fn segment_samples(samples: &[Sample]) -> Segmentation {
    let mut segments: Vec<Segment> = Vec::new();
    let mut classified = Vec::with_capacity(samples.len());
    let mut totals = SegmentTotals::default();

    for (index, sample) in samples.iter().enumerate() {
        let kind = classify_sample(sample.speed_ms, sample.stored_cadence);
        let next_elapsed = samples
            .get(index + 1)
            .map_or(sample.elapsed_s, |next| next.elapsed_s);
        let span = (next_elapsed - sample.elapsed_s).max(0.0);
        match kind {
            SegmentKind::Run => totals.running_time_s += span,
            SegmentKind::Walk => totals.walking_time_s += span,
            SegmentKind::Stand => totals.stopped_time_s += span,
        }

        if let Some(next) = samples.get(index + 1) {
            if let (Some(previous_distance), Some(next_distance)) =
                (sample.distance_m, next.distance_m)
            {
                if next_distance > previous_distance && kind == SegmentKind::Run {
                    totals.running_distance_m += next_distance - previous_distance;
                }
            }
        }

        if let Some(segment) = segments.last_mut() {
            if segment.kind == kind {
                segment.end_s = round_half_even(sample.elapsed_s, 3);
                segment.samples += 1;
            } else {
                segments.push(new_segment(kind, sample.elapsed_s));
            }
        } else {
            segments.push(new_segment(kind, sample.elapsed_s));
        }

        classified.push(ClassifiedSample {
            sample: sample.clone(),
            kind,
        });
    }

    for segment in &mut segments {
        segment.duration_s = round_half_even(segment.end_s - segment.start_s, 3);
    }

    totals.running_time_s = round_half_even(totals.running_time_s, 1);
    totals.walking_time_s = round_half_even(totals.walking_time_s, 1);
    totals.stopped_time_s = round_half_even(totals.stopped_time_s, 1);
    totals.running_distance_m = round_half_even(totals.running_distance_m, 1);

    Segmentation {
        segments,
        samples: classified,
        totals,
    }
}

fn new_segment(kind: SegmentKind, elapsed_s: f64) -> Segment {
    let elapsed_s = round_half_even(elapsed_s, 3);
    Segment {
        kind,
        start_s: elapsed_s,
        end_s: elapsed_s,
        samples: 1,
        duration_s: 0.0,
    }
}

/// Round a finite value to decimal places with ties going to the even digit.
///
/// This is the rule the frozen corpus was generated with. Rust's ordinary
/// `round` breaks exact ties away from zero and would therefore disagree
/// with the reference at half-way values.
pub(crate) fn round_half_even(value: f64, digits: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }
    // A fixed-precision float format rounds the exact binary value to the
    // nearest representable decimal and breaks a tie towards the even digit.
    format!("{value:.digits$}").parse::<f64>().unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_uses_stored_cadence() {
        assert_eq!(classify_sample(Some(0.0), Some(80.0)), SegmentKind::Stand);
        assert_eq!(classify_sample(Some(3.0), Some(0.0)), SegmentKind::Stand);
        assert_eq!(classify_sample(Some(1.3), Some(30.0)), SegmentKind::Walk);
        assert_eq!(classify_sample(Some(2.0), Some(30.0)), SegmentKind::Walk);
        assert_eq!(classify_sample(Some(2.5), Some(30.0)), SegmentKind::Run);
        assert_eq!(classify_sample(Some(1.0), None), SegmentKind::Walk);
        assert_eq!(classify_sample(Some(3.0), Some(78.0)), SegmentKind::Run);
    }

    #[test]
    fn a_hole_is_charged_to_the_sample_before_it() {
        let samples = vec![
            Sample {
                index: 0,
                elapsed_s: 0.0,
                distance_m: Some(0.0),
                speed_ms: Some(3.0),
                stored_cadence: Some(80.0),
                heart_rate_bpm: None,
                altitude_m: None,
            },
            Sample {
                index: 1,
                elapsed_s: 100.0,
                distance_m: Some(300.0),
                speed_ms: Some(3.0),
                stored_cadence: Some(80.0),
                heart_rate_bpm: None,
                altitude_m: None,
            },
        ];
        let result = segment_samples(&samples);
        assert_eq!(result.totals.running_time_s, 100.0);
        assert_eq!(result.totals.running_distance_m, 300.0);
        assert_eq!(result.segments[0].duration_s, 100.0);
    }

    #[test]
    fn exact_ties_round_to_even() {
        assert_eq!(round_half_even(0.5, 0), 0.0);
        assert_eq!(round_half_even(1.5, 0), 2.0);
        assert_eq!(round_half_even(2.5, 0), 2.0);
        assert_eq!(round_half_even(1.25, 1), 1.2);
        assert_eq!(round_half_even(1.35, 1), 1.4);
    }
}
