//! The golden assertions of the vitals engine.
//!
//! The example is the one the plan of record prints — `PLAN.md` §0 and
//! `docs/plan.html` §5.9:
//!
//! ```text
//! TODAY · Tue 12 May
//! Rest         could be better · 2 of 3 signals below your usual
//! Sleep        6 h 10 m   · could be better · 1 h 10 m less than your usual
//! HRV          48 ms      · could be better · 14 below your usual · 3rd night down
//! Resting HR   63 bpm     · about usual · 5 above your usual
//! ```
//!
//! The history below is built to produce exactly those numbers: the median of
//! the last thirty nights is 7 h 20 m of sleep, 62 ms of HRV and 58 bpm of
//! resting heart rate, and each band is drawn from the spread of those nights.
//! Nothing here is read from a file and nothing here is a clock — the whole
//! readout is a function of the history it is given.

use stride_core::band;
use stride_core::date::Date;
use stride_core::direction::{self, Direction};
use stride_core::types::{Duration, Hrv, RestingHr, Sleep, VitalKind, Vitals, WellnessDay};

/// The day the readout is for.
fn today() -> Option<Date> {
    Date::parse("2026-05-12")
}

/// One night of the history: how many nights before the readout, and the three
/// vitals of that night.
fn night(index: i64, sleep_s: f64, hrv: f64, resting: f64) -> Option<WellnessDay> {
    let date = Date::parse("2026-04-13")?.add_days(index);
    let sleep = Sleep::new(Duration::from_seconds(sleep_s)?);
    let hrv = Hrv::from_ms(hrv)?;
    let resting = RestingHr::from_bpm(resting)?;
    Some(WellnessDay::new(
        date,
        Vitals {
            sleep: Some(sleep),
            hrv: Some(hrv),
            resting_hr: Some(resting),
        },
    ))
}

/// The thirty nights before the readout, and the night itself.
///
/// The spread is what makes the bands bands: the usual sleep is 7 h 20 m with
/// a median absolute deviation of 20 minutes, the usual HRV is 62 ms with a
/// deviation of 4 ms, and the usual resting heart rate is 58 bpm with a
/// deviation of 6 bpm.
fn history() -> Option<Vec<WellnessDay>> {
    let mut days = Vec::new();
    for index in 0..29i64 {
        let sleep = match index {
            0..=6 => 25200.0,
            7..=20 => 26400.0,
            _ => 27600.0,
        };
        let hrv = match index {
            0..=6 => 58.0,
            7..=19 => 62.0,
            20..=26 => 66.0,
            27 => 58.0,
            _ => 54.0,
        };
        let resting = match index {
            0..=6 => 52.0,
            7..=19 => 58.0,
            20..=26 => 64.0,
            27 => 64.0,
            _ => 63.0,
        };
        days.push(night(index, sleep, hrv, resting)?);
    }
    // The night the readout is for: 6 h 10 m of sleep, 48 ms of HRV and a
    // resting heart rate of 63 bpm.
    days.push(night(29, 22200.0, 48.0, 63.0)?);
    Some(days)
}

#[test]
fn the_readout_reads_as_the_plan_prints_it() -> Result<(), Box<dyn std::error::Error>> {
    let history = history().ok_or("a history")?;
    let today = today().ok_or("a date")?;
    let readout = direction::day_directions(&history, today);

    // The Rest line: a count and a direction.
    assert_eq!(
        readout.rest.line(),
        "could be better · 2 of 3 signals below your usual"
    );

    // The three vitals, in the order the app shows them.
    let sleep = readout.vitals[0];
    assert_eq!(sleep.vital, VitalKind::Sleep);
    assert_eq!(sleep.value_text().as_deref(), Some("6 h 10 m"));
    assert_eq!(sleep.direction, Some(Direction::CouldBeBetter));
    assert_eq!(
        sleep.line(),
        "could be better · 1 h 10 m less than your usual"
    );

    let hrv = readout.vitals[1];
    assert_eq!(hrv.vital, VitalKind::Hrv);
    assert_eq!(hrv.value_text().as_deref(), Some("48 ms"));
    assert_eq!(hrv.direction, Some(Direction::CouldBeBetter));
    assert_eq!(
        hrv.line(),
        "could be better · 14 below your usual · 3rd night down"
    );

    let resting = readout.vitals[2];
    assert_eq!(resting.vital, VitalKind::RestingHr);
    assert_eq!(resting.value_text().as_deref(), Some("63 bpm"));
    assert_eq!(resting.direction, Some(Direction::AboutUsual));
    assert_eq!(resting.line(), "about usual · 5 above your usual");

    Ok(())
}

#[test]
fn the_bands_are_the_median_and_one_and_a_half_mads() -> Result<(), Box<dyn std::error::Error>> {
    let history = history().ok_or("a history")?;
    let today = today().ok_or("a date")?;

    let sleep = band::band_for(&history, today, VitalKind::Sleep).ok_or("a band")?;
    assert_eq!(sleep.usual(), 26400.0);
    assert_eq!(sleep.mad(), 1200.0);
    assert_eq!(sleep.lower(), 24600.0);
    assert_eq!(sleep.upper(), 28200.0);
    assert_eq!(sleep.samples(), 29);

    let hrv = band::band_for(&history, today, VitalKind::Hrv).ok_or("a band")?;
    assert_eq!(hrv.usual(), 62.0);
    assert_eq!(hrv.mad(), 4.0);
    assert_eq!(hrv.lower(), 56.0);
    assert_eq!(hrv.upper(), 68.0);

    let resting = band::band_for(&history, today, VitalKind::RestingHr).ok_or("a band")?;
    assert_eq!(resting.usual(), 58.0);
    assert_eq!(resting.mad(), 6.0);
    assert_eq!(resting.lower(), 49.0);
    assert_eq!(resting.upper(), 67.0);

    Ok(())
}

#[test]
fn the_same_day_reads_the_same_way_every_time() -> Result<(), Box<dyn std::error::Error>> {
    let history = history().ok_or("a history")?;
    let today = today().ok_or("a date")?;
    // Today and Body read the same values: the core owns them, once.
    let first = direction::day_directions(&history, today);
    let second = direction::day_directions(&history, today);
    assert_eq!(first.vitals, second.vitals);
    assert_eq!(first.rest, second.rest);
    Ok(())
}

#[test]
fn without_enough_nights_the_row_says_so_and_invents_nothing(
) -> Result<(), Box<dyn std::error::Error>> {
    let history = history().ok_or("a history")?;
    let today = today().ok_or("a date")?;
    // The last two nights only: no band can be drawn from two nights.
    let thin = history[27..].to_vec();
    let readout = direction::day_directions(&thin, today);
    for vital in &readout.vitals {
        assert_eq!(vital.direction, None);
        assert_eq!(vital.deviation, None);
        assert_eq!(
            vital.status_text().as_deref(),
            Some("not enough data for a comparison yet")
        );
    }
    assert_eq!(readout.rest.direction, None);
    Ok(())
}
