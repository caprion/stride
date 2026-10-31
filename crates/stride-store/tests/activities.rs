//! The `activities` and `activity_samples` repositories.

mod common;

use stride_store::{NewActivitySamples, Result};

#[test]
fn a_run_is_written_and_read_back() -> Result<()> {
    let (dir, store) = common::store("activity-round-trip")?;
    let activity = common::activity("run-1", "2026-05-10");
    let id = store.upsert_activity(&activity)?;

    let read = store.activity(id)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the run was not written".to_string(),
            ))
        }
    };

    assert_eq!(read.source, "garmin_personal");
    assert_eq!(read.external_id, "run-1");
    assert_eq!(read.name, activity.name);
    assert_eq!(read.sport_type, "running");
    assert_eq!(read.local_date, "2026-05-10");
    assert_eq!(read.distance_m, 10_000.0);
    assert_eq!(read.moving_time_s, 3_000.0);
    assert_eq!(read.elapsed_time_s, 3_100.0);
    assert_eq!(read.elevation_gain_m, 42.0);
    assert_eq!(read.avg_hr, Some(148.0));
    assert_eq!(read.max_hr, Some(171.0));
    assert_eq!(read.fit_blob, activity.fit_blob);
    assert_eq!(read.start_time, activity.start_time);

    drop(dir);
    Ok(())
}

#[test]
fn the_hash_of_the_blob_is_the_hash_of_the_blob() -> Result<()> {
    let (dir, store) = common::store("activity-hash")?;
    let activity = common::activity("run-1", "2026-05-10");
    let id = store.upsert_activity(&activity)?;

    let digest = store.fit_sha256(id)?;
    let digest = match digest {
        Some(digest) => digest,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the hash was not written".to_string(),
            ))
        }
    };
    let expected = stride_store::hash::sha256_hex(&activity.fit_blob);
    assert_eq!(digest, expected);
    assert_eq!(digest.len(), 64);
    drop(dir);
    Ok(())
}

#[test]
fn a_missing_measurement_is_absent_and_not_zero() -> Result<()> {
    let (dir, store) = common::store("activity-no-value")?;
    let mut activity = common::activity("run-1", "2026-05-10");
    activity.avg_hr = None;
    activity.max_hr = None;
    let id = store.upsert_activity(&activity)?;

    let read = store.activity(id)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the run was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.avg_hr, None);
    assert_eq!(read.max_hr, None);
    drop(dir);
    Ok(())
}

#[test]
fn the_same_run_is_written_once() -> Result<()> {
    let (dir, store) = common::store("activity-upsert")?;
    let first = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;

    // The same run, with a different name and a different distance.
    let mut again = common::activity("run-1", "2026-05-10");
    again.name = "the same run, renamed".to_string();
    again.distance_m = 10_500.0;
    let second = store.upsert_activity(&again)?;

    assert_eq!(first, second, "the run was written twice");
    assert_eq!(store.count_activities()?, 1);

    let read = store.activity(first)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the run was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.name, "the same run, renamed");
    assert_eq!(read.distance_m, 10_500.0);
    drop(dir);
    Ok(())
}

#[test]
fn the_series_and_the_run_are_written_together_or_not_at_all() -> Result<()> {
    let (dir, store) = common::store("activity-all-or-nothing")?;

    // A trigger that refuses the samples, so that the write of the series
    // fails and the run has to be rolled back with it.
    store.with_connection(|connection| {
        connection
            .execute_batch(
                "CREATE TRIGGER refuse_samples BEFORE INSERT ON activity_samples
                 BEGIN SELECT RAISE(ABORT, 'no series today'); END;",
            )
            .map_err(|error| stride_store::StoreError::query("the trigger", error))
    })?;

    let series = NewActivitySamples {
        series: vec![1, 2, 3, 4],
        sha256: "a hash".to_string(),
        first_s: Some(0),
        last_s: Some(3),
    };

    let written =
        store.upsert_activity_with_samples(&common::activity("run-1", "2026-05-10"), Some(&series));
    assert!(written.is_err(), "the write should have failed");

    // The run is not there either: the two are one thing.
    assert_eq!(store.count_activities()?, 0);
    assert_eq!(store.count_activity_samples()?, 0);
    assert!(store
        .activity_by_external("garmin_personal", "run-1")?
        .is_none());

    drop(dir);
    Ok(())
}

