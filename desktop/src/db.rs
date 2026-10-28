//! The database.
//!
//! The desktop opens `backend/data/stride.db` with SQLCipher, through
//! `stride-store`, and it opens the **existing** file: a year of history is
//! in that file and losing it is not an option.
//!
//! ## The database the reference writes is not encrypted
//!
//! The Python app writes a plain SQLite file — `backend/app/db.py` has no
//! cipher in it at all — while `docs/rust-core-spec.md` and the Android
//! workstream both say the database is SQLCipher. The file on this machine
//! starts with `SQLite format 3`, so it is the plain one, and SQLCipher
//! refuses it: `file is not a database`.
//!
//! So the desktop converts it, once, on the first run:
//!
//! * the plain file is read and written into a new, encrypted file, with the
//!   same schema and the same rows;
//! * the plain file is kept as `<name>.plaintext-backup`, so nothing is lost
//!   if the conversion is wrong;
//! * the conversion reports what it found — the tables it copied, the rows
//!   it copied, and any drift between the two schemas.
//!
//! The key comes from `STRIDE_DB_KEY`, from Azure Key Vault, or — when
//! neither has one — from a local key file next to the database. The last
//! one is a convenience for a machine that has no vault, and it is written
//! with mode 0600.

use std::path::{Path, PathBuf};
use std::process::Command;

use stride_store::{Store, StoreError};

/// The name of the Key Vault secret that holds the database key.
pub const KEYVAULT_SECRET_ENV: &str = "STRIDE_KEYVAULT_SECRET";

/// The default Key Vault secret name.
pub const KEYVAULT_SECRET_DEFAULT: &str = "stride-db-key";

/// The default Key Vault name.
pub const KEYVAULT_NAME_DEFAULT: &str = "claw-agent";

/// What can go wrong with the database.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// The store could not do it.
    #[error("the database could not be opened: {0}")]
    Store(#[from] StoreError),
    /// A file could not be read or written.
    #[error("could not use the database file at {path}: {source}")]
    Io {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The legacy database could not be read.
    #[error("could not read the legacy database at {path}: {source}")]
    Legacy {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: rusqlite::Error,
    },
    /// The conversion did not work.
    #[error("the conversion failed: {0}")]
    Conversion(String),
    /// There is no key and none could be made.
    #[error("no database key: {0}")]
    NoKey(String),
}

/// How the database file is encrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// There is no file yet.
    Missing,
    /// A plain SQLite file — what the Python app writes.
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

/// What the conversion found.
#[derive(Debug, Clone, Default)]
pub struct ConversionReport {
    /// The tables that were copied, with the number of rows in each.
    pub tables: Vec<(String, i64)>,
    /// The rows that were copied, in total.
    pub rows: i64,
    /// Anything the two schemas disagreed about.
    pub drift: Vec<String>,
}

impl ConversionReport {
    /// A one-line summary, for the log.
    pub fn summary(&self) -> String {
        format!("{} tables, {} rows", self.tables.len(), self.rows)
    }
}

/// What is the file?
///
/// A SQLCipher file has a random header, so the only way to tell the two
/// apart is to look for SQLite's own magic at the front of the file.
pub fn detect_format(path: &Path) -> Result<Format, DbError> {
    let mut head = [0u8; 16];
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Format::Missing),
        Err(source) => {
            return Err(DbError::Io {
                path: path.to_path_buf(),
                source,
            })
        }
    };
    use std::io::Read;
    let read = file.read(&mut head).map_err(|source| DbError::Io {
        path: path.to_path_buf(),
        source,
    })?;
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

