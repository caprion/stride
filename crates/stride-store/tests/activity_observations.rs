//! The source's own facts about a run.
//!
//! The same shape `wellness_observations` has, at the run's grain. Three
//! rules are proved here, and all three are about honesty:
//!
//! * a field the source never sent has **no row at all**;
//! * a field the source sent as null has a row whose value is null — never
//!   a zero for "no time in that zone";
//! * a later read never deletes a row — it replaces the value it carried and
//!   keeps the reading time of the read that carried it, so a failed read
//!   leaves the previous value and its reading time in place.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::Duration;
use stride_store::{NewActivityObservation, Result};

/// An observation to write, with the values a test wants.
fn observation(field: &str, value: Option<f64>) -> NewActivityObservation {
    NewActivityObservation {
        activity_id: 1,
        source: "garmin_personal".to_string(),
        endpoint: "/activitylist-service/activities/search/activities".to_string(),
        field: field.to_string(),
        ordinal: 0,
        value,
        text_value: None,
        unit: Some("s".to_string()),
        observed_at: Some(common::instant()),
        retrieved_at: common::instant(),
    }
}

#[test]
fn an_observation_is_kept_with_its_run_its_field_and_its_reading_time() -> Result<()> {
    let (dir, store) = common::store("activity-observations-write")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.write_activity_observations(&[observation("hrTimeInZone_3", Some(359.0))])?;

    let stored = store.activity_observations(1)?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].field, "hrTimeInZone_3");
    assert_eq!(stored[0].value, Some(359.0));
    assert_eq!(stored[0].unit.as_deref(), Some("s"));
    assert_eq!(
        stored[0].endpoint,
        "/activitylist-service/activities/search/activities"
    );
    assert_eq!(stored[0].retrieved_at, common::instant());

    drop(dir);
    Ok(())
}

#[test]
fn a_field_the_source_never_sent_has_no_row_at_all() -> Result<()> {
    let (dir, store) = common::store("activity-observations-absent")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.write_activity_observations(&[observation("hrTimeInZone_3", Some(359.0))])?;

    // The next read carries one field, and not the other.
    let mut second = observation("hrTimeInZone_4", Some(120.0));
    second.retrieved_at = common::instant() + Duration::days(1);
    store.write_activity_observations(&[second])?;

    let stored = store.activity_observations(1)?;
    assert_eq!(stored.len(), 2);
    // The field the read did not carry is still there, with the day it was
    // read: a later read never deletes a row.
    let third = stored
        .iter()
        .find(|row| row.field == "hrTimeInZone_3")
        .expect("the earlier reading is still there");
    assert_eq!(third.value, Some(359.0));
    assert_eq!(third.retrieved_at, common::instant());

    drop(dir);
    Ok(())
}

#[test]
fn a_field_the_source_sent_as_null_is_not_an_absent_field() -> Result<()> {
    let (dir, store) = common::store("activity-observations-null")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.write_activity_observations(&[observation("hrTimeInZone_3", Some(359.0))])?;

    // The source sent the field, and its value was null.
    let mut second = observation("hrTimeInZone_3", None);
    second.retrieved_at = common::instant() + Duration::days(1);
    store.write_activity_observations(&[second])?;

    let stored = store.activity_observations(1)?;
    assert_eq!(stored.len(), 1);
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
    let (dir, store) = common::store("activity-observations-replace")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.write_activity_observations(&[observation("hrTimeInZone_3", Some(359.0))])?;

    let mut second = observation("hrTimeInZone_3", Some(400.0));
    second.retrieved_at = common::instant() + Duration::days(1);
    store.write_activity_observations(&[second])?;

    let stored = store.activity_observations(1)?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].value, Some(400.0));
    assert_eq!(
        stored[0].retrieved_at,
        common::instant() + Duration::days(1)
    );

    drop(dir);
    Ok(())
}

#[test]
fn the_observations_of_a_run_are_read_by_their_own_key() -> Result<()> {
    let (dir, store) = common::store("activity-observations-key")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.write_activity_observations(&[observation("hrTimeInZone_3", Some(359.0))])?;

    let found = store
        .activity_observation(1, "garmin_personal", "hrTimeInZone_3", 0)?
        .expect("the observation is there");
    assert_eq!(found.value, Some(359.0));

    // A field that was never sent has no row.
    assert!(store
        .activity_observation(1, "garmin_personal", "hrTimeInZone_5", 0)?
        .is_none());

    drop(dir);
    Ok(())
}

#[test]
fn the_observations_of_a_window_of_days_are_read_together() -> Result<()> {
    let (dir, store) = common::store("activity-observations-window")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.upsert_activity(&common::activity("run-2", "2026-05-12"))?;
    store.write_activity_observations(&[
        observation("hrTimeInZone_3", Some(359.0)),
        NewActivityObservation {
            activity_id: 2,
            ..observation("hrTimeInZone_3", Some(120.0))
        },
    ])?;

    let inside = store.activity_observations_between("2026-05-09", "2026-05-11")?;
    assert_eq!(inside.len(), 1);
    assert_eq!(inside[0].activity_id, 1);

    let outside = store.activity_observations_between("2026-05-13", "2026-05-14")?;
    assert!(outside.is_empty());

    drop(dir);
    Ok(())
}

#[test]
fn deleting_everything_clears_the_observations() -> Result<()> {
    let (dir, store) = common::store("activity-observations-delete")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.write_activity_observations(&[observation("hrTimeInZone_3", Some(359.0))])?;
    assert_eq!(store.count_activity_observations()?, 1);

    store.delete_everything()?;

    assert_eq!(store.count_activity_observations()?, 0);
    assert!(store.activity_observations(1)?.is_empty());

    drop(dir);
    Ok(())
}
