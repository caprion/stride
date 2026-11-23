//! The fingerprint: the observations, the missing readings, and the shares
//! of the time in the athlete's own zones.
//!
//! Every fixture here is synthetic: synthetic laps, synthetic samples,
//! synthetic zone floors and a synthetic maximum heart rate. The expected
//! numbers are worked out from the definitions in the module — the mean,
//! the sample standard deviation, the change between two ends of the run —
//! and not copied from an implementation.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::date::Date;
use stride_core::fingerprint::{
    fingerprint, running_only, Fingerprint, FingerprintInput, FingerprintZoneModel, LapReading,
    NoValue, Observation, ZoneDistribution,
};
use stride_core::zones::{ZoneFloors, ZoneSample};

/// The synthetic zone model's floors, in beats per minute.
const FLOORS: [f64; 5] = [100.0, 120.0, 140.0, 160.0, 175.0];

/// The synthetic zone model's maximum heart rate.
const MAX_HEART_RATE: f64 = 180.0;

/// The model's floors, as the core takes them.
fn floors() -> ZoneFloors {
    ZoneFloors {
        zone1: FLOORS[0],
        zone2: FLOORS[1],
        zone3: FLOORS[2],
        zone4: FLOORS[3],
        zone5: FLOORS[4],
    }
}

/// The synthetic stored zone model.
fn zone_model() -> FingerprintZoneModel {
    FingerprintZoneModel {
        max_heart_rate: Some(MAX_HEART_RATE),
        floors: floors(),
        read_on: Date::parse("2026-05-01"),
    }
}

/// A lap of a kilometre in `seconds`, with the readings it carries.
fn lap(
    seconds: f64,
    heart_rate: Option<f64>,
    cadence: Option<f64>,
    speed: Option<f64>,
) -> LapReading {
    LapReading {
        duration_s: seconds,
        distance_m: 1_000.0,
        avg_hr_bpm: heart_rate,
        avg_cadence_spm: cadence,
        avg_speed_ms: speed,
        stride_length_m: None,
    }
}

/// The observation with this key.
fn observation<'a>(fingerprint: &'a Fingerprint, key: &str) -> &'a Observation {
    fingerprint
        .observations
        .iter()
        .find(|observation| observation.key == key)
        .expect("an observation with this key")
}

/// The value of the observation with this key.
fn value(fingerprint: &Fingerprint, key: &str) -> Option<f64> {
    observation(fingerprint, key).value
}

/// The rendered value of the observation with this key.
fn text(fingerprint: &Fingerprint, key: &str) -> Option<String> {
    observation(fingerprint, key).text.clone()
}

/// The reason the observation with this key has no value.
fn reason(fingerprint: &Fingerprint, key: &str) -> Option<String> {
    observation(fingerprint, key).reason.clone()
}

/// Six running laps of a kilometre each, with the readings they carry.
///
/// The cadence runs from 158 to 168 in steps of two, the heart rate from
/// 140 to 150 in steps of two, and the last lap takes ten seconds longer
/// than the others.
fn six_laps() -> Vec<LapReading> {
    vec![
        lap(300.0, Some(140.0), Some(158.0), Some(3.33)),
        lap(300.0, Some(142.0), Some(160.0), Some(3.33)),
        lap(300.0, Some(144.0), Some(162.0), Some(3.33)),
        lap(300.0, Some(146.0), Some(164.0), Some(3.33)),
        lap(300.0, Some(148.0), Some(166.0), Some(3.33)),
        lap(310.0, Some(150.0), Some(168.0), Some(3.23)),
    ]
}

/// An input over the given laps, with no zone model and no samples.
fn input(laps: Vec<LapReading>) -> FingerprintInput {
    FingerprintInput {
        laps,
        run_distance_m: 6_000.0,
        zone_model: None,
        samples: Vec::new(),
    }
}

#[test]
fn the_cadence_is_the_mean_of_the_running_laps() {
    let answer = fingerprint(&input(six_laps()));

    // (158 + 160 + 162 + 164 + 166 + 168) / 6 = 163.
    assert_eq!(value(&answer, "cadence"), Some(163.0));
    assert_eq!(text(&answer, "cadence"), Some("163.0 spm".to_string()));
    assert_eq!(reason(&answer, "cadence"), None);
}

