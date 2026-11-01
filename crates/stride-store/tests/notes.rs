//! The `notes` repository.

mod common;

use stride_store::{NewNote, Result};

/// A note to write.
fn note(kind: &str, activity_id: Option<i64>, text: &str) -> NewNote {
    NewNote {
        kind: kind.to_string(),
        activity_id,
        text: Some(text.to_string()),
        value: None,
        start_date: None,
        end_date: None,
        created_at: common::instant(),
    }
}

#[test]
fn a_note_is_written_and_read_back() -> Result<()> {
    let (dir, store) = common::store("notes-round-trip")?;
    let activity_id = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    let id = store.upsert_note(&note("note", Some(activity_id), "the legs were heavy"))?;

    let read = store.note(id)?;
    let read = match read {
        Some(read) => read,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the note was not written".to_string(),
            ))
        }
    };
    assert_eq!(read.kind, "note");
    assert_eq!(read.activity_id, Some(activity_id));
    assert_eq!(read.text, Some("the legs were heavy".to_string()));
    assert_eq!(read.value, None);
    drop(dir);
    Ok(())
}

#[test]
fn the_same_note_written_twice_is_one_note() -> Result<()> {
    let (dir, store) = common::store("notes-upsert")?;
    let activity_id = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    let first = store.upsert_note(&note("note", Some(activity_id), "the legs were heavy"))?;
    let second = store.upsert_note(&note("note", Some(activity_id), "the legs were heavy"))?;

    assert_eq!(first, second, "the note was written twice");
    assert_eq!(store.count_notes()?, 1);
    drop(dir);
    Ok(())
}

#[test]
fn the_notes_of_a_run_and_of_a_kind_are_readable() -> Result<()> {
    let (dir, store) = common::store("notes-queries")?;
    let first = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    let second = store.upsert_activity(&common::activity("run-2", "2026-05-11"))?;
    store.upsert_note(&note("note", Some(first), "about the first"))?;
    store.upsert_note(&note("note", Some(second), "about the second"))?;

    assert_eq!(store.notes_for_activity(first)?.len(), 1);
    assert_eq!(store.notes_of_kind("note")?.len(), 2);
    assert_eq!(store.notes()?.len(), 2);
    drop(dir);
    Ok(())
}

#[test]
fn the_goal_and_the_pauses_are_readable() -> Result<()> {
    let (dir, store) = common::store("notes-goal")?;
    store.upsert_note(&NewNote {
        kind: "goal".to_string(),
        activity_id: None,
        text: None,
        value: Some(60.0),
        start_date: None,
        end_date: None,
        created_at: common::instant(),
    })?;
    store.upsert_note(&NewNote {
        kind: "pause".to_string(),
        activity_id: None,
        text: Some("a holiday".to_string()),
        value: None,
        start_date: Some("2026-06-01".to_string()),
        end_date: Some("2026-06-14".to_string()),
        created_at: common::instant(),
    })?;

    assert_eq!(store.goal_km()?, Some(60.0));
    let pauses = store.pauses()?;
    assert_eq!(pauses.len(), 1);
    assert_eq!(pauses[0].text, Some("a holiday".to_string()));
    assert_eq!(pauses[0].start_date, Some("2026-06-01".to_string()));
    assert_eq!(pauses[0].end_date, Some("2026-06-14".to_string()));
    drop(dir);
    Ok(())
}

#[test]
fn an_unknown_kind_is_refused() -> Result<()> {
    let (dir, store) = common::store("notes-refused")?;
    assert!(store
        .upsert_note(&note("something else", None, "a note"))
        .is_err());
    assert_eq!(store.count_notes()?, 0);
    drop(dir);
    Ok(())
}

#[test]
fn a_note_can_be_deleted() -> Result<()> {
    let (dir, store) = common::store("notes-delete")?;
    let id = store.upsert_note(&note("note", None, "a note"))?;
    assert_eq!(store.delete_note(id)?, 1);
    assert!(store.note(id)?.is_none());
    assert_eq!(store.count_notes()?, 0);
    drop(dir);
    Ok(())
}

#[test]
fn a_note_about_a_run_goes_when_the_run_goes() -> Result<()> {
    let (dir, store) = common::store("notes-cascade")?;
    let activity_id = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.upsert_note(&note("note", Some(activity_id), "about the run"))?;

    // The foreign key cascades: the note goes with the run.
    store.with_connection(|connection| {
        connection
            .execute(
                "DELETE FROM activities WHERE id = ?1",
                rusqlite::params![activity_id],
            )
            .map_err(|error| stride_store::StoreError::query("delete the run", error))
    })?;
    assert_eq!(store.count_notes()?, 0);
    drop(dir);
    Ok(())
}
