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
