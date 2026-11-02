//! Export, import, and the files a person can move between machines.
//!
//! The store owns the format: a manifest, a digest, and a database with the
//! frozen schema. This module is the rig around it. It chooses a private
//! place for an export, says what is in it, and reads one back through the
//! store's confined, digest-checked import.
//!
//! ## The export is a bundle, not a loose database
//!
//! An export directory holds three files:
//!
//! ```text
//! stride-export-<time>/
//!   manifest.json    the format version, schema version, row counts and digest
//!   stride.db        the encrypted database
//!   db.key           the key that opens this export
//! ```
//!
//! `db.key` is a **new key made for this export**. It is not the key that
//! opens the rig's live database, so a copied export never opens the live
//! database. The bundle is still sensitive: the key and the athlete's rows
//! travel together. The rig keeps it under a private directory with `0700`
//! and gives each file `0600`.
//!
//! The Garmin key is never in an export. It is a credential for Garmin, not
//! data about the athlete, and the export has no use for it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use stride_store::export;

use crate::dbkey::DatabaseSecret;
use crate::error::{Error, Result};
use crate::paths;
use crate::workspace::Workspace;

/// What an export wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSummary {
    /// The directory the export is in.
    pub path: PathBuf,
    /// How many rows the export holds.
    pub rows: i64,
    /// The export format version.
    pub format_version: i64,
    /// The database schema version.
    pub schema_version: i64,
    /// The rows in each table.
    pub tables: BTreeMap<String, i64>,
}

/// What an import read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    /// The directory the import came from.
    pub path: PathBuf,
    /// How many rows were read from the export.
    pub rows: usize,
    /// The export format version.
    pub format_version: i64,
    /// The database schema version.
    pub schema_version: i64,
    /// How many runs were merged.
    pub activities: usize,
    /// How many wellness days were merged.
    pub wellness: usize,
    /// How many plan blocks were merged.
    pub plan_blocks: usize,
    /// How many planned sessions were merged.
    pub plan_sessions: usize,
    /// How many notes were merged.
    pub notes: usize,
    /// Anything the import refused. A failed import refuses the whole
    /// bundle and returns a failure instead; a successful one has nothing
    /// to refuse.
    pub refused: Vec<String>,
}

/// Write an export into the rig's private exports directory.
pub fn export(workspace: &Workspace) -> Result<ExportSummary> {
    let root = paths::exports_directory(workspace.data_dir());
    std::fs::create_dir_all(&root).map_err(|source| Error::io(&root, source))?;
    paths::set_private_mode(&root)?;
    let stamp = export_stamp();
    let mut candidate = root.join(format!("stride-export-{stamp}"));
    let mut suffix = 1u32;
    while candidate.exists() {
        candidate = root.join(format!("stride-export-{stamp}-{suffix}"));
        suffix += 1;
    }
    export_to(workspace, &candidate)
}

/// Write an export into a directory the athlete named.
pub fn export_to(workspace: &Workspace, destination: &Path) -> Result<ExportSummary> {
    if destination.exists() {
        let mut entries =
            std::fs::read_dir(destination).map_err(|source| Error::io(destination, source))?;
        if entries.next().is_some() {
            return Err(Error::config(format!(
                "the export destination {} already has files in it",
                destination.display()
            )));
        }
    }

    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;

    // The export gets its own key. The live database key stays here.
    let export_key = DatabaseSecret::generate()?;
    let manifest = match export::export(workspace.store(), destination, &export_key.store_key()?) {
        Ok(manifest) => manifest,
        Err(error) => {
            let _ = std::fs::remove_dir_all(destination);
            return Err(Error::Store(error));
        }
    };

    let key_path = destination.join(paths::DATABASE_KEY_NAME);
    if let Err(error) = export_key.write(&key_path) {
        let _ = std::fs::remove_dir_all(destination);
        return Err(error);
    }

    // The store creates the files, so the rig tightens their modes after
    // the write. An export is private even when its directory is on a
    // shared filesystem.
    paths::set_private_mode(destination)?;
    paths::set_private_mode(&destination.join(export::DATABASE_NAME))?;
    paths::set_private_mode(&destination.join(export::MANIFEST_NAME))?;
    paths::set_private_mode(&key_path)?;

    Ok(ExportSummary {
        path: destination.to_path_buf(),
        rows: manifest.rows,
        format_version: manifest.format_version,
        schema_version: manifest.schema_version,
        tables: manifest.tables,
    })
}

/// Read an export back into the rig's database.
///
/// The import is a merge: rows already in the rig are updated, rows that
/// are not are added, and importing the same export twice adds nothing the
/// second time. The store verifies the digest before it opens the database.
pub fn import(workspace: &Workspace, source: &Path) -> Result<ImportSummary> {
    let manifest = export::read_manifest(source)?;
    let key = import_key(workspace, source)?;
    let report = export::import(workspace.store(), source, &key)?;

    Ok(ImportSummary {
        path: source.to_path_buf(),
        rows: manifest.rows.max(0) as usize,
        format_version: manifest.format_version,
        schema_version: manifest.schema_version,
        activities: report.activities,
        wellness: report.wellness,
        plan_blocks: report.plan_blocks,
        plan_sessions: report.plan_sessions,
        notes: report.notes,
        refused: Vec::new(),
    })
}

/// The key that opens an export.
///
/// A rig export carries its own key. An older export without one is opened
/// with the rig's live database key, which is the only case in which that
/// key crosses into an import.
fn import_key(workspace: &Workspace, source: &Path) -> Result<stride_store::DatabaseKey> {
    let path = source.join(paths::DATABASE_KEY_NAME);
    if !path.exists() {
        return workspace.database_key().store_key();
    }
    check_private_file(&path)?;
    let loaded_key = DatabaseSecret::load(&path)?;
    loaded_key.store_key()
}

/// Refuse a key file other people can read.
fn check_private_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(Error::key(format!(
                "the export key at {} can be read by other people; run `chmod 600 {}` on it",
                path.display(),
                path.display()
            )));
        }
    }
    Ok(())
}

/// A timestamp that names an export directory.
fn export_stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use std::sync::Arc;

    use stride_store::Store;

    use super::*;
    use crate::key::KeyStore;

    fn workspace(name: &str) -> Result<(Workspace, PathBuf)> {
        let directory = std::env::temp_dir().join(format!("stride-tools-{name}"));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory)?;
        let key = DatabaseSecret::generate()?;
        let store = Arc::new(Store::open_and_migrate(
            &directory.join(paths::DATABASE_NAME),
            &key.store_key()?,
        )?);
        let keys = Arc::new(KeyStore::empty());
        let workspace = Workspace::new(store, Arc::new(key), keys, directory.clone());
        Ok((workspace, directory))
    }

    #[test]
    fn an_export_round_trips() -> Result<()> {
        let (workspace, directory) = workspace("round-trip")?;
        let destination = directory.join("export");
        let written = export_to(&workspace, &destination)?;
        assert_eq!(written.format_version, 1);
        assert_eq!(written.schema_version, stride_store::SCHEMA_VERSION);
        assert!(destination.join("manifest.json").is_file());
        assert!(destination.join("stride.db").is_file());
        assert!(destination.join("db.key").is_file());

        let read = import(&workspace, &destination)?;
        assert_eq!(read.rows, written.rows as usize);
        assert!(read.refused.is_empty());

        let _ = std::fs::remove_dir_all(&directory);
        Ok(())
    }
}
