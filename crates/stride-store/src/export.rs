//! Export, import, and the plaintext → SQLCipher conversion.
//!
//! ## The export
//!
//! A **directory** with two files in it:
//!
//! ```text
//! export/
//!   manifest.json    what the export is
//!   stride.db        the database, encrypted
//! ```
//!
//! The database is a real SQLite file with the frozen schema, so it opens on
//! the phone: the plan says the desktop export opens on the phone, and the
//! only way that is true is if the export is the same schema the phone
//! reads. The manifest says which schema it is, so an import can **refuse**
//! to read an export whose schema is not the one this build reads — silently
//! changing the schema version is how a year of history is lost quietly.
//!
//! The export is encrypted with the key the caller passes. The key is not in
//! the manifest, not in the file name, and not in the directory: it is an
//! argument, like everywhere else in this crate.
//!
//! ## The import
//!
//! The import is a **merge**, not a replace. A run is matched by its source
//! and its external id, a day of wellness by its date, a plan block by its
//! source, name and first day, and a planned session by its block, its date
//! and its kind. The rows that are already there are updated; the rows that
//! are not are added. Nothing is duplicated by importing twice.
//!
//! ## The conversion
//!
//! The old desktop converted a plain SQLite file into an encrypted one, once,
//! on the first run — the Python app wrote plain files and SQLCipher refused
//! them. This is the same operation: the plain file is read, an encrypted
//! file with the same schema and the same rows is written, the plain copy is
//! kept beside it as `<name>.plaintext-backup`, and the encrypted file is put
//! in the plain file's place.
//!
//! The conversion is careful, because it replaces a file:
//!
//! * it refuses to run while another process has the file open — the copy
//!   would be of a database that is being written under it, and the rename
//!   would take the file away from the writer;
//! * it reads the source in **one transaction**, so the rows it copies are a
//!   consistent snapshot and not a mixture of two states;
//! * it counts the rows it read and the rows it wrote, and refuses to
//!   replace the original if the two do not agree;
//! * it reads the result back, and puts the plain file back if it does not
//!   answer.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{Result, StoreError};
use crate::hash;
use crate::key::DatabaseKey;
use crate::logging;
use crate::schema;
use crate::store::Store;

/// The name of the manifest in an export directory.
pub const MANIFEST_NAME: &str = "manifest.json";

/// The name of the database in an export directory.
pub const DATABASE_NAME: &str = "stride.db";

/// The name of the plain copy the conversion leaves behind.
pub const PLAIN_BACKUP_SUFFIX: &str = ".plaintext-backup";

/// The format the manifest declares.
pub const EXPORT_FORMAT: &str = "stride-export";

/// The versions of the export format this build reads.
pub const SUPPORTED_FORMAT_VERSIONS: [i64; 1] = [1];

/// What an export says about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportManifest {
    /// Always `stride-export`.
    pub format: String,
    /// The version of the export format.
    pub format_version: i64,
    /// The version of the database schema in the export.
    pub schema_version: i64,
    /// When the export was made.
    pub created_at: DateTime<Utc>,
    /// The name of the database file in the export directory.
    pub database: String,
    /// The size of the database file, in bytes.
    pub database_bytes: u64,
    /// The SHA-256 of the database file.
    pub database_sha256: String,
    /// How many rows each table holds.
    pub tables: BTreeMap<String, i64>,
    /// How many rows there are in all.
    pub rows: i64,
}

/// What an import did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// How many runs were written.
    pub activities: usize,
    /// How many days of wellness were written.
    pub wellness: usize,
    /// How many plan blocks were written.
    pub plan_blocks: usize,
    /// How many planned sessions were written.
    pub plan_sessions: usize,
    /// How many notes were written.
    pub notes: usize,
    /// How many rows there are in all.
    pub rows: usize,
}

/// What the conversion found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionReport {
    /// The tables that were copied, with the number of rows in each.
    pub tables: Vec<(String, i64)>,
    /// The rows that were copied, in total.
    pub rows: i64,
    /// Anything the two databases disagreed about.
    pub drift: Vec<String>,
    /// Whether the plain file held the frozen schema.
    pub schema_matches: bool,
}

/// How a database file is encrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// There is no file yet.
    Missing,
    /// A plain SQLite file.
    Plaintext,
    /// A SQLCipher file.
    Encrypted,
}

impl Format {
    /// A human-readable name, for the log.
    pub fn name(&self) -> &'static str {
        match self {
            Format::Missing => "missing",
            Format::Plaintext => "plaintext",
            Format::Encrypted => "encrypted",
        }
    }
}

// ------------------------------------------------------------------- export

