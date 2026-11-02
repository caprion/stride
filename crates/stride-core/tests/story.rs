//! The golden assertions of the story engine.
//!
//! Every shape has one assertion, and a sweep proves the engine never says a
//! verdict: the sentence is checked on the *output*, over a spread of facts,
//! against the contract's own ban list.

use stride_core::date::Date;
use stride_core::story::{story, RunFacts, Split};
use stride_core::strings::{lint, CATALOGUE};

/// Every word the catalogue holds: the vocabulary the app may say.
///
/// The unit words are the core's too — `units.rs` writes "km", "m", "h" and
/// "bpm" — so the story's whole vocabulary is the catalogue's words plus the
/// units'.
fn catalogue_vocabulary() -> Vec<String> {
    let mut words = vec![
        "km".to_string(),
        "m".to_string(),
        "h".to_string(),
        "bpm".to_string(),
    ];
    for entry in CATALOGUE {
        for word in entry
            .template
            .split(|character: char| !character.is_alphanumeric())
        {
            if !word.is_empty() {
                words.push(word.to_lowercase());
            }
        }
    }
    words
}

fn date(text: &str) -> Option<Date> {
    Date::parse(text)
}

/// A run with the given facts.
fn run(
    day: &str,
    distance_m: Option<f64>,
    moving_time_s: Option<f64>,
    avg_hr: Option<f64>,
) -> Option<RunFacts> {
    let date = date(day)?;
    Some(RunFacts {
        date,
        distance_m,
        moving_time_s,
        avg_hr,
        splits: Vec::new(),
    })
}

/// A run with splits.
fn run_with_splits(
    day: &str,
    distance_m: f64,
    moving_time_s: f64,
    splits: Vec<Split>,
) -> Option<RunFacts> {
    let date = date(day)?;
    Some(RunFacts {
        date,
        distance_m: Some(distance_m),
        moving_time_s: Some(moving_time_s),
        avg_hr: None,
        splits,
    })
}

#[test]
fn the_plain_shape_reads_the_runs_own_numbers() {
    let run = match run("2026-05-12", Some(6_400.0), Some(2_460.0), Some(162.0)) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(story(&run, &[]), "6.4 km in 41 m, at an average of 162 bpm");
}

#[test]
fn a_missing_number_is_left_out_and_the_sentence_stays_a_sentence() {
    let no_heart_rate = match run("2026-05-12", Some(6_400.0), Some(2_460.0), None) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(story(&no_heart_rate, &[]), "6.4 km in 41 m");

    let no_time = match run("2026-05-12", Some(6_400.0), None, Some(162.0)) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(story(&no_time, &[]), "6.4 km, at an average of 162 bpm");

    let distance_only = match run("2026-05-12", Some(6_400.0), None, None) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(story(&distance_only, &[]), "6.4 km");

    let nothing = match run("2026-05-12", None, None, None) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(story(&nothing, &[]), "nothing unusual about this one");
}