#[test]
fn the_variation_is_the_sample_standard_deviation_over_the_mean() {
    let answer = fingerprint(&input(six_laps()));

    // The mean of the six cadences is 163, and the deviations are -5, -3,
    // -1, 1, 3, 5. The sum of their squares is 70, over five degrees of
    // freedom: 14, and the deviation is √14.
    let deviation = 14.0_f64.sqrt();
    let expected = deviation / 163.0 * 100.0;
    assert_eq!(value(&answer, "cadence_variability"), Some(expected));
    assert_eq!(
        text(&answer, "cadence_variability"),
        Some("2.3 %".to_string())
    );
}

#[test]
fn the_drift_is_measured_between_the_first_third_and_the_last_third() {
    let answer = fingerprint(&input(six_laps()));

    // The first third is the laps at 158 and 160 (a mean of 159) and the
    // last third is the laps at 166 and 168 (a mean of 167).
    let expected = (167.0 - 159.0) / 159.0 * 100.0;
    assert_eq!(value(&answer, "cadence_drift"), Some(expected));
    // The sign is the direction: the cadence rose.
    assert_eq!(text(&answer, "cadence_drift"), Some("+5.0 %".to_string()));
}

#[test]
fn the_heart_rate_drift_is_the_half_split() {
    let answer = fingerprint(&input(six_laps()));

    // Six laps are three and three: the first half averages 140, 142 and
    // 144 (142) and the second half 146, 148 and 150 (148).
    assert_eq!(value(&answer, "heart_rate_drift"), Some(6.0));
    assert_eq!(
        text(&answer, "heart_rate_drift"),
        Some("+6 bpm".to_string())
    );
}

#[test]
fn the_pace_drift_is_the_half_split_of_the_paces() {
    let answer = fingerprint(&input(six_laps()));

    // Every lap is a kilometre: five in 300 seconds (300 s/km) and the last
    // in 310 (310 s/km). The first half is 300 s/km and the second half is
    // (300 + 300 + 310) / 3.
    let expected = (910.0 / 3.0) - 300.0;
    assert_eq!(value(&answer, "pace_drift"), Some(expected));
    assert_eq!(text(&answer, "pace_drift"), Some("+3 s/km".to_string()));
}

#[test]
fn a_lap_is_walked_only_when_the_cadence_and_the_pace_both_say_so() {
    let walked = lap(600.0, Some(95.0), Some(110.0), Some(1.5));
    let short_strided = lap(300.0, Some(160.0), Some(120.0), Some(3.3));
    let run = lap(300.0, Some(160.0), Some(168.0), Some(3.3));

    let running = running_only(&[walked, short_strided, run]);

    // The walking lap is dropped. The short-strided runner is kept: the
    // cadence is low but the pace is not slow, and both conditions must
    // hold for a lap to be walked.
    assert_eq!(running.len(), 2);
    assert_eq!(running[0].avg_cadence_spm, Some(120.0));
    assert_eq!(running[1].avg_cadence_spm, Some(168.0));
}

#[test]
fn when_everything_looks_like_walking_the_laps_are_kept() {
    let laps = vec![
        lap(600.0, Some(95.0), Some(110.0), Some(1.5)),
        lap(600.0, Some(96.0), Some(112.0), Some(1.5)),
        lap(600.0, Some(97.0), Some(114.0), Some(1.5)),
    ];

    // The classification is not trustworthy, so the original laps are used
    // rather than analysing nothing.
    let running = running_only(&laps);
    assert_eq!(running.len(), 3);
}

#[test]
fn a_fragment_is_not_measured() {
    // A fragment inside the run: a hundred metres, timed at a pace that is
    // mostly noise. Left in, it drags the drift and the variation into
    // figures nobody ran.
    let mut laps = six_laps();
    laps.insert(
        3,
        LapReading {
            duration_s: 9.0,
            distance_m: 100.0,
            avg_hr_bpm: Some(171.0),
            avg_cadence_spm: Some(178.0),
            avg_speed_ms: Some(11.1),
            stride_length_m: None,
        },
    );

    let answer = fingerprint(&input(laps));

    assert_eq!(answer.laps, 6, "the fragment was not measured");
    // And the drift is the drift of the six real laps.
    let drift = value(&answer, "pace_drift").expect("the pace drift");
    assert!(
        drift.abs() < 60.0,
        "the pace drift is a figure a run can have, and it was {drift}"
    );
}