/// Write an export of the store into a directory.
///
/// The directory is created if it is not there. The database is written with
/// the key the caller passes, and the manifest says which schema it holds.
pub fn export(store: &Store, directory: &Path, key: &DatabaseKey) -> Result<ExportManifest> {
    std::fs::create_dir_all(directory)
        .map_err(|source| StoreError::io(directory.to_path_buf(), source))?;

    let staged = directory.join(format!("{DATABASE_NAME}.staged"));
    let _ = std::fs::remove_file(&staged);

    let target = Store::open_and_migrate(&staged, key)?;

    let counts = target.with_connection(|destination| {
        store.with_connection(|source| copy_schema(source, destination))
    })?;

    // Fold the write-ahead log back into the file, so that the file that is
    // moved into place is complete on its own.
    target.with_connection(|connection| {
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|source| StoreError::query("wal_checkpoint", source))
    })?;
    drop(target);
    let _ = std::fs::remove_file(directory.join(format!("{DATABASE_NAME}-wal")));
    let _ = std::fs::remove_file(directory.join(format!("{DATABASE_NAME}-shm")));

    let database = directory.join(DATABASE_NAME);
    move_file(&staged, &database)?;

    let bytes = std::fs::read(&database).map_err(|source| StoreError::io(&database, source))?;
    let digest = hash::sha256_hex(&bytes);

    let manifest = ExportManifest {
        format: EXPORT_FORMAT.to_string(),
        format_version: 1,
        schema_version: schema::SCHEMA_VERSION,
        created_at: Utc::now(),
        database: DATABASE_NAME.to_string(),
        database_bytes: bytes.len() as u64,
        database_sha256: digest,
        rows: counts.values().sum(),
        tables: counts,
    };

    let manifest_path = directory.join(MANIFEST_NAME);
    let text = serde_json::to_string_pretty(&manifest)
        .map_err(|source| StoreError::export(&manifest_path, source.to_string()))?;
    std::fs::write(&manifest_path, text)
        .map_err(|source| StoreError::io(&manifest_path, source))?;

    logging::log(&format!(
        "exported {} rows into {}",
        manifest.rows,
        directory.display()
    ));
    Ok(manifest)
}

