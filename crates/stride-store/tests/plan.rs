//! The `plan_blocks` and `plan_sessions` repositories.
//!
//! The rule these tests exist for: **exactly one active block per plan
//! source**. The old implementation counted every block it had ever
//! imported, so a plan imported twice looked like two plans.

mod common;

use stride_store::{NewPlanBlock, NewPlanSession, Result};

/// A block to write.
fn block(name: &str, source: &str, status: &str) -> NewPlanBlock {
    NewPlanBlock {
        name: name.to_string(),
        source: source.to_string(),
        start_date: "2026-05-01".to_string(),
        end_date: None,
        goal: Some("a goal".to_string()),
        status: status.to_string(),
        notes: None,
        created_at: common::instant(),
    }
}

/// A session to write.
fn planned(block_id: i64, date: &str, kind: &str) -> NewPlanSession {
    NewPlanSession {
        block_id,
        date: date.to_string(),
        session_type: kind.to_string(),
        title: Some(format!("a {kind} session")),
        details: None,
        target_distance_m: Some(10_000.0),
        pace_low_s_km: Some(300.0),
        pace_high_s_km: Some(330.0),
    }
}

#[test]
fn a_block_is_written_and_read_back() -> Result<()> {
    let (dir, store) = common::store("plan-round-trip")?;
    let id = store.upsert_plan_block(&block("a block", "typed", "active"))?;

    let read = store.plan_block(id)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the block was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.name, "a block");
    assert_eq!(read.source, "typed");
    assert_eq!(read.start_date, "2026-05-01");
    assert_eq!(read.end_date, None);
    assert_eq!(read.goal, Some("a goal".to_string()));
    assert_eq!(read.status, "active");
    drop(dir);
    Ok(())
}

#[test]
fn there_is_one_active_block_per_source() -> Result<()> {
    let (dir, store) = common::store("plan-one-active")?;

    let first = store.upsert_plan_block(&block("the first block", "baseline", "active"))?;
    assert_eq!(store.count_active_plan_blocks("baseline")?, 1);

    // A second block of the same source, imported again: the first one is
    // archived, and the second one is the one in force.
    let second = store.upsert_plan_block(&block("the second block", "baseline", "active"))?;
    assert_eq!(store.count_active_plan_blocks("baseline")?, 1);

    let active = store.active_plan_block("baseline")?;
    let active = match active {
        Some(active) => active,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "there is no active block".to_string(),
            ))
        }
    };
    assert_eq!(active.id, second);
    assert_eq!(active.name, "the second block");

    // The first one is still there — archived, not deleted.
    let archived = store.plan_block(first)?;
    let archived = match archived {
        Some(archived) => archived,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the first block was deleted".to_string(),
            ))
        }
    };
    assert_eq!(archived.status, "archived");

    // Both are in the source's list of blocks. Only one of them is active.
    assert_eq!(store.plan_blocks_of_source("baseline")?.len(), 2);
    assert_eq!(store.count_active_plan_blocks("baseline")?, 1);

    // Another source has its own block, and it does not disturb this one.
    store.upsert_plan_block(&block("a prescribed block", "prescribed", "active"))?;
    assert_eq!(store.count_active_plan_blocks("prescribed")?, 1);
    assert_eq!(store.count_active_plan_blocks("baseline")?, 1);
    assert_eq!(store.plan_blocks()?.len(), 3);

    drop(dir);
    Ok(())
}

#[test]
fn the_same_block_imported_twice_is_one_block() -> Result<()> {
    let (dir, store) = common::store("plan-import-twice")?;
    let first = store.upsert_plan_block(&block("a block", "typed", "active"))?;
    let second = store.upsert_plan_block(&block("a block", "typed", "active"))?;

    assert_eq!(first, second, "the block was written twice");
    assert_eq!(store.plan_blocks()?.len(), 1);
    assert_eq!(store.count_active_plan_blocks("typed")?, 1);
    drop(dir);
    Ok(())
}

#[test]
fn an_archived_block_does_not_take_the_place_of_the_active_one() -> Result<()> {
    let (dir, store) = common::store("plan-archived")?;
    store.upsert_plan_block(&block("the active block", "typed", "active"))?;
    store.upsert_plan_block(&block("an old block", "typed", "archived"))?;

    assert_eq!(store.count_active_plan_blocks("typed")?, 1);
    let active = store.active_plan_block("typed")?;
    let active = match active {
        Some(active) => active,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "there is no active block".to_string(),
            ))
        }
    };
    assert_eq!(active.name, "the active block");
    drop(dir);
    Ok(())
}