#[test]
fn a_run_of_fragments_is_still_measured() {
    // Three hundred metres each is on the line; shorter is a fragment. When
    // every lap is one, the fragments are used: a short run still says
    // something.
    let laps = vec![
        LapReading {
            duration_s: 80.0,
            distance_m: 200.0,
            avg_hr_bpm: Some(150.0),
            avg_cadence_spm: Some(170.0),
            avg_speed_ms: Some(2.5),
            stride_length_m: None,
        },
        LapReading {
            duration_s: 80.0,
            distance_m: 200.0,
            avg_hr_bpm: Some(152.0),
            avg_cadence_spm: Some(171.0),
            avg_speed_ms: Some(2.5),
            stride_length_m: None,
        },
        LapReading {
            duration_s: 80.0,
            distance_m: 200.0,
            avg_hr_bpm: Some(154.0),
            avg_cadence_spm: Some(172.0),
            avg_speed_ms: Some(2.5),
            stride_length_m: None,
        },
    ];

    let answer = fingerprint(&input(laps));

    assert_eq!(
        answer.laps, 3,
        "the fragments were used, as there was nothing else"
    );
}

#[test]
fn the_fingerprint_uses_the_running_laps_only() {
    let mut laps = six_laps();
    // A walking lap at the end of the run, which the half split would
    // otherwise drag the average heart rate down.
    laps.push(lap(600.0, Some(95.0), Some(110.0), Some(1.5)));

    let answer = fingerprint(&input(laps));

    assert_eq!(answer.laps, 6);
    // Six running laps again: three and three, the same drift as before.
    assert_eq!(value(&answer, "heart_rate_drift"), Some(6.0));
}

#[test]
fn a_run_too_short_to_split_says_so() {
    let laps = vec![lap(300.0, Some(140.0), Some(158.0), Some(3.33))];

    let answer = fingerprint(&input(laps));

    // One lap is not enough to compare one end of the run with the other,
    // and not enough to measure a variation from.
    assert_eq!(value(&answer, "cadence_drift"), None);
    assert_eq!(
        reason(&answer, "cadence_drift"),
        Some("the run has too few laps to compare one end with the other".to_string())
    );
    assert_eq!(value(&answer, "heart_rate_drift"), None);
    assert_eq!(value(&answer, "pace_drift"), None);
    assert_eq!(value(&answer, "decoupling"), None);
    assert_eq!(value(&answer, "cadence_variability"), None);
    assert_eq!(
        reason(&answer, "cadence_variability"),
        Some("the run has too few laps to measure a variation from".to_string())
    );
    // The value itself is still there.
    assert_eq!(value(&answer, "cadence"), Some(158.0));
}

#[test]
fn two_laps_can_be_split_in_half_but_not_in_thirds() {
    let laps = vec![
        lap(300.0, Some(140.0), Some(158.0), Some(3.33)),
        lap(300.0, Some(142.0), Some(160.0), Some(3.33)),
    ];

    let answer = fingerprint(&input(laps));

    // The half split needs one lap at each end, and there is one.
    assert_eq!(value(&answer, "heart_rate_drift"), Some(2.0));
    // The thirds need three laps, and there are two.
    assert_eq!(value(&answer, "cadence_drift"), None);
    assert_eq!(
        reason(&answer, "cadence_drift"),
        Some("the run has too few laps to compare one end with the other".to_string())
    );
}

#[test]
fn a_run_with_no_cadence_says_so() {
    let laps = vec![
        lap(300.0, Some(140.0), None, Some(3.33)),
        lap(300.0, Some(142.0), None, Some(3.33)),
        lap(300.0, Some(144.0), None, Some(3.33)),
    ];

    let answer = fingerprint(&input(laps));

    assert_eq!(value(&answer, "cadence"), None);
    assert_eq!(text(&answer, "cadence"), None);
    assert_eq!(
        reason(&answer, "cadence"),
        Some("the run has no cadence readings".to_string())
    );
    assert_eq!(
        reason(&answer, "cadence_variability"),
        Some("the run has no cadence readings".to_string())
    );
    assert_eq!(
        reason(&answer, "cadence_drift"),
        Some("the run has no cadence readings".to_string())
    );
    // The heart rate is still measured.
    assert_eq!(value(&answer, "heart_rate_drift"), Some(3.0));
}