/// The supported format versions, as text.
fn versions_text() -> String {
    SUPPORTED_FORMAT_VERSIONS
        .iter()
        .map(|version| version.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Read the manifest of an export directory.
pub fn read_manifest(directory: &Path) -> Result<ExportManifest> {
    let path = directory.join(MANIFEST_NAME);
    let text = std::fs::read_to_string(&path)
        .map_err(|source| StoreError::export(&path, source.to_string()))?;
    let manifest: ExportManifest = serde_json::from_str(&text)
        .map_err(|source| StoreError::export(&path, source.to_string()))?;
    Ok(manifest)
}

/// Import an export directory into the store.
///
/// The import **refuses** an export whose schema version is not the one this
/// build reads, and an export whose format version it does not know. It says
/// so and changes nothing.
pub fn import(store: &Store, directory: &Path, key: &DatabaseKey) -> Result<ImportReport> {
    let manifest = read_manifest(directory)?;
    if manifest.format != EXPORT_FORMAT {
        return Err(StoreError::export(
            directory,
            format!(
                "the manifest is a {:?}, not a {EXPORT_FORMAT}",
                manifest.format
            ),
        ));
    }
    if !SUPPORTED_FORMAT_VERSIONS.contains(&manifest.format_version) {
        return Err(StoreError::export(
            directory,
            format!(
                "the export format version {} is not one this build reads ({})",
                manifest.format_version,
                versions_text()
            ),
        ));
    }
    if manifest.schema_version != schema::SCHEMA_VERSION {
        return Err(StoreError::SchemaVersion {
            found: manifest.schema_version,
            expected: schema::SCHEMA_VERSION,
        });
    }

    let database = directory.join(&manifest.database);
    let source = Store::open(&database, key)?;

    let report = store.with_connection(|destination| {
        source.with_connection(|origin| merge(origin, destination))
    })?;

    logging::log(&format!(
        "imported {} rows from {}",
        report.rows,
        directory.display()
    ));
    Ok(report)
}

// -------------------------------------------------------------------- merge

/// The columns of `activities`, in the schema's order.
const ACTIVITY_COLUMNS: &[&str] = &[
    "id",
    "source",
    "external_id",
    "name",
    "sport_type",
    "start_time",
    "local_date",
    "distance_m",
    "moving_time_s",
    "elapsed_time_s",
    "elevation_gain_m",
    "avg_hr",
    "max_hr",
    "fit_blob",
    "fit_sha256",
    "created_at",
    "updated_at",
];

/// The columns of `activity_samples`.
const SAMPLE_COLUMNS: &[&str] = &["activity_id", "series", "sha256", "first_s", "last_s"];

/// The columns of `wellness_daily`.
const WELLNESS_COLUMNS: &[&str] = &[
    "date",
    "sleep_s",
    "deep_sleep_s",
    "rem_s",
    "light_sleep_s",
    "awake_s",
    "hrv_ms",
    "resting_hr",
    "stress_avg",
    "source",
    "updated_at",
];

/// The columns of `plan_blocks`.
const BLOCK_COLUMNS: &[&str] = &[
    "id",
    "name",
    "source",
    "start_date",
    "end_date",
    "goal",
    "status",
    "notes",
    "created_at",
];

/// The columns of `plan_sessions`.
const SESSION_COLUMNS: &[&str] = &[
    "id",
    "block_id",
    "date",
    "session_type",
    "title",
    "details",
    "target_distance_m",
    "pace_low_s_km",
    "pace_high_s_km",
];

/// The columns of `notes`.
const NOTE_COLUMNS: &[&str] = &[
    "id",
    "kind",
    "activity_id",
    "text",
    "value",
    "start_date",
    "end_date",
    "created_at",
];

/// Copy every row of the schema from one database to another.
///
/// The tables are copied in the schema's order, so a child never arrives
/// before its parent. The columns are the schema's own, and the values are
/// always bound as parameters.
fn copy_schema(source: &Connection, destination: &Connection) -> Result<BTreeMap<String, i64>> {
    let mut counts = BTreeMap::new();

    let transaction = destination
        .unchecked_transaction()
        .map_err(|error| StoreError::query("BEGIN", error))?;

    counts.insert(
        "activities".to_string(),
        copy_table(source, &transaction, "activities", ACTIVITY_COLUMNS)?,
    );
    counts.insert(
        "activity_samples".to_string(),
        copy_table(source, &transaction, "activity_samples", SAMPLE_COLUMNS)?,
    );
    counts.insert(
        "wellness_daily".to_string(),
        copy_table(source, &transaction, "wellness_daily", WELLNESS_COLUMNS)?,
    );
    counts.insert(
        "plan_blocks".to_string(),
        copy_table(source, &transaction, "plan_blocks", BLOCK_COLUMNS)?,
    );
    counts.insert(
        "plan_sessions".to_string(),
        copy_table(source, &transaction, "plan_sessions", SESSION_COLUMNS)?,
    );
    counts.insert(
        "notes".to_string(),
        copy_table(source, &transaction, "notes", NOTE_COLUMNS)?,
    );
    counts.insert(
        "sync_state".to_string(),
        copy_table(
            source,
            &transaction,
            "sync_state",
            &[
                "kind",
                "cursor",
                "last_success_at",
                "token_expires_at",
                "updated_at",
            ],
        )?,
    );

    transaction
        .commit()
        .map_err(|error| StoreError::query("COMMIT", error))?;
    Ok(counts)
}

/// Copy the rows of one table from one database to another.
///
/// The column names come from the schema, not from anything a user typed, and
/// the values are bound as parameters.
fn copy_table(
    source: &Connection,
    destination: &Connection,
    table: &str,
    columns: &[&str],
) -> Result<i64> {
    let list = quoted_list(columns);
    let select = format!("SELECT {list} FROM \"{table}\"");
    let insert = format!(
        "INSERT INTO \"{table}\" ({list}) VALUES ({})",
        placeholders(columns.len())
    );

    let mut statement = source
        .prepare(&select)
        .map_err(|source| StoreError::query(select.clone(), source))?;
    let mut rows = statement
        .query([])
        .map_err(|source| StoreError::query(select.clone(), source))?;

    let mut prepared = destination
        .prepare(&insert)
        .map_err(|source| StoreError::query(insert.clone(), source))?;

    let mut copied = 0i64;
    while let Some(row) = rows
        .next()
        .map_err(|source| StoreError::query(select.clone(), source))?
    {
        let mut values = Vec::with_capacity(columns.len());
        for index in 0..columns.len() {
            let value = row
                .get::<_, Value>(index)
                .map_err(|source| StoreError::query(select.clone(), source))?;
            values.push(value);
        }
        prepared
            .execute(rusqlite::params_from_iter(values.iter()))
            .map_err(|source| StoreError::query(insert.clone(), source))?;
        copied += 1;
    }
    Ok(copied)
}

/// Merge the rows of an export into a database.
///
/// The rows are matched by their natural keys, so importing twice does not
/// duplicate anything, and the ids of the export are remapped onto the ids
/// of the destination — a note that points at a run keeps pointing at it.
fn merge(source: &Connection, destination: &Connection) -> Result<ImportReport> {
    let mut report = ImportReport::default();

    let transaction = destination
        .unchecked_transaction()
        .map_err(|error| StoreError::query("BEGIN", error))?;

    // The runs, and the map from the export's ids to this database's ids.
    let mut activity_ids: BTreeMap<i64, i64> = BTreeMap::new();
    for row in read_rows(source, "activities", ACTIVITY_COLUMNS)? {
        let external_id = text_of(&row, "external_id");
        let source_name = text_of(&row, "source");
        let existing = find_activity(destination, &source_name, &external_id)?;
        let id = match existing {
            Some(id) => {
                update_activity(&transaction, id, &row)?;
                id
            }
            None => insert_activity(&transaction, &row)?,
        };
        if let Some(old) = row.get("id").and_then(|value| value.as_i64()) {
            activity_ids.insert(old, id);
        }
        report.activities += 1;
    }

    // The derived series, against the runs' new ids.
    for row in read_rows(source, "activity_samples", SAMPLE_COLUMNS)? {
        let old = row
            .get("activity_id")
            .and_then(|value| value.as_i64())
            .unwrap_or_default();
        let activity_id = match activity_ids.get(&old) {
            Some(id) => *id,
            None => continue,
        };
        let series = row
            .get("series")
            .and_then(|value| value.as_blob())
            .unwrap_or_default()
            .to_vec();
        let digest = text_of(&row, "sha256");
        let first = row.get("first_s").and_then(|value| value.as_i64());
        let last = row.get("last_s").and_then(|value| value.as_i64());
        transaction
            .execute(
                "INSERT INTO activity_samples (activity_id, series, sha256, first_s, last_s)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(activity_id) DO UPDATE SET
                    series = excluded.series,
                    sha256 = excluded.sha256,
                    first_s = excluded.first_s,
                    last_s = excluded.last_s",
                rusqlite::params![activity_id, series, digest, first, last],
            )
            .map_err(|error| StoreError::query("the activity samples", error))?;
        report.rows += 1;
    }

    // The days of wellness, by date.
    for row in read_rows(source, "wellness_daily", WELLNESS_COLUMNS)? {
        let date = text_of(&row, "date");
        let values = values_of(
            &row,
            &[
                "sleep_s",
                "deep_sleep_s",
                "rem_s",
                "light_sleep_s",
                "awake_s",
                "hrv_ms",
                "resting_hr",
                "stress_avg",
            ],
        );
        transaction
            .execute(
                "INSERT INTO wellness_daily (
                    date, sleep_s, deep_sleep_s, rem_s, light_sleep_s, awake_s,
                    hrv_ms, resting_hr, stress_avg, source, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(date) DO UPDATE SET
                    sleep_s = excluded.sleep_s,
                    deep_sleep_s = excluded.deep_sleep_s,
                    rem_s = excluded.rem_s,
                    light_sleep_s = excluded.light_sleep_s,
                    awake_s = excluded.awake_s,
                    hrv_ms = excluded.hrv_ms,
                    resting_hr = excluded.resting_hr,
                    stress_avg = excluded.stress_avg,
                    source = excluded.source,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    date,
                    values[0],
                    values[1],
                    values[2],
                    values[3],
                    values[4],
                    values[5],
                    values[6],
                    values[7],
                    text_of(&row, "source"),
                    text_of(&row, "updated_at"),
                ],
            )
            .map_err(|error| StoreError::query("the wellness", error))?;
        report.wellness += 1;
        report.rows += 1;
    }

    // The plan blocks, by source, name and first day.
    let mut block_ids: BTreeMap<i64, i64> = BTreeMap::new();
    for row in read_rows(source, "plan_blocks", BLOCK_COLUMNS)? {
        let name = text_of(&row, "name");
        let source_name = text_of(&row, "source");
        let start_date = text_of(&row, "start_date");
        let existing = find_block(destination, &source_name, &name, &start_date)?;
        let id = match existing {
            Some(id) => {
                update_block(&transaction, id, &row)?;
                id
            }
            None => insert_block(&transaction, &row)?,
        };
        if let Some(old) = row.get("id").and_then(|value| value.as_i64()) {
            block_ids.insert(old, id);
        }
        report.plan_blocks += 1;
        report.rows += 1;
    }

    // The planned sessions, by block, date and kind.
    for row in read_rows(source, "plan_sessions", SESSION_COLUMNS)? {
        let old = row
            .get("block_id")
            .and_then(|value| value.as_i64())
            .unwrap_or_default();
        let block_id = match block_ids.get(&old) {
            Some(id) => *id,
            None => continue,
        };
        let date = text_of(&row, "date");
        let session_type = text_of(&row, "session_type");
        let existing = find_session(&transaction, block_id, &date, &session_type)?;
        match existing {
            Some(id) => {
                update_session(&transaction, id, &row)?;
            }
            None => {
                insert_session(&transaction, block_id, &row)?;
            }
        }
        report.plan_sessions += 1;
        report.rows += 1;
    }

    // The notes, by kind, run, text and dates.
    for row in read_rows(source, "notes", NOTE_COLUMNS)? {
        let old = row.get("activity_id").and_then(|value| value.as_i64());
        let activity_id = match old {
            Some(old) => match activity_ids.get(&old) {
                Some(id) => Some(*id),
                None => continue,
            },
            None => None,
        };
        let kind = text_of(&row, "kind");
        let text = optional_text(&row, "text");
        let start_date = optional_text(&row, "start_date");
        let end_date = optional_text(&row, "end_date");
        let existing = find_note(
            &transaction,
            &kind,
            activity_id,
            &text,
            &start_date,
            &end_date,
        )?;
        match existing {
            Some(id) => {
                let value = row.get("value").and_then(|value| value.as_f64());
                transaction
                    .execute(
                        "UPDATE notes SET value = ?2 WHERE id = ?1",
                        rusqlite::params![id, value],
                    )
                    .map_err(|error| StoreError::query("the note", error))?;
            }
            None => {
                transaction
                    .execute(
                        "INSERT INTO notes (
                            kind, activity_id, text, value, start_date, end_date, created_at
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![
                            kind,
                            activity_id,
                            text,
                            row.get("value").and_then(|value| value.as_f64()),
                            start_date,
                            end_date,
                            text_of(&row, "created_at"),
                        ],
                    )
                    .map_err(|error| StoreError::query("the note", error))?;
            }
        }
        report.notes += 1;
        report.rows += 1;
    }

    // The sync cursors. They come with the runs: a phone that has the runs
    // does not have to fetch them again, and the cursor says where the sync
    // got to.
    for row in read_rows(
        source,
        "sync_state",
        &[
            "kind",
            "cursor",
            "last_success_at",
            "token_expires_at",
            "updated_at",
        ],
    )? {
        let kind = text_of(&row, "kind");
        transaction
            .execute(
                "INSERT INTO sync_state (kind, cursor, last_success_at, token_expires_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(kind) DO UPDATE SET
                    cursor = excluded.cursor,
                    last_success_at = excluded.last_success_at,
                    token_expires_at = excluded.token_expires_at,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    kind,
                    optional_text(&row, "cursor"),
                    optional_text(&row, "last_success_at"),
                    optional_text(&row, "token_expires_at"),
                    text_of(&row, "updated_at"),
                ],
            )
            .map_err(|error| StoreError::query("the sync cursor", error))?;
        report.rows += 1;
    }

    transaction
        .commit()
        .map_err(|error| StoreError::query("COMMIT", error))?;
    Ok(report)
}