#[test]
fn the_series_can_be_pruned_and_derived_again() -> Result<()> {
    let (dir, store) = common::store("activity-prune")?;
    let id = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;

    let series = NewActivitySamples {
        series: vec![9, 9, 9],
        sha256: "a hash".to_string(),
        first_s: Some(0),
        last_s: Some(2),
    };
    store.upsert_activity_with_samples(&common::activity("run-1", "2026-05-10"), Some(&series))?;

    let read = store.activity_samples(id)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the series was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.series, vec![9, 9, 9]);
    assert_eq!(read.first_s, Some(0));
    assert_eq!(read.last_s, Some(2));

    // Pruned: gone, but not lost — the FIT file is still there.
    assert_eq!(store.prune_activity_samples(id)?, 1);
    assert!(store.activity_samples(id)?.is_none());
    assert!(store.fit_blob(id)?.is_some());
    assert_eq!(store.count_activities()?, 1);

    drop(dir);
    Ok(())
}

#[test]
fn the_feed_pages_through_the_runs() -> Result<()> {
    let (dir, store) = common::store("activity-feed")?;
    for day in 1..=5 {
        store.upsert_activity(&common::activity(
            &format!("run-{day}"),
            &format!("2026-05-0{day}"),
        ))?;
    }

    let first = store.activity_feed(None, 2)?;
    assert_eq!(first.activities.len(), 2);
    assert_eq!(first.activities[0].local_date, "2026-05-05");
    assert_eq!(first.activities[1].local_date, "2026-05-04");
    let cursor = match &first.next_cursor {
        Some(cursor) => cursor.clone(),
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the feed did not offer a second page".to_string(),
            ))
        }
    };

    let second = store.activity_feed(Some(&cursor), 2)?;
    assert_eq!(second.activities.len(), 2);
    assert_eq!(second.activities[0].local_date, "2026-05-03");
    assert_eq!(second.activities[1].local_date, "2026-05-02");

    let third = store.activity_feed(Some(&second.next_cursor.unwrap_or(cursor.clone())), 2)?;
    assert_eq!(third.activities.len(), 1);
    assert_eq!(third.activities[0].local_date, "2026-05-01");
    assert!(
        third.next_cursor.is_none(),
        "the last page offered another one"
    );

    // Every run is visited exactly once.
    let mut seen: Vec<String> = Vec::new();
    let mut cursor = None;
    loop {
        let page = store.activity_feed(cursor.as_ref(), 2)?;
        for run in &page.activities {
            seen.push(run.local_date.clone());
        }
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    assert_eq!(seen.len(), 5);
    seen.sort();
    assert_eq!(
        seen,
        vec![
            "2026-05-01".to_string(),
            "2026-05-02".to_string(),
            "2026-05-03".to_string(),
            "2026-05-04".to_string(),
            "2026-05-05".to_string(),
        ]
    );

    drop(dir);
    Ok(())
}

#[test]
fn the_cursor_is_opaque_and_survives_a_round_trip() -> Result<()> {
    let cursor = stride_store::FeedCursor::new("2026-05-10", 42);
    let encoded = cursor.encode();
    assert!(
        !encoded.contains("2026"),
        "the cursor shows the date: {encoded}"
    );
    let decoded = stride_store::FeedCursor::decode(&encoded)?;
    assert_eq!(decoded, cursor);
    assert_eq!(decoded.local_date(), "2026-05-10");
    assert_eq!(decoded.id(), 42);

    // A cursor that this build did not write is an error, not a guess.
    assert!(stride_store::FeedCursor::decode("not a cursor").is_err());
    assert!(stride_store::FeedCursor::decode("").is_err());
    Ok(())
}

#[test]
fn the_runs_of_a_date_and_of_a_sport_are_readable() -> Result<()> {
    let (dir, store) = common::store("activity-queries")?;
    store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    let mut cycling = common::activity("ride-1", "2026-05-10");
    cycling.sport_type = "cycling".to_string();
    store.upsert_activity(&cycling)?;

    assert_eq!(store.activities_for_date("2026-05-10")?.len(), 2);
    assert_eq!(store.activities_of_sport("cycling")?.len(), 1);
    assert_eq!(
        store.activities_between("2026-05-01", "2026-05-31")?.len(),
        2
    );
    assert_eq!(
        store.activities_between("2026-06-01", "2026-06-30")?.len(),
        0
    );
    drop(dir);
    Ok(())
}