#[test]
fn the_efficiency_factor_is_the_speed_over_the_heart_rate_fraction() {
    let laps = vec![
        lap(300.0, Some(140.0), Some(158.0), Some(3.33)),
        lap(300.0, Some(145.0), Some(160.0), Some(3.33)),
        lap(300.0, Some(160.0), Some(162.0), Some(3.33)),
    ];
    let mut input = input(laps);
    input.zone_model = Some(zone_model());

    let answer = fingerprint(&input);

    // Every lap is a kilometre in 300 seconds: 3.3333 m/s. The mean heart
    // rate is 148.333, and the fraction of the maximum is 0.824074.
    let expected = (1_000.0 / 300.0) / ((140.0 + 145.0 + 160.0) / 3.0 / 180.0);
    assert_eq!(value(&answer, "efficiency"), Some(expected));
    assert_eq!(
        text(&answer, "efficiency"),
        Some("4.04 m/s per unit of max heart rate".to_string())
    );
}

#[test]
fn the_decoupling_is_the_fall_of_the_efficiency_factor() {
    let laps = vec![
        lap(300.0, Some(140.0), Some(158.0), Some(3.33)),
        lap(300.0, Some(145.0), Some(160.0), Some(3.33)),
        lap(300.0, Some(160.0), Some(162.0), Some(3.33)),
    ];
    let mut input = input(laps);
    input.zone_model = Some(zone_model());

    let answer = fingerprint(&input);

    // Three laps are one lap at each end. The first: 3.3333 m/s over
    // 140/180 = 0.777778, which is 4.285714. The last: 3.3333 m/s over
    // 160/180 = 0.888889, which is 3.75. The fall is 12.5 per cent.
    let decoupling = value(&answer, "decoupling").unwrap_or(0.0);
    assert!(
        (decoupling - 12.5).abs() < 1e-9,
        "the decoupling is {decoupling}"
    );
    assert_eq!(text(&answer, "decoupling"), Some("12.5 %".to_string()));
}

#[test]
fn the_efficiency_factor_needs_a_maximum_heart_rate() {
    let laps = six_laps();
    let mut input = input(laps);
    input.zone_model = Some(FingerprintZoneModel {
        max_heart_rate: None,
        floors: floors(),
        read_on: Date::parse("2026-05-01"),
    });

    let answer = fingerprint(&input);

    assert_eq!(value(&answer, "efficiency"), None);
    assert_eq!(
        reason(&answer, "efficiency"),
        Some("no maximum heart rate to measure the efficiency factor against".to_string())
    );
    assert_eq!(value(&answer, "decoupling"), None);
}

#[test]
fn a_stride_length_over_three_metres_is_rejected() {
    let mut laps = six_laps();
    laps[0].stride_length_m = Some(1.10);
    laps[1].stride_length_m = Some(1.20);
    laps[2].stride_length_m = Some(4.0);
    laps[3].stride_length_m = Some(1.30);

    let answer = fingerprint(&input(laps));

    // The four-metre stride is not a stride: it is a decoding fault. The
    // mean of the three that remain is 1.2 metres.
    assert_eq!(value(&answer, "stride_length"), Some(120.0));
    assert_eq!(text(&answer, "stride_length"), Some("120 cm".to_string()));
}

#[test]
fn a_run_whose_file_carries_no_stride_length_says_so() {
    let answer = fingerprint(&input(six_laps()));

    assert_eq!(value(&answer, "stride_length"), None);
    assert_eq!(
        reason(&answer, "stride_length"),
        Some("the run's file carries no stride length".to_string())
    );
}