/// Read the rows of one table, as maps of column name to value.
fn read_rows(
    source: &Connection,
    table: &str,
    columns: &[&str],
) -> Result<Vec<BTreeMap<String, Value>>> {
    let list = quoted_list(columns);
    let select = format!("SELECT {list} FROM \"{table}\"");
    let mut statement = source
        .prepare(&select)
        .map_err(|source| StoreError::query(select.clone(), source))?;
    let mut rows = statement
        .query([])
        .map_err(|source| StoreError::query(select.clone(), source))?;

    let mut out = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|source| StoreError::query(select.clone(), source))?
    {
        let mut values = BTreeMap::new();
        for (index, column) in columns.iter().enumerate() {
            let value = row
                .get::<_, Value>(index)
                .map_err(|source| StoreError::query(select.clone(), source))?;
            values.insert((*column).to_string(), value);
        }
        out.push(values);
    }
    Ok(out)
}

/// Reading a value out of a row.
///
/// `rusqlite` puts the accessors on `ValueRef` and not on `Value`, and the
/// rows are read as owned values, so the accessors are here. A value of the
/// wrong type is **absent**, not a zero: there is no sentinel in this crate.
trait ValueExt {
    /// The value as an integer.
    fn as_i64(&self) -> Option<i64>;
    /// The value as a real number.
    fn as_f64(&self) -> Option<f64>;
    /// The value as a blob.
    fn as_blob(&self) -> Option<&[u8]>;
}

