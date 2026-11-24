//! The athlete's own races.
//!
//! A race is an **observation**, not a verdict: the athlete types it, and
//! the app never invents one and never infers one from the training. What
//! is proved here:
//!
//! * a race keeps its distance, its time, the day it was run, the day it
//!   was entered, the day it was last changed and its label;
//! * the **anchor** — the race the paces come from — is on one race at a
//!   time, and entering another race never moves it;
//! * a label the app does not know is refused rather than kept.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::{NewRaceObservation, Result};

/// A race to write, with the values a test wants.
fn race() -> NewRaceObservation {
    NewRaceObservation {
        distance_m: 10_000.0,
        time_s: 2_496.0,
        race_date: "2026-10-12".to_string(),
        label: "10K".to_string(),
        name: None,
        source: "typed".to_string(),
        entered_on: "2026-10-13".to_string(),
        updated_on: "2026-11-03".to_string(),
    }
}

#[test]
fn a_race_is_kept_with_its_day_its_label_and_its_dates() -> Result<()> {
    let (dir, store) = common::store("races-write")?;
    let id = store.write_race(&race())?;

    let stored = store.race(id)?.expect("the race is there");
    assert_eq!(stored.distance_m, 10_000.0);
    assert_eq!(stored.time_s, 2_496.0);
    assert_eq!(stored.race_date, "2026-10-12");
    assert_eq!(stored.label, "10K");
    assert_eq!(stored.name, None);
    assert_eq!(stored.source, "typed");
    assert_eq!(stored.entered_on, "2026-10-13");
    assert_eq!(stored.updated_on, "2026-11-03");
    assert_eq!(store.count_race_observations()?, 1);

    drop(dir);
    Ok(())
}

#[test]
fn the_first_race_takes_the_anchor_and_a_second_one_does_not_move_it() -> Result<()> {
    let (dir, store) = common::store("races-anchor")?;
    let first = store.write_race(&race())?;

    let anchor = store.anchor_race()?.expect("the anchor is there");
    assert_eq!(anchor.id, first);

    // A second race, typed later.
    let mut second = race();
    second.race_date = "2026-11-08".to_string();
    second.label = "half marathon".to_string();
    second.distance_m = 21_097.5;
    second.time_s = 5_500.0;
    let other = store.write_race(&second)?;

    // The anchor has not moved: the athlete points it at a race, and
    // entering another race never moves it by itself.
    let anchor = store.anchor_race()?.expect("the anchor is there");
    assert_eq!(anchor.id, first);
    assert_ne!(other, first);
    assert_eq!(store.races()?.len(), 2);

    drop(dir);
    Ok(())
}

#[test]
fn the_athlete_points_the_anchor_at_a_race() -> Result<()> {
    let (dir, store) = common::store("races-point")?;
    let first = store.write_race(&race())?;
    let mut second = race();
    second.race_date = "2026-11-08".to_string();
    second.label = "half marathon".to_string();
    let other = store.write_race(&second)?;

    assert!(store.set_anchor_race(other)?);

    let anchor = store.anchor_race()?.expect("the anchor is there");
    assert_eq!(anchor.id, other);
    // And the race that held it does not hold it any more.
    assert!(!store.race(first)?.expect("the first race is there").anchor);

    // A race that is not there is not an anchor.
    assert!(!store.set_anchor_race(9_999)?);
    let anchor = store.anchor_race()?.expect("the anchor is there");
    assert_eq!(anchor.id, other);

    drop(dir);
    Ok(())
}

#[test]
fn a_label_the_app_does_not_know_is_refused() -> Result<()> {
    let (dir, store) = common::store("races-label")?;
    let mut unknown = race();
    unknown.label = "a fun run".to_string();

    assert!(store.write_race(&unknown).is_err());
    assert_eq!(store.count_race_observations()?, 0);

    drop(dir);
    Ok(())
}

#[test]
fn a_race_is_changed_in_place_and_keeps_its_place_in_the_history() -> Result<()> {
    let (dir, store) = common::store("races-update")?;
    let id = store.write_race(&race())?;

    let mut changed = race();
    changed.time_s = 2_400.0;
    changed.updated_on = "2026-11-20".to_string();
    assert_eq!(store.update_race(id, &changed)?, 1);

    let stored = store.race(id)?.expect("the race is there");
    assert_eq!(stored.time_s, 2_400.0);
    assert_eq!(stored.updated_on, "2026-11-20");
    assert_eq!(stored.entered_on, "2026-10-13");
    assert_eq!(store.count_race_observations()?, 1);

    drop(dir);
    Ok(())
}

#[test]
fn the_races_are_read_newest_first() -> Result<()> {
    let (dir, store) = common::store("races-order")?;
    store.write_race(&race())?;
    let mut later = race();
    later.race_date = "2026-11-08".to_string();
    later.label = "half marathon".to_string();
    store.write_race(&later)?;

    let races = store.races()?;
    assert_eq!(races.len(), 2);
    assert_eq!(races[0].race_date, "2026-11-08");
    assert_eq!(races[1].race_date, "2026-10-12");

    drop(dir);
    Ok(())
}

#[test]
fn deleting_everything_clears_the_races() -> Result<()> {
    let (dir, store) = common::store("races-delete")?;
    store.write_race(&race())?;
    assert_eq!(store.count_race_observations()?, 1);

    store.delete_everything()?;

    assert_eq!(store.count_race_observations()?, 0);
    assert!(store.anchor_race()?.is_none());

    drop(dir);
    Ok(())
}