#[test]
fn every_active_block_of_a_source_can_be_archived() -> Result<()> {
    let (dir, store) = common::store("plan-archive-all")?;
    store.upsert_plan_block(&block("a block", "typed", "active"))?;
    assert_eq!(store.archive_active_plan_blocks("typed")?, 1);
    assert_eq!(store.count_active_plan_blocks("typed")?, 0);
    assert!(store.active_plan_block("typed")?.is_none());
    drop(dir);
    Ok(())
}

#[test]
fn an_unknown_source_or_status_is_refused() -> Result<()> {
    let (dir, store) = common::store("plan-refused")?;
    assert!(store
        .upsert_plan_block(&block("a block", "something else", "active"))
        .is_err());
    assert!(store
        .upsert_plan_block(&block("a block", "typed", "something else"))
        .is_err());
    assert_eq!(store.plan_blocks()?.len(), 0);
    drop(dir);
    Ok(())
}

#[test]
fn a_session_is_written_and_read_back() -> Result<()> {
    let (dir, store) = common::store("plan-session")?;
    let block_id = store.upsert_plan_block(&block("a block", "typed", "active"))?;
    let id = store.upsert_plan_session(&planned(block_id, "2026-05-10", "easy"))?;

    let read = store.plan_session(id)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the session was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.block_id, block_id);
    assert_eq!(read.date, "2026-05-10");
    assert_eq!(read.session_type, "easy");
    assert_eq!(read.title, Some("a easy session".to_string()));
    assert_eq!(read.target_distance_m, Some(10_000.0));
    assert_eq!(read.pace_low_s_km, Some(300.0));
    assert_eq!(read.pace_high_s_km, Some(330.0));
    drop(dir);
    Ok(())
}

#[test]
fn the_same_session_imported_twice_is_one_session() -> Result<()> {
    let (dir, store) = common::store("plan-session-twice")?;
    let block_id = store.upsert_plan_block(&block("a block", "typed", "active"))?;
    let first = store.upsert_plan_session(&planned(block_id, "2026-05-10", "easy"))?;
    let second = store.upsert_plan_session(&planned(block_id, "2026-05-10", "easy"))?;

    assert_eq!(first, second, "the session was written twice");
    assert_eq!(store.count_plan_sessions()?, 1);
    drop(dir);
    Ok(())
}

#[test]
fn the_sessions_of_a_range_a_block_and_a_day_are_readable() -> Result<()> {
    let (dir, store) = common::store("plan-session-queries")?;
    let block_id = store.upsert_plan_block(&block("a block", "typed", "active"))?;
    store.upsert_plan_session(&planned(block_id, "2026-05-10", "easy"))?;
    store.upsert_plan_session(&planned(block_id, "2026-05-11", "tempo"))?;
    store.upsert_plan_session(&planned(block_id, "2026-05-20", "long"))?;

    assert_eq!(store.plan_sessions_for_block(block_id)?.len(), 3);
    assert_eq!(store.plan_sessions_for_day("2026-05-11")?.len(), 1);
    let range = store.plan_sessions_between("2026-05-10", "2026-05-15")?;
    assert_eq!(range.len(), 2);
    assert_eq!(range[0].date, "2026-05-10");
    assert_eq!(range[1].date, "2026-05-11");
    drop(dir);
    Ok(())
}

#[test]
fn a_session_of_an_archived_block_stays_with_its_block() -> Result<()> {
    let (dir, store) = common::store("plan-session-archived")?;
    let first = store.upsert_plan_block(&block("the first block", "typed", "active"))?;
    store.upsert_plan_session(&planned(first, "2026-05-10", "easy"))?;

    let second = store.upsert_plan_block(&block("the second block", "typed", "active"))?;
    store.upsert_plan_session(&planned(second, "2026-05-11", "easy"))?;

    // The first block is archived, and its session is still its session.
    assert_eq!(store.plan_sessions_for_block(first)?.len(), 1);
    assert_eq!(store.plan_sessions_for_block(second)?.len(), 1);
    assert_eq!(store.count_plan_sessions()?, 2);
    drop(dir);
    Ok(())
}
