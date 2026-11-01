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

#[test]
fn a_goal_is_set_and_taken_away() -> Result<()> {
    let (dir, store) = common::store("notes-set-goal")?;
    let id = store.set_goal(Some(40.0), common::instant())?;
    assert!(id.is_some());
    assert_eq!(store.goal_km()?, Some(40.0));

    // A second goal replaces the first: there is one goal, and it is the
    // one that was written last.
    store.set_goal(Some(50.0), common::instant())?;
    assert_eq!(store.goal_km()?, Some(50.0));
    assert_eq!(store.count_notes()?, 1);

    // A blank goal is no goal: the row goes, and nothing reads as a zero.
    store.set_goal(None, common::instant())?;
    assert_eq!(store.goal_km()?, None);
    assert_eq!(store.count_notes()?, 0);
    drop(dir);
    Ok(())
}

#[test]
fn a_pause_is_set_and_the_one_in_force_is_found() -> Result<()> {
    let (dir, store) = common::store("notes-set-pause")?;
    store.set_pause(
        Some("2026-05-10"),
        Some("2026-05-14"),
        Some("travel"),
        common::instant(),
    )?;

    let in_force = store.pause_in_force("2026-05-12")?;
    let in_force = match in_force {
        Some(pause) => pause,
        None => {
            return Err(stride_store::StoreError::Invariant(
                "the pause was not in force".to_string(),
            ))
        }
    };
    assert_eq!(in_force.text, Some("travel".to_string()));
    assert_eq!(in_force.start_date, Some("2026-05-10".to_string()));

    // A day outside the pause is not in it.
    assert!(store.pause_in_force("2026-05-20")?.is_none());

    // A second pause replaces the first.
    store.set_pause(
        Some("2026-06-01"),
        Some("2026-06-14"),
        Some("sick"),
        common::instant(),
    )?;
    assert_eq!(store.pauses()?.len(), 1);
    assert_eq!(store.pauses()?[0].text, Some("sick".to_string()));

    // And the pause can be taken away.
    store.set_pause(None, None, None, common::instant())?;
    assert_eq!(store.pauses()?.len(), 0);
    drop(dir);
    Ok(())
}

#[test]
fn a_run_carries_one_note() -> Result<()> {
    let (dir, store) = common::store("notes-set-note")?;
    let activity_id = store.upsert_activity(&common::activity("run-1", "2026-05-10"))?;
    store.set_note(activity_id, Some("the first note"), common::instant())?;
    store.set_note(activity_id, Some("the second note"), common::instant())?;

    let notes = store.notes_for_activity(activity_id)?;
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].text, Some("the second note".to_string()));

    // An empty note takes the note away.
    store.set_note(activity_id, None, common::instant())?;
    assert_eq!(store.notes_for_activity(activity_id)?.len(), 0);
    drop(dir);
    Ok(())
}
