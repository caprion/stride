//! `heart_rate_declarations`.
//!
//! The athlete's own heart rates: the maximum and/or the resting rate, with
//! where each came from, the day it was measured and the day it was
//! entered.
//!
//! **One declaration is one or two rows.** A maximum and a resting rate
//! entered together are one declaration with two rows, and the rows name
//! the declaration they belong to. A later pair is a **new** declaration:
//! the earlier one is never edited and never deleted, and it stays in the
//! dated history with its own dates and its own sources.
//!
//! One declaration is in force at a time — the most recent one — and the
//! rows are written in one transaction, so a declaration is never half
//! written.
//!
//! The derived bands are **not stored**: they are recomputed on read from
//! the declaration in force, by the rule in the core.

use rusqlite::{params, OptionalExtension, Row, TransactionBehavior};

use crate::error::{Result, StoreError};
use crate::model::{HeartRateDeclarationRow, NewHeartRateDeclaration};
use crate::repos::collect;
use crate::store::Store;

/// The columns of `heart_rate_declarations`.
const COLUMNS: &str = "id, declaration_id, kind, value_bpm, source, measured_on, entered_on, \
    updated_on";

/// Read one row of `heart_rate_declarations`.
fn map(row: &Row<'_>) -> rusqlite::Result<HeartRateDeclarationRow> {
    Ok(HeartRateDeclarationRow {
        id: row.get("id")?,
        declaration_id: row.get("declaration_id")?,
        kind: row.get("kind")?,
        value_bpm: row.get("value_bpm")?,
        source: row.get("source")?,
        measured_on: row.get("measured_on")?,
        entered_on: row.get("entered_on")?,
        updated_on: row.get("updated_on")?,
    })
}

/// Whether a declaration carries anything at all.
///
/// A declaration with no heart rate is not a declaration: there is no value
/// for it to hold, and a row that holds nothing would be read as a
/// declaration of no value — which is not the same thing as no
/// declaration.
fn carries_a_value(declaration: &NewHeartRateDeclaration) -> bool {
    !declaration.rates.is_empty()
}

impl Store {
    /// Write a declaration.
    ///
    /// The rows of one declaration are written in **one transaction**, so a
    /// declaration is never half written: either the pair is there or
    /// neither of it is.
    ///
    /// A declaration that carries no heart rate at all is refused.
    ///
    /// Returns the declaration's id — the id of the first row written, which
    /// every row of the declaration names.
    pub fn write_heart_rate_declaration(
        &self,
        declaration: &NewHeartRateDeclaration,
    ) -> Result<i64> {
        if !carries_a_value(declaration) {
            return Err(StoreError::Invariant(
                "the heart rate declaration carries no value at all".to_string(),
            ));
        }
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::query("BEGIN", source))?;

        let mut declaration_id: Option<i64> = None;
        for rate in &declaration.rates {
            // The first row names the declaration; the rest join it. The
            // declaration's id is the id of its first row, so the two rows
            // of one declaration are never read as two declarations.
            let id = transaction
                .query_row(
                    "INSERT INTO heart_rate_declarations (
                        declaration_id, kind, value_bpm, source, measured_on, entered_on, updated_on
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     RETURNING id",
                    params![
                        declaration_id.unwrap_or_default(),
                        rate.kind,
                        rate.value_bpm,
                        rate.source,
                        rate.measured_on,
                        declaration.entered_on,
                        declaration.updated_on,
                    ],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|source| StoreError::query("write a heart rate declaration", source))?;
            if declaration_id.is_none() {
                declaration_id = Some(id);
                transaction
                    .execute(
                        "UPDATE heart_rate_declarations SET declaration_id = ?2 WHERE id = ?1",
                        params![id, id],
                    )
                    .map_err(|source| {
                        StoreError::query("write a heart rate declaration", source)
                    })?;
            }
        }

        transaction
            .commit()
            .map_err(|source| StoreError::query("COMMIT", source))?;
        Ok(declaration_id.unwrap_or_default())
    }

    /// Every declared heart rate, newest declaration first.
    ///
    /// The rows of one declaration sit together: they share the day the
    /// declaration was made and the id it was given.
    pub fn heart_rate_declarations(&self) -> Result<Vec<HeartRateDeclarationRow>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {COLUMNS} FROM heart_rate_declarations
             ORDER BY entered_on DESC, declaration_id DESC, id ASC"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|source| StoreError::query("the heart rate declarations", source))?;
        let mapped = statement
            .query_map([], map)
            .map_err(|source| StoreError::query("the heart rate declarations", source))?;
        collect(mapped)
    }

    /// One declared heart rate, by the id of its row.
    pub fn heart_rate_declaration_row(&self, id: i64) -> Result<Option<HeartRateDeclarationRow>> {
        let conn = self.lock()?;
        let sql = format!("SELECT {COLUMNS} FROM heart_rate_declarations WHERE id = ?1");
        conn.query_row(&sql, params![id], map)
            .optional()
            .map_err(|source| StoreError::query("the heart rate declaration", source))
    }

    /// How many declared heart rates there are.
    pub fn count_heart_rate_declarations(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM heart_rate_declarations", [], |row| {
            row.get(0)
        })
        .map_err(|source| StoreError::query("count the heart rate declarations", source))
    }
}
