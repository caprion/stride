//! The `wellness_daily` repository.

mod common;

use stride_store::Result;

#[test]
fn a_day_is_written_and_read_back() -> Result<()> {
    let (dir, store) = common::store("wellness-round-trip")?;
    store.upsert_wellness_daily(&common::wellness("2026-05-10"))?;

    let read = store.wellness_daily("2026-05-10")?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the day was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.date, "2026-05-10");
    assert_eq!(read.sleep_s, Some(22_200.0));
    assert_eq!(read.deep_sleep_s, Some(5_400.0));
    assert_eq!(read.rem_s, Some(4_800.0));
    assert_eq!(read.light_sleep_s, Some(12_000.0));
    assert_eq!(read.awake_s, Some(1_800.0));
    assert_eq!(read.hrv_ms, Some(48.0));
    assert_eq!(read.resting_hr, Some(63.0));
    assert_eq!(read.stress_avg, Some(28.0));
    assert_eq!(read.source, "garmin_personal");
    drop(dir);
    Ok(())
}

#[test]
fn a_day_is_written_once() -> Result<()> {
    let (dir, store) = common::store("wellness-upsert")?;
    store.upsert_wellness_daily(&common::wellness("2026-05-10"))?;

    let mut again = common::wellness("2026-05-10");
    again.sleep_s = Some(21_000.0);
    again.stress_avg = None;
    store.upsert_wellness_daily(&again)?;

    assert_eq!(store.count_wellness_daily()?, 1);
    let read = store.wellness_daily("2026-05-10")?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the day was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.sleep_s, Some(21_000.0));
    assert_eq!(read.stress_avg, None);
    drop(dir);
    Ok(())
}

#[test]
fn the_wellness_of_a_range_is_readable() -> Result<()> {
    let (dir, store) = common::store("wellness-range")?;
    for day in 1..=5 {
        store.upsert_wellness_daily(&common::wellness(&format!("2026-05-0{day}")))?;
    }
    store.upsert_wellness_daily(&common::wellness("2026-04-30"))?;

    let range = store.wellness_between("2026-05-02", "2026-05-04")?;
    assert_eq!(range.len(), 3);
    assert_eq!(range[0].date, "2026-05-02");
    assert_eq!(range[1].date, "2026-05-03");
    assert_eq!(range[2].date, "2026-05-04");

    // The window the direction band is computed over: the last thirty nights
    // before the day itself.
    let before = store.wellness_before("2026-05-05", 30)?;
    assert_eq!(before.len(), 5);
    assert_eq!(before[0].date, "2026-04-30");
    assert_eq!(before[4].date, "2026-05-04");

    let latest = store.latest_wellness("2026-05-05")?;
    assert_eq!(latest.map(|day| day.date), Some("2026-05-05".to_string()));
    drop(dir);
    Ok(())
}

#[test]
fn a_day_with_no_reading_is_absent_and_not_zero() -> Result<()> {
    let (dir, store) = common::store("wellness-no-value")?;
    let mut day = common::wellness("2026-05-10");
    day.hrv_ms = None;
    day.resting_hr = None;
    day.stress_avg = None;
    store.upsert_wellness_daily(&day)?;

    let read = store.wellness_daily("2026-05-10")?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the day was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.hrv_ms, None);
    assert_eq!(read.resting_hr, None);
    assert_eq!(read.stress_avg, None);
    assert_eq!(read.sleep_s, Some(22_200.0));
    drop(dir);
    Ok(())
}

#[test]
fn a_provider_checkpoint_preserves_missing_fields_and_records_no_data_days() -> Result<()> {
    let (_dir, store) = common::store("wellness-checkpoint")?;
    let mut row = common::wellness("2026-05-10");
    store.upsert_wellness_daily(&row)?;
    let mut checkpoint = stride_store::SyncState {
        kind: "wellness".to_string(),
        cursor: Some("checked-2026-05-10".to_string()),
        last_success_at: None,
        token_expires_at: None,
        updated_at: row.updated_at,
    };
    row.hrv_ms = None;
    row.sleep_s = Some(21_000.0);
    store.write_wellness_checkpoint(Some(&row), &checkpoint)?;
    let read = store.wellness_daily("2026-05-10")?;
    assert_eq!(read.as_ref().and_then(|row| row.hrv_ms), Some(48.0));
    assert_eq!(read.as_ref().and_then(|row| row.sleep_s), Some(21_000.0));
    checkpoint.cursor = Some("checked-2026-05-09".to_string());
    store.write_wellness_checkpoint(None, &checkpoint)?;
    assert_eq!(store.count_wellness_daily()?, 1);
    assert_eq!(
        store.sync_state("wellness")?.and_then(|state| state.cursor),
        checkpoint.cursor
    );
    checkpoint.kind = "activities".to_string();
    row.sleep_s = Some(1.0);
    assert!(store
        .write_wellness_checkpoint(Some(&row), &checkpoint)
        .is_err());
    assert_eq!(
        store
            .wellness_daily("2026-05-10")?
            .and_then(|row| row.sleep_s),
        Some(21_000.0)
    );
    Ok(())
}

#[test]
fn a_failed_checkpoint_rolls_back_the_wellness_write() -> Result<()> {
    let (_dir, store) = common::store("wellness-checkpoint-rollback")?;
    let row = common::wellness("2026-05-10");
    store.with_connection(|connection| {
        connection
            .execute_batch(
                "CREATE TRIGGER refuse_checkpoint BEFORE INSERT ON sync_state
             BEGIN SELECT RAISE(ABORT, 'no checkpoint'); END;",
            )
            .map_err(|error| stride_store::StoreError::query("the trigger", error))
    })?;
    let checkpoint = stride_store::SyncState {
        kind: "wellness".to_string(),
        cursor: Some("checked".to_string()),
        last_success_at: None,
        token_expires_at: None,
        updated_at: row.updated_at,
    };
    assert!(store
        .write_wellness_checkpoint(Some(&row), &checkpoint)
        .is_err());
    assert!(store.wellness_daily("2026-05-10")?.is_none());
    assert!(store.sync_state("wellness")?.is_none());
    Ok(())
}
