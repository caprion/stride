//! The athlete's own threshold.
//!
//! The choice is a **dated fact**: the value (or the pair of values), the
//! source it came from, the rule that computed it, the base it was made
//! for, the day the choice was made and the day the row was last changed.
//! What is proved here:
//!
//! * one choice is **in force** at a time, and the earlier choices stay in
//!   the dated history;
//! * a choice that carries no value at all is refused;
//! * the **kind** a choice carries is read from the values it carries, so
//!   the kind and the values cannot disagree.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::{NewThresholdObservation, Result, ThresholdKind};

/// A choice to write, with the values a test wants.
fn choice() -> NewThresholdObservation {
    NewThresholdObservation {
        hr_bpm: Some(165.0),
        pace_low_s_per_km: None,
        pace_high_s_per_km: None,
        source: "watch_zone_model".to_string(),
        rule_id: Some("zone_model.zone5_floor".to_string()),
        base: Some("pfitzinger".to_string()),
        origin_date: Some("2026-11-03".to_string()),
        origin_ref: Some("1".to_string()),
        label: None,
        entered_on: "2026-11-03".to_string(),
        updated_on: "2026-11-03".to_string(),
    }
}

#[test]
fn a_choice_is_kept_with_its_source_its_rule_its_base_and_its_dates() -> Result<()> {
    let (dir, store) = common::store("threshold-write")?;
    let id = store.write_threshold_choice(&choice(), true)?;

    let stored = store.threshold_choice(id)?.expect("the choice is there");
    assert_eq!(stored.hr_bpm, Some(165.0));
    assert_eq!(stored.source, "watch_zone_model");
    assert_eq!(stored.rule_id.as_deref(), Some("zone_model.zone5_floor"));
    assert_eq!(stored.base.as_deref(), Some("pfitzinger"));
    assert_eq!(stored.origin_date.as_deref(), Some("2026-11-03"));
    assert_eq!(stored.entered_on, "2026-11-03");
    assert_eq!(stored.updated_on, "2026-11-03");
    assert!(stored.in_force);
    assert_eq!(store.count_threshold_observations()?, 1);

    drop(dir);
    Ok(())
}

#[test]
fn one_choice_is_in_force_at_a_time_and_the_earlier_one_stays_in_the_history() -> Result<()> {
    let (dir, store) = common::store("threshold-history")?;
    let first = store.write_threshold_choice(&choice(), true)?;

    // A later choice: the athlete's own number, typed.
    let mut second = choice();
    second.hr_bpm = Some(168.0);
    second.source = "typed".to_string();
    second.rule_id = None;
    second.label = Some("lab test".to_string());
    second.entered_on = "2027-03-03".to_string();
    second.updated_on = "2027-03-03".to_string();
    let later = store.write_threshold_choice(&second, true)?;

    // One choice is in force.
    let in_force = store.threshold_in_force()?.expect("a choice is in force");
    assert_eq!(in_force.id, later);

    // The earlier choice is not deleted and not edited: it is in the
    // history, with its own value and its own date.
    let earlier = store
        .threshold_choice(first)?
        .expect("the earlier choice is still there");
    assert_eq!(earlier.hr_bpm, Some(165.0));
    assert_eq!(earlier.entered_on, "2026-11-03");
    assert!(!earlier.in_force);

    // And the history reads as a list of facts, newest first.
    let history = store.threshold_choices()?;
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].id, later);
    assert_eq!(history[1].id, first);

    drop(dir);
    Ok(())
}

#[test]
fn a_choice_with_no_value_at_all_is_refused() -> Result<()> {
    let (dir, store) = common::store("threshold-empty")?;
    let mut empty = choice();
    empty.hr_bpm = None;

    assert!(store.write_threshold_choice(&empty, true).is_err());
    assert_eq!(store.count_threshold_observations()?, 0);

    drop(dir);
    Ok(())
}

#[test]
fn the_kind_a_choice_carries_is_read_from_its_values() -> Result<()> {
    let (dir, store) = common::store("threshold-kind")?;

    let heart_rate = store.write_threshold_choice(&choice(), true)?;
    assert_eq!(
        store
            .threshold_choice(heart_rate)?
            .expect("the choice is there")
            .kind(),
        Some(ThresholdKind::HeartRate)
    );

    // A pace band on its own.
    let mut band = choice();
    band.hr_bpm = None;
    band.pace_low_s_per_km = Some(256.0);
    band.pace_high_s_per_km = Some(261.0);
    band.source = "race".to_string();
    let pace = store.write_threshold_choice(&band, false)?;
    assert_eq!(
        store
            .threshold_choice(pace)?
            .expect("the choice is there")
            .kind(),
        Some(ThresholdKind::PaceBand)
    );

    // A pair: a heart rate and a pace band together.
    let mut pair = choice();
    pair.pace_low_s_per_km = Some(246.0);
    pair.pace_high_s_per_km = Some(252.0);
    pair.source = "field_test".to_string();
    let both = store.write_threshold_choice(&pair, false)?;
    assert_eq!(
        store
            .threshold_choice(both)?
            .expect("the choice is there")
            .kind(),
        Some(ThresholdKind::Both)
    );

    drop(dir);
    Ok(())
}

#[test]
fn the_choices_of_one_base_are_read_apart() -> Result<()> {
    let (dir, store) = common::store("threshold-base")?;
    store.write_threshold_choice(&choice(), true)?;

    let mut other = choice();
    other.base = Some("eighty_twenty".to_string());
    other.entered_on = "2027-01-10".to_string();
    store.write_threshold_choice(&other, false)?;

    let pfitzinger = store.threshold_choices_for_base("pfitzinger")?;
    assert_eq!(pfitzinger.len(), 1);
    assert_eq!(pfitzinger[0].base.as_deref(), Some("pfitzinger"));

    let other = store.threshold_choices_for_base("eighty_twenty")?;
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].base.as_deref(), Some("eighty_twenty"));

    drop(dir);
    Ok(())
}

#[test]
fn deleting_everything_clears_the_choices() -> Result<()> {
    let (dir, store) = common::store("threshold-delete")?;
    store.write_threshold_choice(&choice(), true)?;
    assert_eq!(store.count_threshold_observations()?, 1);

    store.delete_everything()?;

    assert_eq!(store.count_threshold_observations()?, 0);
    assert!(store.threshold_in_force()?.is_none());

    drop(dir);
    Ok(())
}