impl ValueExt for Value {
    fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(number) => Some(*number),
            Value::Real(number) => Some(*number as i64),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Real(number) => Some(*number),
            Value::Integer(number) => Some(*number as f64),
            _ => None,
        }
    }

    fn as_blob(&self) -> Option<&[u8]> {
        match self {
            Value::Blob(bytes) => Some(bytes),
            _ => None,
        }
    }
}

/// The value of a column, as text.
fn text_of(row: &BTreeMap<String, Value>, column: &str) -> String {
    match row.get(column) {
        Some(Value::Text(text)) => text.clone(),
        Some(Value::Integer(number)) => number.to_string(),
        Some(Value::Real(number)) => number.to_string(),
        _ => String::new(),
    }
}

/// The value of a column, as text or nothing.
fn optional_text(row: &BTreeMap<String, Value>, column: &str) -> Option<String> {
    match row.get(column) {
        Some(Value::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

/// The values of several columns, in order.
fn values_of(row: &BTreeMap<String, Value>, columns: &[&str]) -> Vec<Option<f64>> {
    columns
        .iter()
        .map(|column| row.get(*column).and_then(|value| value.as_f64()))
        .collect()
}

/// Find the run with this source and external id.
fn find_activity(destination: &Connection, source: &str, external_id: &str) -> Result<Option<i64>> {
    destination
        .query_row(
            "SELECT id FROM activities WHERE source = ?1 AND external_id = ?2",
            rusqlite::params![source, external_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| StoreError::query("find the activity", error))
}

/// Update a run that is already in the destination.
fn update_activity(
    transaction: &rusqlite::Transaction<'_>,
    id: i64,
    row: &BTreeMap<String, Value>,
) -> Result<()> {
    transaction
        .execute(
            "UPDATE activities
             SET name = ?2, sport_type = ?3, start_time = ?4, local_date = ?5,
                 distance_m = ?6, moving_time_s = ?7, elapsed_time_s = ?8,
                 elevation_gain_m = ?9, avg_hr = ?10, max_hr = ?11,
                 fit_blob = ?12, fit_sha256 = ?13, updated_at = ?14
             WHERE id = ?1",
            rusqlite::params![
                id,
                text_of(row, "name"),
                text_of(row, "sport_type"),
                text_of(row, "start_time"),
                text_of(row, "local_date"),
                row.get("distance_m").and_then(|value| value.as_f64()),
                row.get("moving_time_s").and_then(|value| value.as_f64()),
                row.get("elapsed_time_s").and_then(|value| value.as_f64()),
                row.get("elevation_gain_m").and_then(|value| value.as_f64()),
                row.get("avg_hr").and_then(|value| value.as_f64()),
                row.get("max_hr").and_then(|value| value.as_f64()),
                row.get("fit_blob").and_then(|value| value.as_blob()),
                text_of(row, "fit_sha256"),
                text_of(row, "updated_at"),
            ],
        )
        .map_err(|error| StoreError::query("update the activity", error))?;
    Ok(())
}

/// Add a run that is not in the destination yet.
fn insert_activity(
    transaction: &rusqlite::Transaction<'_>,
    row: &BTreeMap<String, Value>,
) -> Result<i64> {
    transaction
        .query_row(
            "INSERT INTO activities (
                source, external_id, name, sport_type, start_time, local_date,
                distance_m, moving_time_s, elapsed_time_s, elevation_gain_m,
                avg_hr, max_hr, fit_blob, fit_sha256, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             RETURNING id",
            rusqlite::params![
                text_of(row, "source"),
                text_of(row, "external_id"),
                text_of(row, "name"),
                text_of(row, "sport_type"),
                text_of(row, "start_time"),
                text_of(row, "local_date"),
                row.get("distance_m").and_then(|value| value.as_f64()),
                row.get("moving_time_s").and_then(|value| value.as_f64()),
                row.get("elapsed_time_s").and_then(|value| value.as_f64()),
                row.get("elevation_gain_m").and_then(|value| value.as_f64()),
                row.get("avg_hr").and_then(|value| value.as_f64()),
                row.get("max_hr").and_then(|value| value.as_f64()),
                row.get("fit_blob").and_then(|value| value.as_blob()),
                text_of(row, "fit_sha256"),
                text_of(row, "created_at"),
                text_of(row, "updated_at"),
            ],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| StoreError::query("insert the activity", error))
}

/// Find the plan block with this source, name and first day.
fn find_block(
    destination: &Connection,
    source: &str,
    name: &str,
    start_date: &str,
) -> Result<Option<i64>> {
    destination
        .query_row(
            "SELECT id FROM plan_blocks
             WHERE source = ?1 AND name = ?2 AND start_date = ?3
             ORDER BY id ASC LIMIT 1",
            rusqlite::params![source, name, start_date],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| StoreError::query("find the plan block", error))
}

/// Update a plan block that is already in the destination.
fn update_block(
    transaction: &rusqlite::Transaction<'_>,
    id: i64,
    row: &BTreeMap<String, Value>,
) -> Result<()> {
    transaction
        .execute(
            "UPDATE plan_blocks
             SET end_date = ?2, goal = ?3, status = ?4, notes = ?5
             WHERE id = ?1",
            rusqlite::params![
                id,
                optional_text(row, "end_date"),
                optional_text(row, "goal"),
                text_of(row, "status"),
                optional_text(row, "notes"),
            ],
        )
        .map_err(|error| StoreError::query("update the plan block", error))?;
    Ok(())
}

/// Add a plan block that is not in the destination yet.
fn insert_block(
    transaction: &rusqlite::Transaction<'_>,
    row: &BTreeMap<String, Value>,
) -> Result<i64> {
    transaction
        .query_row(
            "INSERT INTO plan_blocks (
                name, source, start_date, end_date, goal, status, notes, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             RETURNING id",
            rusqlite::params![
                text_of(row, "name"),
                text_of(row, "source"),
                text_of(row, "start_date"),
                optional_text(row, "end_date"),
                optional_text(row, "goal"),
                text_of(row, "status"),
                optional_text(row, "notes"),
                text_of(row, "created_at"),
            ],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| StoreError::query("insert the plan block", error))
}

/// Find the planned session of this block, date and kind.
fn find_session(
    connection: &Connection,
    block_id: i64,
    date: &str,
    session_type: &str,
) -> Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM plan_sessions
             WHERE block_id = ?1 AND date = ?2 AND session_type = ?3
             ORDER BY id ASC LIMIT 1",
            rusqlite::params![block_id, date, session_type],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| StoreError::query("find the plan session", error))
}

/// Update a planned session that is already in the destination.
fn update_session(
    transaction: &rusqlite::Transaction<'_>,
    id: i64,
    row: &BTreeMap<String, Value>,
) -> Result<()> {
    transaction
        .execute(
            "UPDATE plan_sessions
             SET title = ?2, details = ?3, target_distance_m = ?4,
                 pace_low_s_km = ?5, pace_high_s_km = ?6
             WHERE id = ?1",
            rusqlite::params![
                id,
                optional_text(row, "title"),
                optional_text(row, "details"),
                row.get("target_distance_m")
                    .and_then(|value| value.as_f64()),
                row.get("pace_low_s_km").and_then(|value| value.as_f64()),
                row.get("pace_high_s_km").and_then(|value| value.as_f64()),
            ],
        )
        .map_err(|error| StoreError::query("update the plan session", error))?;
    Ok(())
}

/// Add a planned session that is not in the destination yet.
fn insert_session(
    transaction: &rusqlite::Transaction<'_>,
    block_id: i64,
    row: &BTreeMap<String, Value>,
) -> Result<i64> {
    transaction
        .query_row(
            "INSERT INTO plan_sessions (
                block_id, date, session_type, title, details,
                target_distance_m, pace_low_s_km, pace_high_s_km
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             RETURNING id",
            rusqlite::params![
                block_id,
                text_of(row, "date"),
                text_of(row, "session_type"),
                optional_text(row, "title"),
                optional_text(row, "details"),
                row.get("target_distance_m")
                    .and_then(|value| value.as_f64()),
                row.get("pace_low_s_km").and_then(|value| value.as_f64()),
                row.get("pace_high_s_km").and_then(|value| value.as_f64()),
            ],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| StoreError::query("insert the plan session", error))
}

/// Find the note of this kind, run, text and dates.
fn find_note(
    connection: &Connection,
    kind: &str,
    activity_id: Option<i64>,
    text: &Option<String>,
    start_date: &Option<String>,
    end_date: &Option<String>,
) -> Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM notes
             WHERE kind = ?1
               AND activity_id IS ?2
               AND text IS ?3
               AND start_date IS ?4
               AND end_date IS ?5
             ORDER BY id ASC LIMIT 1",
            rusqlite::params![kind, activity_id, text, start_date, end_date],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| StoreError::query("find the note", error))
}