#[test]
fn the_zone_shares_come_from_the_stored_model() {
    let mut input = input(six_laps());
    input.zone_model = Some(zone_model());
    // Four samples, ten seconds apart: the first at 130 bpm (zone 2), the
    // second with no reading at all (outside the zones), the third and the
    // fourth at 150 bpm (zone 3).
    input.samples = vec![
        ZoneSample {
            elapsed_s: 0.0,
            moving: true,
            heart_rate_bpm: Some(130.0),
        },
        ZoneSample {
            elapsed_s: 10.0,
            moving: true,
            heart_rate_bpm: None,
        },
        ZoneSample {
            elapsed_s: 20.0,
            moving: true,
            heart_rate_bpm: Some(150.0),
        },
        ZoneSample {
            elapsed_s: 30.0,
            moving: true,
            heart_rate_bpm: Some(150.0),
        },
    ];

    let answer = fingerprint(&input);

    match answer.zones {
        ZoneDistribution::Known { zones, outside_s } => {
            // Zone 2 holds the ten seconds before the missing reading and
            // zone 3 the ten seconds after it. The time outside the zones
            // is the missing reading's own ten seconds.
            assert_eq!(zones[1].seconds, 10.0);
            assert_eq!(zones[2].seconds, 10.0);
            assert_eq!(zones[0].seconds, 0.0);
            assert_eq!(outside_s, 10.0);
            // The shares are of the time **in the zones**: 20 seconds.
            assert_eq!(zones[1].share_pct, 50.0);
            assert_eq!(zones[2].share_pct, 50.0);
        }
        ZoneDistribution::Unavailable(reason) => {
            panic!("the zone distribution is not available: {reason:?}")
        }
    }
}

#[test]
fn a_run_with_no_stored_model_has_no_zone_distribution() {
    let mut input = input(six_laps());
    input.samples = vec![
        ZoneSample {
            elapsed_s: 0.0,
            moving: true,
            heart_rate_bpm: Some(130.0),
        },
        ZoneSample {
            elapsed_s: 10.0,
            moving: true,
            heart_rate_bpm: Some(130.0),
        },
    ];

    let answer = fingerprint(&input);

    match answer.zones {
        ZoneDistribution::Unavailable(NoValue::NoZoneModel) => {}
        other => panic!("the zone distribution is not the model's absence: {other:?}"),
    }
}

#[test]
fn a_model_whose_floors_are_not_in_order_has_no_zone_distribution() {
    let mut input = input(six_laps());
    input.zone_model = Some(FingerprintZoneModel {
        max_heart_rate: Some(MAX_HEART_RATE),
        floors: ZoneFloors {
            zone1: 100.0,
            zone2: 90.0,
            zone3: 140.0,
            zone4: 160.0,
            zone5: 175.0,
        },
        read_on: Date::parse("2026-05-01"),
    });
    input.samples = vec![
        ZoneSample {
            elapsed_s: 0.0,
            moving: true,
            heart_rate_bpm: Some(130.0),
        },
        ZoneSample {
            elapsed_s: 10.0,
            moving: true,
            heart_rate_bpm: Some(130.0),
        },
    ];

    let answer = fingerprint(&input);

    match answer.zones {
        ZoneDistribution::Unavailable(NoValue::UnusableZoneModel) => {}
        other => panic!("the zone distribution is not the model's rejection: {other:?}"),
    }
}

#[test]
fn the_fingerprint_carries_observations_and_no_score() {
    let answer = fingerprint(&input(six_laps()));

    let keys: Vec<&str> = answer
        .observations
        .iter()
        .map(|observation| observation.key.as_str())
        .collect();
    assert_eq!(
        keys,
        vec![
            "cadence",
            "cadence_variability",
            "efficiency",
            "pace_variability",
            "cadence_drift",
            "heart_rate_drift",
            "pace_drift",
            "decoupling",
            "stride_length",
        ]
    );
    // There is no score: nothing here is a grade of the run.
    assert!(!keys.contains(&"form_score"));
}

#[test]
fn every_observation_has_a_value_or_a_reason_and_never_both() {
    let mut input = input(six_laps());
    input.zone_model = Some(zone_model());

    let answer = fingerprint(&input);

    for observation in &answer.observations {
        assert_eq!(
            observation.value.is_some(),
            observation.reason.is_none(),
            "{} has a value and a reason, or neither",
            observation.key
        );
        assert_eq!(
            observation.value.is_some(),
            observation.text.is_some(),
            "{} has a value and no words, or the other way round",
            observation.key
        );
    }
}
