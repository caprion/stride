//! The source's own wellness observations.
//!
//! Three rules are proved here, and all three are about honesty:
//!
//! * a field the source never sent has **no row at all**;
//! * a field the source sent as null has a row whose value is null;
//! * a later read never deletes a row — it replaces the value it carried and
//!   keeps the reading time of the read that carried it, so a failed read
//!   leaves the previous value and its reading time in place.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::Duration;
use stride_store::{NewWellnessObservation, Result};

/// An observation to write, with the values a test wants.
fn observation(field: &str, value: Option<f64>) -> NewWellnessObservation {
    NewWellnessObservation {
        date: "2026-05-10".to_string(),
        source: "garmin_personal".to_string(),
        endpoint: "/usersummary-service/usersummary/daily".to_string(),
        field: field.to_string(),
        ordinal: 0,
        value,
        text_value: None,
        unit: Some("s".to_string()),
        observed_at: None,
        retrieved_at: common::instant(),
    }
}

#[test]
fn an_observation_is_kept_with_its_day_and_its_reading_time() -> Result<()> {
    let (dir, store) = common::store("observations-write")?;
    store.write_wellness_observations(&[observation("lowStressDuration", Some(1_800.0))])?;

    let stored = store.wellness_observations("2026-05-10")?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].field, "lowStressDuration");
    assert_eq!(stored[0].value, Some(1_800.0));
    assert_eq!(stored[0].unit.as_deref(), Some("s"));
    assert_eq!(stored[0].date, "2026-05-10");
    assert_eq!(stored[0].endpoint, "/usersummary-service/usersummary/daily");
    assert_eq!(stored[0].retrieved_at, common::instant());

    drop(dir);
    Ok(())
}

#[test]
fn a_field_the_source_never_sent_has_no_row_at_all() -> Result<()> {
    let (dir, store) = common::store("observations-absent")?;
    store.write_wellness_observations(&[observation("lowStressDuration", Some(1_800.0))])?;

    // The next read carries one field, and not the other.
    let mut second = observation("mediumStressDuration", Some(2_400.0));
    second.retrieved_at = common::instant() + Duration::days(1);
    store.write_wellness_observations(&[second])?;

    let stored = store.wellness_observations("2026-05-10")?;
    assert_eq!(stored.len(), 2);
    // The field the read did not carry is still there, with the day it was
    // read: a later read never deletes a row.
    let low = stored
        .iter()
        .find(|row| row.field == "lowStressDuration")
        .expect("the earlier reading is still there");
    assert_eq!(low.value, Some(1_800.0));
    assert_eq!(low.retrieved_at, common::instant());

    drop(dir);
    Ok(())
}

#[test]
fn a_field_the_source_sent_as_null_is_not_an_absent_field() -> Result<()> {
    let (dir, store) = common::store("observations-null")?;
    store.write_wellness_observations(&[observation("lowStressDuration", Some(1_800.0))])?;

    // The source sent the field, and its value was null.
    let mut second = observation("lowStressDuration", None);
    second.retrieved_at = common::instant() + Duration::days(1);
    store.write_wellness_observations(&[second])?;

    let stored = store.wellness_observations("2026-05-10")?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].field, "lowStressDuration");
    assert_eq!(stored[0].value, None);
    assert_eq!(stored[0].text_value, None);
    assert_eq!(
        stored[0].retrieved_at,
        common::instant() + Duration::days(1)
    );

    drop(dir);
    Ok(())
}

#[test]
fn a_later_read_replaces_the_value_and_keeps_the_new_reading_time() -> Result<()> {
    let (dir, store) = common::store("observations-replace")?;
    store.write_wellness_observations(&[observation("lowStressDuration", Some(1_800.0))])?;

    let mut second = observation("lowStressDuration", Some(2_400.0));
    second.retrieved_at = common::instant() + Duration::days(1);
    store.write_wellness_observations(&[second])?;

    let stored = store.wellness_observations("2026-05-10")?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].value, Some(2_400.0));
    assert_eq!(
        stored[0].retrieved_at,
        common::instant() + Duration::days(1)
    );

    drop(dir);
    Ok(())
}

#[test]
fn a_text_value_is_kept_as_text() -> Result<()> {
    let (dir, store) = common::store("observations-text")?;
    let mut row = observation("stressQualifier", None);
    row.text_value = Some("BALANCED".to_string());
    store.write_wellness_observations(&[row])?;

    let stored = store.wellness_observations("2026-05-10")?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].value, None);
    assert_eq!(stored[0].text_value.as_deref(), Some("BALANCED"));

    drop(dir);
    Ok(())
}

#[test]
fn the_same_field_of_two_days_is_two_rows() -> Result<()> {
    let (dir, store) = common::store("observations-days")?;
    let mut other = observation("lowStressDuration", Some(900.0));
    other.date = "2026-05-11".to_string();
    store.write_wellness_observations(&[observation("lowStressDuration", Some(1_800.0)), other])?;

    assert_eq!(store.wellness_observations("2026-05-10")?.len(), 1);
    assert_eq!(store.wellness_observations("2026-05-11")?.len(), 1);
    assert_eq!(store.count_wellness_observations()?, 2);

    drop(dir);
    Ok(())
}

#[test]
fn the_repeated_items_of_one_field_keep_their_own_rows() -> Result<()> {
    let (dir, store) = common::store("observations-ordinal")?;
    let mut first = observation("bodyBatteryValuesArray", Some(71.0));
    first.ordinal = 0;
    let mut second = observation("bodyBatteryValuesArray", Some(69.0));
    second.ordinal = 1;
    store.write_wellness_observations(&[first, second])?;

    let stored = store.wellness_observations("2026-05-10")?;
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].ordinal, 0);
    assert_eq!(stored[0].value, Some(71.0));
    assert_eq!(stored[1].ordinal, 1);
    assert_eq!(stored[1].value, Some(69.0));

    drop(dir);
    Ok(())
}

#[test]
fn deleting_everything_clears_the_observations() -> Result<()> {
    let (dir, store) = common::store("observations-delete")?;
    store.write_wellness_observations(&[observation("lowStressDuration", Some(1_800.0))])?;
    assert_eq!(store.count_wellness_observations()?, 1);

    store.delete_everything()?;

    assert_eq!(store.count_wellness_observations()?, 0);
    assert!(store.wellness_observations("2026-05-10")?.is_empty());

    drop(dir);
    Ok(())
}