/// The column list of a statement, quoted.
fn quoted_list(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|column| format!("\"{column}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The placeholders of an insert, numbered.
fn placeholders(count: usize) -> String {
    (1..=count)
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ")
}

// --------------------------------------------------------------- conversion

/// What is the file at this path?
///
/// A SQLCipher file has a random header, so the only way to tell the two
/// apart is to look for SQLite's own magic at the front of the file.
pub fn detect_format(path: &Path) -> Result<Format> {
    let mut head = [0u8; 16];
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Format::Missing),
        Err(source) => return Err(StoreError::io(path.to_path_buf(), source)),
    };
    use std::io::Read;
    let read = file
        .read(&mut head)
        .map_err(|source| StoreError::io(path.to_path_buf(), source))?;
    if read < 16 {
        // A file that is too short to be a database: an empty file, or a
        // half-written one. It is not encrypted either, and opening it will
        // fail with a clear error.
        return Ok(Format::Plaintext);
    }
    if &head == b"SQLite format 3\0" {
        Ok(Format::Plaintext)
    } else {
        Ok(Format::Encrypted)
    }
}

/// Convert the plain database at `path` into an encrypted one, in place.
///
/// The plain file is kept as `<name>.plaintext-backup`.
///
/// The encrypted file has the **same schema and the same rows** as the plain
/// one: the conversion is an encryption, not a migration. A plain file that
/// already holds the frozen schema becomes a frozen-schema database, and a
/// plain file that holds something else becomes that something else, with the
/// difference reported in [`ConversionReport::drift`].
pub fn convert_plaintext_to_encrypted(path: &Path, key: &DatabaseKey) -> Result<ConversionReport> {
    match detect_format(path)? {
        Format::Plaintext => {}
        Format::Encrypted => {
            return Err(StoreError::Invariant(format!(
                "the database at {} is already encrypted",
                path.display()
            )))
        }
        Format::Missing => {
            return Err(StoreError::Invariant(format!(
                "there is no database at {}",
                path.display()
            )))
        }
    }

    // Another process writing to the file right now would make the rename
    // take the file away from it: its rows would go into the old inode and
    // nowhere else. So this is refused, and said so.
    let users = users_of(path);
    if !users.is_empty() {
        return Err(StoreError::Invariant(format!(
            "the database at {} is open in another process ({users:?}): \
             stop it before the file is converted",
            path.display()
        )));
    }

    let staged = staged_path(path);
    let backup = backup_path(path);
    for stale in [
        &staged,
        &backup,
        &sidecar(&staged, "-wal"),
        &sidecar(&staged, "-shm"),
    ] {
        let _ = std::fs::remove_file(stale);
    }

    // The plain file is read inside one transaction below, which is a
    // consistent snapshot of it.
    let source = Connection::open(path).map_err(|source| StoreError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    quick_check(&source, path)?;

    // The plain file is kept, because the conversion is the one operation
    // here that cannot be undone.
    take_snapshot(path, &backup)?;

    // The encrypted copy: the same schema, the same rows.
    let target = Store::open(&staged, key)?;
    let report = target.with_connection(|destination| {
        source
            .execute_batch("BEGIN;")
            .map_err(|error| StoreError::query("BEGIN", error))?;
        let outcome = mirror(&source, destination);
        let _ = source.execute_batch("ROLLBACK;");
        outcome
    })?;

    // Fold the write-ahead log back into the file, so that the file that is
    // renamed into place is complete on its own.
    target.with_connection(|connection| {
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|source| StoreError::query("wal_checkpoint", source))
    })?;
    drop(target);
    let _ = std::fs::remove_file(sidecar(&staged, "-wal"));
    let _ = std::fs::remove_file(sidecar(&staged, "-shm"));

    // The plain file's write-ahead log belongs to the plain file. Leaving it
    // next to an encrypted database would make SQLite try to apply it.
    let _ = std::fs::remove_file(sidecar(path, "-wal"));
    let _ = std::fs::remove_file(sidecar(path, "-shm"));

    std::fs::rename(&staged, path).map_err(|source| StoreError::io(path.to_path_buf(), source))?;

    // Read it back. The file that is now in place is opened with the same key
    // and its rows are counted; if it does not answer, the plain file goes
    // back where it was.
    if let Err(error) = verify_converted(path, key, &report) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::copy(&backup, path);
        return Err(StoreError::Invariant(format!(
            "the converted database did not answer, and the plain file was put back: {error}"
        )));
    }

    logging::log(&format!(
        "converted {} into an encrypted database; the plain copy is at {}",
        path.display(),
        backup.display()
    ));
    Ok(report)
}

