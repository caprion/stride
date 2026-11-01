//! `notes`.
//!
//! The only free text the app stores: a note on a run, and the goal and
//! pause settings — the three inputs that change what the app says.
//!
//! There is no verdict in here and no score. A note says what the athlete
//! wrote; the app reads it and says nothing of its own.

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::datetimes::DbDateTime;
use crate::error::{Result, StoreError};
use crate::model::{note_kind, NewNote, Note};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `notes`.
const COLUMNS: &str = "id, kind, activity_id, text, value, start_date, end_date, created_at";

/// The kinds a note may have.
const KINDS: [&str; 3] = [note_kind::NOTE, note_kind::GOAL, note_kind::PAUSE];

/// Read one row of `notes`.
fn map_note(row: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get("id")?,
        kind: row.get("kind")?,
        activity_id: row.get("activity_id")?,
        text: row.get("text")?,
        value: row.get("value")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        created_at: row.get::<_, DbDateTime>("created_at")?.0,
    })
}

impl Store {
    /// Write one note.
    ///
    /// A note is identified by its kind, its run and its text — so the same
    /// note written twice is one note.
    ///
    /// Returns the note's primary key.
    pub fn upsert_note(&self, note: &NewNote) -> Result<i64> {
        if !KINDS.contains(&note.kind.as_str()) {
            return Err(StoreError::Invariant(format!(
                "the note kind {:?} is not one of {}",
                note.kind,
                KINDS.join(", ")
            )));
        }

        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let existing = transaction
            .query_row(
                "SELECT id FROM notes
                 WHERE kind = ?1
                   AND activity_id IS ?2
                   AND text IS ?3
                   AND start_date IS ?4
                   AND end_date IS ?5
                 ORDER BY id ASC
                 LIMIT 1",
                params![
                    note.kind,
                    note.activity_id,
                    note.text,
                    note.start_date,
                    note.end_date,
                ],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|source| StoreError::query("find the note", source))?;

        let id = match existing {
            Some(id) => {
                transaction
                    .execute(
                        "UPDATE notes SET value = ?2 WHERE id = ?1",
                        params![id, note.value],
                    )
                    .map_err(|source| StoreError::query("update the note", source))?;
                id
            }
            None => transaction
                .query_row(
                    "INSERT INTO notes (
                        kind, activity_id, text, value, start_date, end_date, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     RETURNING id",
                    params![
                        note.kind,
                        note.activity_id,
                        note.text,
                        note.value,
                        note.start_date,
                        note.end_date,
                        DbDateTime(note.created_at),
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|source| StoreError::query("insert the note", source))?,
        };

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// One note.
    pub fn note(&self, id: i64) -> Result<Option<Note>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM notes WHERE id = ?1");
        conn.query_row(&sql, params![id], map_note)
            .optional()
            .map_err(|source| StoreError::query("note by id", source))
    }

    /// Every note, oldest first.
    pub fn notes(&self) -> Result<Vec<Note>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM notes ORDER BY id ASC");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("notes", source))?;
        let mapped = statement
            .query_map([], map_note)
            .map_err(|source| StoreError::query("notes", source))?;
        collect(mapped)
    }