#[test]
fn the_usual_shape_compares_the_run_with_the_athletes_own_history() {
    // Three earlier Wednesdays, and a run that was well beyond them.
    let history = vec![
        match run("2026-04-15", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
        match run("2026-04-22", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
        match run("2026-04-29", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
    ];
    let wednesday = match run("2026-05-13", Some(10_200.0), Some(3_600.0), Some(160.0)) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(
        story(&wednesday, &history),
        "10.2 km, 3.2 km more than your usual Wednesday"
    );

    // And the other way round: a run well under the usual.
    let short = match run("2026-05-13", Some(3_000.0), Some(1_200.0), Some(150.0)) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(
        story(&short, &history),
        "3 km, 4 km less than your usual Wednesday"
    );
}

#[test]
fn a_run_within_ten_per_cent_of_the_usual_is_the_usual() {
    let history = vec![
        match run("2026-04-15", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
        match run("2026-04-22", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
        match run("2026-04-29", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
    ];
    let ordinary = match run("2026-05-13", Some(7_200.0), Some(2_800.0), Some(155.0)) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(
        story(&ordinary, &history),
        "7.2 km in 47 m, at an average of 155 bpm"
    );
}

#[test]
fn two_earlier_runs_are_not_a_usual() {
    let history = vec![
        match run("2026-04-15", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
        match run("2026-04-22", Some(7_000.0), None, None) {
            Some(run) => run,
            None => return,
        },
    ];
    let wednesday = match run("2026-05-13", Some(10_200.0), Some(3_600.0), Some(160.0)) {
        Some(run) => run,
        None => return,
    };
    // Two runs are "twice so far", not a usual: the plain shape is the answer.
    assert_eq!(
        story(&wednesday, &history),
        "10.2 km in 1 h, at an average of 160 bpm"
    );
}

#[test]
fn the_split_shape_says_which_part_of_the_run_stood_out() {
    // Four kilometres: the last one is distinctly slower than the rest.
    let run = match run_with_splits(
        "2026-05-12",
        4_000.0,
        1_320.0,
        vec![
            Split {
                distance_m: 1_000.0,
                moving_time_s: 300.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 300.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 300.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 420.0,
            },
        ],
    ) {
        Some(run) => run,
        None => return,
    };
    // The average pace is 5:30; the last kilometre took 7:00, which is more
    // than a tenth slower.
    assert_eq!(story(&run, &[]), "the last 1 km was the slowest of the run");

    // And the other way round: the first kilometre was distinctly quicker.
    let quick = match run_with_splits(
        "2026-05-12",
        4_000.0,
        1_320.0,
        vec![
            Split {
                distance_m: 1_000.0,
                moving_time_s: 240.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 360.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 360.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 360.0,
            },
        ],
    ) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(
        story(&quick, &[]),
        "the first 1 km was the fastest of the run"
    );
}

#[test]
fn a_run_whose_splits_are_even_says_nothing_about_them() {
    let run = match run_with_splits(
        "2026-05-12",
        4_000.0,
        1_200.0,
        vec![
            Split {
                distance_m: 1_000.0,
                moving_time_s: 300.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 302.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 298.0,
            },
            Split {
                distance_m: 1_000.0,
                moving_time_s: 300.0,
            },
        ],
    ) {
        Some(run) => run,
        None => return,
    };
    assert_eq!(story(&run, &[]), "4 km in 20 m");
}

#[test]
fn the_story_is_never_a_verdict() {
    // The output itself is checked, over a spread of facts: the ban list is
    // the contract's, and the sentence is what the athlete reads.
    let days = ["2026-05-12", "2026-05-13", "2026-04-15"];
    let distances = [None, Some(0.0), Some(3_000.0), Some(10_200.0)];
    let times = [None, Some(0.0), Some(1_200.0), Some(3_600.0)];
    let heart_rates = [None, Some(0.0), Some(140.0), Some(190.0)];
    let vocabulary = catalogue_vocabulary();
    let mut checked = 0usize;
    for day in days {
        for distance in distances {
            for time in times {
                for heart_rate in heart_rates {
                    let facts = match run(day, distance, time, heart_rate) {
                        Some(facts) => facts,
                        None => return,
                    };
                    let history = vec![match run("2026-04-15", Some(7_000.0), None, None) {
                        Some(facts) => facts,
                        None => return,
                    }];
                    let sentence = story(&facts, &history);
                    assert!(!sentence.is_empty(), "the story is never empty");
                    // The contract's ban list, checked on the output.
                    for phrase in lint::banned_phrases() {
                        assert!(
                            !lint::contains_phrase(&sentence, phrase),
                            "the story says \"{phrase}\": \"{sentence}\""
                        );
                    }
                    // And the stronger claim: every word the sentence uses is
                    // a word the catalogue holds. The engine renders whole
                    // sentences out of the catalogue, so a word the
                    // catalogue does not hold cannot be said - not "good",
                    // not "great", not anything else at all.
                    for word in sentence.split(|character: char| !character.is_alphanumeric()) {
                        if word.is_empty()
                            || word.chars().all(|character| character.is_ascii_digit())
                        {
                            continue;
                        }
                        assert!(
                            vocabulary.contains(&word.to_lowercase()),
                            "the story says \"{word}\", which the catalogue does not hold: \"{sentence}\""
                        );
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 50, "the sweep should cover a spread of facts");
}