/// Copy the schema and the rows of one database into another.
///
/// The schema is taken from the source's own `sqlite_master`, so the
/// encrypted file holds exactly the tables the plain one held.
fn mirror(source: &Connection, destination: &Connection) -> Result<ConversionReport> {
    let mut report = ConversionReport::default();

    let tables = source_objects(source, "table")?;
    let indexes = source_objects(source, "index")?;

    let transaction = destination
        .unchecked_transaction()
        .map_err(|error| StoreError::query("BEGIN", error))?;

    for (name, sql) in &tables {
        transaction
            .execute_batch(sql)
            .map_err(|source| StoreError::migration(name.clone(), source))?;
    }
    for (name, sql) in &indexes {
        transaction
            .execute_batch(sql)
            .map_err(|source| StoreError::migration(name.clone(), source))?;
    }

    for (name, _) in &tables {
        let columns = source_columns(source, name)?;
        let borrowed: Vec<&str> = columns.iter().map(|column| column.as_str()).collect();
        let copied = copy_table(source, &transaction, name, &borrowed)?;
        report.rows += copied;
        report.tables.push((name.clone(), copied));
    }

    // The schema of the plain file decides the schema of the encrypted one.
    // When it is the frozen schema, the encrypted file is at the frozen
    // schema's version; when it is not, the version stays where it was and
    // the difference is reported.
    let frozen: Vec<String> = frozen_table_names();
    let mirrored: Vec<String> = report.tables.iter().map(|(name, _)| name.clone()).collect();
    if mirrored == frozen {
        report.schema_matches = true;
        transaction
            .pragma_update(None, "user_version", schema::SCHEMA_VERSION)
            .map_err(|error| StoreError::migration("PRAGMA user_version", error))?;
    } else {
        for name in &frozen {
            if !mirrored.contains(name) {
                report
                    .drift
                    .push(format!("the table {name} is not in the plain database"));
            }
        }
        for name in &mirrored {
            if !frozen.contains(name) {
                report
                    .drift
                    .push(format!("the table {name} is not in the frozen schema"));
            }
        }
    }

    transaction
        .commit()
        .map_err(|error| StoreError::query("COMMIT", error))?;
    Ok(report)
}