    /// The notes of one run, oldest first.
    pub fn notes_for_activity(&self, activity_id: i64) -> Result<Vec<Note>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM notes
             WHERE activity_id = ?1
             ORDER BY id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("notes of a run", source))?;
        let mapped = statement
            .query_map(params![activity_id], map_note)
            .map_err(|source| StoreError::query("notes of a run", source))?;
        collect(mapped)
    }

    /// The notes of one kind, oldest first.
    pub fn notes_of_kind(&self, kind: &str) -> Result<Vec<Note>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM notes WHERE kind = ?1 ORDER BY id ASC");
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("notes of a kind", source))?;
        let mapped = statement
            .query_map(params![kind], map_note)
            .map_err(|source| StoreError::query("notes of a kind", source))?;
        collect(mapped)
    }

    /// The weekly goal, in kilometres.
    ///
    /// The last one wins: the goal is a setting, and the newest setting is
    /// the one in force.
    pub fn goal_km(&self) -> Result<Option<f64>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT value FROM notes WHERE kind = ?1 AND value IS NOT NULL ORDER BY id DESC LIMIT 1",
            params![note_kind::GOAL],
            |row| row.get::<_, Option<f64>>(0),
        )
        .optional()
        .map_err(|source| StoreError::query("the goal", source))
        .map(|value| value.flatten())
    }

    /// The goal in force, as the row that holds it.
    pub fn goal_note(&self) -> Result<Option<Note>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM notes
             WHERE kind = ?1 AND value IS NOT NULL
             ORDER BY id DESC
             LIMIT 1"
        );
        conn.query_row(&sql, params![note_kind::GOAL], map_note)
            .optional()
            .map_err(|source| StoreError::query("the goal", source))
    }

    /// Set the weekly goal, or take it away.
    ///
    /// **A blank goal is no goal.** The goal is written as a setting, and a
    /// setting that is taken away leaves no row behind: a goal of zero would
    /// read as a failure, and it is not a goal at all.
    ///
    /// The write and the removal are in one transaction: there is never a
    /// moment with two goals.
    ///
    /// Returns the goal's id, or `None` when there is no goal.
    pub fn set_goal(
        &self,
        kilometres: Option<f64>,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<i64>> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        transaction
            .execute(
                "DELETE FROM notes WHERE kind = ?1",
                params![note_kind::GOAL],
            )
            .map_err(|source| StoreError::query("remove the goal", source))?;

        let id = match kilometres {
            None => None,
            Some(value) if !value.is_finite() || value < 0.0 => None,
            Some(value) => {
                let note = NewNote {
                    kind: note_kind::GOAL.to_string(),
                    activity_id: None,
                    text: None,
                    value: Some(value),
                    start_date: None,
                    end_date: None,
                    created_at,
                };
                Some(
                    transaction
                        .query_row(
                            "INSERT INTO notes (
                                kind, activity_id, text, value, start_date, end_date, created_at
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                             RETURNING id",
                            params![
                                note.kind,
                                note.activity_id,
                                note.text,
                                note.value,
                                note.start_date,
                                note.end_date,
                                DbDateTime(note.created_at),
                            ],
                            |row| row.get::<_, i64>(0),
                        )
                        .map_err(|source| StoreError::query("write the goal", source))?,
                )
            }
        };

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// The pause in force on a day.
    ///
    /// The pause whose days cover the day asked for. `None` when the plan
    /// is not paused on that day.
    pub fn pause_in_force(&self, date: &str) -> Result<Option<Note>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM notes
             WHERE kind = ?1
               AND start_date IS NOT NULL
               AND end_date IS NOT NULL
               AND start_date <= ?2
               AND end_date >= ?2
             ORDER BY id DESC
             LIMIT 1"
        );
        conn.query_row(&sql, params![note_kind::PAUSE, date], map_note)
            .optional()
            .map_err(|source| StoreError::query("the pause", source))
    }

    /// The pause in force, or the last one, when there is one.
    ///
    /// The settings show the pause that is on, and the last one when the
    /// plan is not paused now.
    pub fn pause(&self) -> Result<Option<Note>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM notes
             WHERE kind = ?1
             ORDER BY start_date DESC, id DESC
             LIMIT 1"
        );
        conn.query_row(&sql, params![note_kind::PAUSE], map_note)
            .optional()
            .map_err(|source| StoreError::query("the pause", source))
    }

    /// Set the pause, or take it away.
    ///
    /// The pause is one setting: the old one goes and the new one comes,
    /// in one transaction. `None` takes the pause away.
    ///
    /// Returns the pause's id, or `None` when there is no pause.
    pub fn set_pause(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        reason: Option<&str>,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<i64>> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        transaction
            .execute(
                "DELETE FROM notes WHERE kind = ?1",
                params![note_kind::PAUSE],
            )
            .map_err(|source| StoreError::query("remove the pause", source))?;

        let id = match (start_date, end_date) {
            (Some(start), Some(end)) => {
                let note = NewNote {
                    kind: note_kind::PAUSE.to_string(),
                    activity_id: None,
                    text: reason.map(|reason| reason.to_string()),
                    value: None,
                    start_date: Some(start.to_string()),
                    end_date: Some(end.to_string()),
                    created_at,
                };
                Some(
                    transaction
                        .query_row(
                            "INSERT INTO notes (
                                kind, activity_id, text, value, start_date, end_date, created_at
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                             RETURNING id",
                            params![
                                note.kind,
                                note.activity_id,
                                note.text,
                                note.value,
                                note.start_date,
                                note.end_date,
                                DbDateTime(note.created_at),
                            ],
                            |row| row.get::<_, i64>(0),
                        )
                        .map_err(|source| StoreError::query("write the pause", source))?,
                )
            }
            _ => None,
        };

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// Write the note on a run, or take it away.
    ///
    /// A run carries one note: the old one goes and the new one comes, in
    /// one transaction. `None` takes the note away.
    ///
    /// Returns the note's id, or `None` when the run has no note.
    pub fn set_note(
        &self,
        activity_id: i64,
        text: Option<&str>,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<i64>> {
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        transaction
            .execute(
                "DELETE FROM notes WHERE kind = ?1 AND activity_id IS ?2",
                params![note_kind::NOTE, activity_id],
            )
            .map_err(|source| StoreError::query("remove the note", source))?;

        let id = match text {
            None => None,
            Some(text) if text.trim().is_empty() => None,
            Some(text) => {
                let note = NewNote {
                    kind: note_kind::NOTE.to_string(),
                    activity_id: Some(activity_id),
                    text: Some(text.trim().to_string()),
                    value: None,
                    start_date: None,
                    end_date: None,
                    created_at,
                };
                Some(
                    transaction
                        .query_row(
                            "INSERT INTO notes (
                                kind, activity_id, text, value, start_date, end_date, created_at
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                             RETURNING id",
                            params![
                                note.kind,
                                note.activity_id,
                                note.text,
                                note.value,
                                note.start_date,
                                note.end_date,
                                DbDateTime(note.created_at),
                            ],
                            |row| row.get::<_, i64>(0),
                        )
                        .map_err(|source| StoreError::query("write the note", source))?,
                )
            }
        };

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(id)
    }

    /// The pauses, oldest first.
    pub fn pauses(&self) -> Result<Vec<Note>> {
        self.notes_of_kind(note_kind::PAUSE)
    }

    /// Delete one note.
    pub fn delete_note(&self, id: i64) -> Result<usize> {
        let conn = self.lock()?;
        let removed = conn
            .execute("DELETE FROM notes WHERE id = ?1", params![id])
            .map_err(|source| StoreError::query("delete the note", source))?;
        Ok(removed)
    }

    /// How many notes there are.
    pub fn count_notes(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM notes", [], |row| row.get(0))
            .map_err(|source| StoreError::query("count notes", source))
    }
}