/// Open the database, converting it first if it is the plain one.
///
/// The store is migrated after it is opened, exactly as the reference's
/// `init_db` does: the tables are created if they are missing and the
/// additive columns are added.
pub fn open_database(
    path: &Path,
    key: &[u8],
) -> Result<(Store, Option<ConversionReport>), DbError> {
    match detect_format(path)? {
        Format::Missing => {
            let store = Store::open_and_migrate(path, key)?;
            harden(path);
            Ok((store, None))
        }
        Format::Encrypted => {
            let store = Store::open(path, key)?;
            store.migrate()?;
            harden(path);
            Ok((store, None))
        }
        Format::Plaintext => {
            let report = convert_plaintext_to_encrypted(path, key)?;
            let store = Store::open(path, key)?;
            store.migrate()?;
            harden(path);
            Ok((store, Some(report)))
        }
    }
}

/// Make the database and its sidecars owner-only.
///
/// The reference does the same thing in `harden_storage`: a year of heart
/// rate, sleep and GPS routes is not a file other processes on this machine
/// should be able to read.
fn harden(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for suffix in ["", "-wal", "-shm"] {
            let sidecar = sidecar(path, suffix);
            if sidecar.exists() {
                let _ = std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Convert the plain database at `path` into an encrypted one, in place.
///
/// The plain file is kept as `<name>.plaintext-backup`.
pub fn convert_plaintext_to_encrypted(
    path: &Path,
    key: &[u8],
) -> Result<ConversionReport, DbError> {
    let staged = staged_path(path);
    let backup = backup_path(path);

    // A stale staged file from an interrupted conversion would be read as
    // the source of truth, so it goes first.
    let _ = std::fs::remove_file(&staged);

    checkpoint_legacy(path)?;

    let store = Store::open_and_migrate(&staged, key)?;
    let source = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let report = store
        .with_connection(|conn| {
            let report = copy_tables(conn, &source)?;
            // Fold the write-ahead log back into the file, so that the file
            // that is renamed into place is complete on its own.
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|source| StoreError::query("wal_checkpoint".to_string(), source))?;
            Ok(report)
        })
        .map_err(|error| DbError::Conversion(error.to_string()))?;
    drop(store);

    let _ = std::fs::remove_file(sidecar(&staged, "-wal"));
    let _ = std::fs::remove_file(sidecar(&staged, "-shm"));

    // Keep the plain file: the conversion is the one operation here that
    // cannot be undone.
    copy_file(path, &backup)?;
    copy_file(&sidecar(path, "-wal"), &sidecar(&backup, "-wal"))?;
    copy_file(&sidecar(path, "-shm"), &sidecar(&backup, "-shm"))?;

    // The plain file's write-ahead log belongs to the plain file. Leaving it
    // next to an encrypted database would make SQLite try to apply it.
    let _ = std::fs::remove_file(sidecar(path, "-wal"));
    let _ = std::fs::remove_file(sidecar(path, "-shm"));

    std::fs::rename(&staged, path).map_err(|source| DbError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(report)
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
    name.push(".plaintext-backup");
    path.with_file_name(name)
}

/// The write-ahead log of a database.
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// Copy a file, if it is there.
fn copy_file(from: &Path, to: &Path) -> Result<(), DbError> {
    if !from.exists() {
        return Ok(());
    }
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|source| DbError::Io {
            path: to.to_path_buf(),
            source,
        })
}

/// Merge the legacy database's write-ahead log into its main file.
///
/// The legacy file is the Python app's, and it may have a log with rows in
/// it that are not in the main file yet. Copying the file without the log
/// would copy a database that is missing the most recent rows.
fn checkpoint_legacy(path: &Path) -> Result<(), DbError> {
    if !sidecar(path, "-wal").exists() {
        return Ok(());
    }
    let connection = rusqlite::Connection::open(path).map_err(|source| DbError::Legacy {
        path: path.to_path_buf(),
        source,
    })?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|source| DbError::Legacy {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

/// Copy every table of the legacy database into the encrypted one.
///
/// The columns are matched by name, not by position: the two schemas have
/// the same columns in a different order in three tables, and a copy by
/// position would put a timestamp in a distance column.
///
/// The two databases are read and written by two connections, because
/// SQLCipher will not attach a plain database to an encrypted one — it reads
/// the plain header as an encrypted page and gives up. So the rows are read
/// from the plain connection and written to the encrypted one, one row at a
/// time, with the values passed through as they are.
fn copy_tables(
    conn: &rusqlite::Connection,
    source_path: &Path,
) -> Result<ConversionReport, stride_store::StoreError> {
    let mut report = ConversionReport::default();

    let source = rusqlite::Connection::open(source_path)
        .map_err(|source| StoreError::query(format!("open {}", source_path.display()), source))?;

    conn.execute_batch("PRAGMA foreign_keys=OFF;")
        .map_err(|source| StoreError::query("PRAGMA foreign_keys=OFF".to_string(), source))?;

    let tables = strings(
        &source,
        "SELECT name FROM main.sqlite_master WHERE type='table' \
         AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;

    for table in tables {
        let target_columns = columns(conn, &table)?;
        let source_columns = columns(&source, &table)?;
        if target_columns.is_empty() {
            report
                .drift
                .push(format!("the table {table} is not in the current schema"));
            continue;
        }
        let shared: Vec<String> = target_columns
            .iter()
            .filter(|column| source_columns.contains(column))
            .cloned()
            .collect();
        for column in &source_columns {
            if !target_columns.contains(column) {
                report.drift.push(format!(
                    "the column {table}.{column} is not in the current schema"
                ));
            }
        }
        for column in &target_columns {
            if !source_columns.contains(column) {
                report.drift.push(format!(
                    "the column {table}.{column} is not in the old database"
                ));
            }
        }
        if shared.is_empty() {
            report
                .drift
                .push(format!("the table {table} has no column in common"));
            continue;
        }

        let list = shared
            .iter()
            .map(|column| format!("\"{column}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let select = format!("SELECT {list} FROM main.\"{table}\"");
        let insert = format!("INSERT INTO main.\"{table}\" ({list}) VALUES ({})", {
            let mut placeholders = Vec::with_capacity(shared.len());
            for index in 1..=shared.len() {
                placeholders.push(format!("?{index}"));
            }
            placeholders.join(", ")
        });

        let mut statement = source
            .prepare(&select)
            .map_err(|error| StoreError::query(select.clone(), error))?;
        let mut rows = statement
            .query([])
            .map_err(|error| StoreError::query(select.clone(), error))?;
        let mut prepared = conn
            .prepare(&insert)
            .map_err(|error| StoreError::query(insert.clone(), error))?;

        let mut copied = 0i64;
        while let Some(row) = rows
            .next()
            .map_err(|error| StoreError::query(select.clone(), error))?
        {
            let mut values = Vec::with_capacity(shared.len());
            for index in 0..shared.len() {
                let value = row
                    .get::<_, rusqlite::types::Value>(index)
                    .map_err(|error| StoreError::query(select.clone(), error))?;
                values.push(value);
            }
            prepared
                .execute(rusqlite::params_from_iter(values.iter()))
                .map_err(|error| StoreError::query(insert.clone(), error))?;
            copied += 1;
        }
        report.rows += copied;
        report.tables.push((table, copied));
    }

    // The foreign keys are checked after the copy, not during it: the tables
    // arrive in alphabetical order and the children arrive before their
    // parents.
    let violations = conn
        .prepare("PRAGMA foreign_key_check")
        .and_then(|mut statement| {
            let mut rows = statement.query([])?;
            let mut count = 0i64;
            while rows.next()?.is_some() {
                count += 1;
            }
            Ok(count)
        })
        .unwrap_or(0);
    if violations > 0 {
        report
            .drift
            .push(format!("{violations} rows break a foreign key"));
    }

    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|source| StoreError::query("PRAGMA foreign_keys=ON".to_string(), source))?;

    Ok(report)
}

/// The columns of a table.
fn columns(
    conn: &rusqlite::Connection,
    table: &str,
) -> Result<Vec<String>, stride_store::StoreError> {
    let statement = format!("PRAGMA main.table_info(\"{table}\")");
    let mut prepared = conn
        .prepare(&statement)
        .map_err(|source| StoreError::query(statement.clone(), source))?;
    let rows = prepared
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|source| StoreError::query(statement.clone(), source))?;
    let mut columns = Vec::new();
    for row in rows {
        match row {
            Ok(column) => columns.push(column),
            Err(error) => {
                return Err(StoreError::query(statement.clone(), error));
            }
        }
    }
    Ok(columns)
}

/// The strings of a query that returns one column.
fn strings(
    conn: &rusqlite::Connection,
    sql: &str,
) -> Result<Vec<String>, stride_store::StoreError> {
    let mut prepared = conn
        .prepare(sql)
        .map_err(|source| StoreError::query(sql.to_string(), source))?;
    let rows = prepared
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| StoreError::query(sql.to_string(), source))?;
    let mut values = Vec::new();
    for row in rows {
        match row {
            Ok(value) => values.push(value),
            Err(error) => return Err(StoreError::query(sql.to_string(), error)),
        }
    }
    Ok(values)
}

/// Find the key the database is encrypted with.
///
/// The order is the order the constitution gives: the environment, then Azure
/// Key Vault, then — only because there was nothing else — a local key file
/// beside the database.
pub fn resolve_key(
    explicit: Option<Vec<u8>>,
    data_dir: &Path,
) -> Result<(Vec<u8>, crate::config::KeySource), DbError> {
    if let Some(key) = explicit {
        if !key.is_empty() {
            return Ok((key, crate::config::KeySource::Environment));
        }
    }
    if let Some(key) = key_from_vault()? {
        return Ok((key, crate::config::KeySource::KeyVault));
    }
    let path = data_dir.join("stride-desktop.key");
    let key = read_or_create_key(&path)?;
    Ok((key, crate::config::KeySource::LocalFile))
}

/// Read the key from Azure Key Vault, with the Azure CLI.
///
/// The CLI is invoked with an argument list and without a shell, so a vault
/// or a secret name cannot be interpolated into a command.
pub fn key_from_vault() -> Result<Option<Vec<u8>>, DbError> {
    let vault = std::env::var("STRIDE_KEYVAULT_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| KEYVAULT_NAME_DEFAULT.to_string());
    let secret = std::env::var(KEYVAULT_SECRET_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| KEYVAULT_SECRET_DEFAULT.to_string());

    let output = Command::new("az")
        .args([
            "keyvault",
            "secret",
            "show",
            "--vault-name",
            &vault,
            "--name",
            &secret,
            "--query",
            "value",
            "-o",
            "tsv",
        ])
        .output();
    let Ok(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() || value.contains('\n') {
        return Ok(None);
    }
    Ok(Some(value.into_bytes()))
}

/// The local key file: read it, or make it.
fn read_or_create_key(path: &Path) -> Result<Vec<u8>, DbError> {
    if let Ok(raw) = std::fs::read_to_string(path) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.as_bytes().to_vec());
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| DbError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let key = crate::state::random_token();
    std::fs::write(path, &key).map_err(|source| DbError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(key.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_missing() {
        let path = std::env::temp_dir().join("stride-desktop-absent.db");
        let _ = std::fs::remove_file(&path);
        assert_eq!(detect_format(&path).expect("the format"), Format::Missing);
    }

    #[test]
    fn a_plain_file_is_plaintext() {
        let path = std::env::temp_dir().join("stride-desktop-plain.db");
        std::fs::write(&path, b"SQLite format 3\0 and then some").expect("the file");
        assert_eq!(detect_format(&path).expect("the format"), Format::Plaintext);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_encrypted_file_is_not_plaintext() {
        let path = std::env::temp_dir().join("stride-desktop-encrypted.db");
        std::fs::write(&path, [0x7fu8; 32]).expect("the file");
        assert_eq!(detect_format(&path).expect("the format"), Format::Encrypted);
        let _ = std::fs::remove_file(&path);
    }
}