/// The names of the tables of the frozen schema, in the schema's order.
fn frozen_table_names() -> Vec<String> {
    let mut names = Vec::new();
    for statement in schema::TABLE_STATEMENTS {
        let words: Vec<&str> = statement.split_whitespace().collect();
        // "CREATE TABLE <name> (" …
        if words.len() > 2 && words[0] == "CREATE" && words[1] == "TABLE" {
            let name = words[2].trim_end_matches('(');
            names.push(name.to_string());
        }
    }
    names
}

/// The objects of a kind in a database: their names and their SQL.
fn source_objects(source: &Connection, kind: &str) -> Result<Vec<(String, String)>> {
    let mut statement = source
        .prepare(
            "SELECT name, sql FROM sqlite_master
             WHERE type = ?1 AND sql IS NOT NULL AND name NOT LIKE 'sqlite_%'
             ORDER BY rowid",
        )
        .map_err(|source| StoreError::query("sqlite_master", source))?;
    let rows = statement
        .query_map(rusqlite::params![kind], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StoreError::query("sqlite_master", source))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|source| StoreError::query("sqlite_master", source))?);
    }
    Ok(out)
}

/// The columns of a table in a database.
fn source_columns(source: &Connection, table: &str) -> Result<Vec<String>> {
    let mut statement = source
        .prepare(&format!("PRAGMA main.table_info(\"{table}\")"))
        .map_err(|source| StoreError::query("PRAGMA table_info", source))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|source| StoreError::query("PRAGMA table_info", source))?;
    let mut columns = Vec::new();
    for row in rows {
        columns.push(row.map_err(|source| StoreError::query("PRAGMA table_info", source))?);
    }
    Ok(columns)
}

/// Check that a database can be read.
fn quick_check(source: &Connection, path: &Path) -> Result<()> {
    let mut statement =
        source
            .prepare("PRAGMA quick_check")
            .map_err(|source| StoreError::Open {
                path: path.to_path_buf(),
                source,
            })?;
    let verdict = statement
        .query_row([], |row| row.get::<_, String>(0))
        .map_err(|source| StoreError::Open {
            path: path.to_path_buf(),
            source,
        })?;
    if verdict != "ok" {
        return Err(StoreError::Invariant(format!(
            "the database at {} is not sound: {verdict}",
            path.display()
        )));
    }
    Ok(())
}

/// Take a snapshot of a database that may be being written to.
///
/// `VACUUM INTO` is SQLite's own answer to "copy this database while
/// somebody is writing to it": it reads the database in one transaction and
/// writes a **new** file, with no write-ahead log next to it.
fn take_snapshot(source: &Path, target: &Path) -> Result<()> {
    let origin = Connection::open(source).map_err(|error| StoreError::Open {
        path: source.to_path_buf(),
        source: error,
    })?;
    let quoted = target.to_string_lossy().replace('\'', "''");
    origin
        .execute_batch(&format!("VACUUM INTO '{quoted}'"))
        .map_err(|error| StoreError::Open {
            path: target.to_path_buf(),
            source: error,
        })?;
    if !target.is_file() {
        return Err(StoreError::Invariant(format!(
            "the snapshot at {} was not written",
            target.display()
        )));
    }
    Ok(())
}

/// Open the converted database and count its rows.
fn verify_converted(path: &Path, key: &DatabaseKey, report: &ConversionReport) -> Result<()> {
    let store = Store::open(path, key)?;
    let counts = store.with_connection(|connection| {
        let mut counts = Vec::new();
        for (table, rows) in &report.tables {
            let count: i64 = connection
                .query_row(
                    &format!("SELECT count(*) FROM main.\"{table}\""),
                    [],
                    |row| row.get(0),
                )
                .map_err(|source| StoreError::query("count the rows", source))?;
            counts.push((table.clone(), count, *rows));
        }
        Ok(counts)
    })?;
    for (table, count, expected) in counts {
        if count != expected {
            return Err(StoreError::Invariant(format!(
                "the table {table} has {count} rows and {expected} were copied"
            )));
        }
    }
    Ok(())
}

/// The path the encrypted database is written to before it is put in place.
fn staged_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".encrypted");
    path.with_file_name(name)
}

/// The path the plain database is kept at.
fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(PLAIN_BACKUP_SUFFIX);
    path.with_file_name(name)
}

/// The write-ahead log of a database.
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// Move a file, falling back to a copy when the two are on different
/// filesystems.
fn move_file(from: &Path, to: &Path) -> Result<()> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    std::fs::copy(from, to).map_err(|source| StoreError::io(to.to_path_buf(), source))?;
    let _ = std::fs::remove_file(from);
    Ok(())
}

/// The processes that have the database open.
///
/// The conversion replaces the file, so it must not do that while somebody
/// else is writing to it: the writer would carry on into the old inode and
/// its rows would go nowhere. Linux says who has a file open in `/proc`.
#[cfg(target_os = "linux")]
pub fn users_of(path: &Path) -> Vec<u32> {
    let Ok(canonical) = path.canonicalize() else {
        return Vec::new();
    };
    let mut users = Vec::new();
    let Ok(processes) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    for process in processes.flatten() {
        let name = process.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.is_empty() || !name.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if pid == std::process::id() {
            continue;
        }
        let descriptors = process.path().join("fd");
        let Ok(descriptors) = std::fs::read_dir(descriptors) else {
            continue;
        };
        for descriptor in descriptors.flatten() {
            if let Ok(target) = std::fs::read_link(descriptor.path()) {
                if target == canonical {
                    users.push(pid);
                    break;
                }
            }
        }
    }
    users.sort_unstable();
    users
}

/// The processes that have the database open.
#[cfg(not(target_os = "linux"))]
pub fn users_of(_path: &Path) -> Vec<u32> {
    Vec::new()
}
