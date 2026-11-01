//! The sync cursor.
//!
//! The rule: **a cursor advances only after a successful parse — never
//! before.** A failed sync leaves the cursor where it was.

mod common;

use stride_store::Result;

#[test]
fn a_cursor_starts_where_the_last_one_stopped() -> Result<()> {
    let (dir, store) = common::store("sync-start")?;
    assert!(store.sync_state("activities")?.is_none());

    let mut cursor = store.begin_sync("activities")?;
    assert_eq!(cursor.current(), None);
    cursor.advance("the first cursor");
    cursor.commit()?;

    let cursor = store.begin_sync("activities")?;
    assert_eq!(cursor.current(), Some("the first cursor"));
    drop(dir);
    Ok(())
}

#[test]
fn a_failed_sync_leaves_the_cursor_where_it_was() -> Result<()> {
    let (dir, store) = common::store("sync-failure")?;

    {
        let mut cursor = store.begin_sync("activities")?;
        cursor.advance("the good cursor");
        cursor.commit_at(common::instant())?;
    }

    // A sync that fails: the cursor is advanced in memory, and the guard is
    // dropped instead of committed.
    {
        let mut cursor = store.begin_sync("activities")?;
        assert_eq!(cursor.current(), Some("the good cursor"));
        cursor.advance("the cursor of a sync that failed");
        assert_eq!(cursor.pending(), Some("the cursor of a sync that failed"));
        // The guard is dropped here: the parse failed, so nothing is written.
    }

    let state = store.sync_state("activities")?;
    let state = match state {
        Some(state) => state,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the state was not written".to_string(),
            ))
        }
    };
    assert_eq!(state.cursor, Some("the good cursor".to_string()));
    assert_eq!(state.last_success_at, Some(common::instant()));

    drop(dir);
    Ok(())
}

#[test]
fn a_committed_cursor_is_the_next_one_s_starting_point() -> Result<()> {
    let (dir, store) = common::store("sync-commit")?;
    let mut cursor = store.begin_sync("activities")?;
    cursor.advance("page two");
    cursor.commit_at(common::instant())?;

    let state = store.sync_state("activities")?;
    let state = match state {
        Some(state) => state,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the state was not written".to_string(),
            ))
        }
    };
    assert_eq!(state.cursor, Some("page two".to_string()));
    assert_eq!(state.last_success_at, Some(common::instant()));
    assert_eq!(state.updated_at, common::instant());
    drop(dir);
    Ok(())
}

#[test]
fn a_commit_with_nothing_advanced_leaves_the_cursor_alone() -> Result<()> {
    let (dir, store) = common::store("sync-nothing")?;
    let mut cursor = store.begin_sync("activities")?;
    cursor.advance("the first cursor");
    cursor.commit()?;

    let cursor = store.begin_sync("activities")?;
    cursor.commit()?;

    let state = store.sync_state("activities")?;
    let state = match state {
        Some(state) => state,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the state was not written".to_string(),
            ))
        }
    };
    assert_eq!(state.cursor, Some("the first cursor".to_string()));
    drop(dir);
    Ok(())
}

#[test]
fn every_kind_of_data_has_its_own_cursor() -> Result<()> {
    let (dir, store) = common::store("sync-kinds")?;
    for kind in ["activities", "wellness", "daily_summaries"] {
        let mut cursor = store.begin_sync(kind)?;
        cursor.advance(format!("the {kind} cursor"));
        cursor.commit()?;
    }

    let states = store.all_sync_state()?;
    assert_eq!(states.len(), 3);
    let activities = store.sync_state("activities")?;
    let activities = match activities {
        Some(state) => state,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the state was not written".to_string(),
            ))
        }
    };
    assert_eq!(activities.cursor, Some("the activities cursor".to_string()));
    drop(dir);
    Ok(())
}

#[test]
fn the_token_expiry_is_remembered() -> Result<()> {
    let (dir, store) = common::store("sync-token")?;
    store.record_token_expiry("activities", common::instant(), common::instant())?;

    let state = store.sync_state("activities")?;
    let state = match state {
        Some(state) => state,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the state was not written".to_string(),
            ))
        }
    };
    assert_eq!(state.token_expires_at, Some(common::instant()));
    assert_eq!(state.cursor, None);
    drop(dir);
    Ok(())
}

#[test]
fn a_cursor_can_be_cleared() -> Result<()> {
    let (dir, store) = common::store("sync-clear")?;
    let mut cursor = store.begin_sync("activities")?;
    cursor.advance("a cursor");
    cursor.commit()?;

    store.clear_sync_cursor("activities")?;
    let state = store.sync_state("activities")?;
    let state = match state {
        Some(state) => state,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the state was not written".to_string(),
            ))
        }
    };
    assert_eq!(state.cursor, None);
    drop(dir);
    Ok(())
}
