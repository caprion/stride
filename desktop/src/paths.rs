//! Where the rig keeps its files.
//!
//! Three files, in one directory:
//!
//! ```text
//! ~/.local/share/stride/
//!   stride.db          the database, encrypted
//!   db.key             the key that opens it, with 0600
//!   garmin.key         the Garmin key, with 0600 — only if the athlete put it there
//! ```
//!
//! The directory is the user's, and it is created with `0700`: the database
//! is encrypted, but the directory around it should not be a thing other
//! people can walk through.
//!
//! The location follows the platform's own convention — `$XDG_DATA_HOME` on
//! Linux, `%LOCALAPPDATA%` on Windows, `~/.local/share` everywhere else — and
//! `STRIDE_DATA_DIR` overrides it, for a rig that is run from a checkout, or
//! for a test.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// The name of the directory the rig keeps its files in.
pub const DIRECTORY_NAME: &str = "stride";

/// The environment variable that overrides the data directory.
pub const DATA_DIR_VARIABLE: &str = "STRIDE_DATA_DIR";

/// The name of the database file.
pub const DATABASE_NAME: &str = "stride.db";

/// The name of the file the database key lives in.
pub const DATABASE_KEY_NAME: &str = "db.key";

/// The name of the file the Garmin key may live in.
pub const GARMIN_KEY_NAME: &str = "garmin.key";

/// The data directory.
///
/// `STRIDE_DATA_DIR` if it is set, the platform's own data directory
/// otherwise.
pub fn data_dir() -> Result<PathBuf> {
    if let Some(value) = non_empty_var(DATA_DIR_VARIABLE) {
        return Ok(PathBuf::from(value));
    }
    platform_data_dir()
}

/// The platform's data directory, with the rig's own name on the end.
fn platform_data_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(value) = non_empty_var("LOCALAPPDATA") {
            return Ok(PathBuf::from(value).join(DIRECTORY_NAME));
        }
    }
    if let Some(value) = non_empty_var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(value).join(DIRECTORY_NAME));
    }
    if let Some(home) = non_empty_var("HOME") {
        return Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join(DIRECTORY_NAME));
    }
    Err(Error::config(format!(
        "the rig does not know where to keep its files: neither {DATA_DIR_VARIABLE}, nor \
         XDG_DATA_HOME, nor HOME is set"
    )))
}

/// The database file, in a directory.
pub fn database_path(directory: &Path) -> PathBuf {
    directory.join(DATABASE_NAME)
}

/// The database key file, in a directory.
pub fn database_key_path(directory: &Path) -> PathBuf {
    directory.join(DATABASE_KEY_NAME)
}

/// The Garmin key file, in a directory.
pub fn garmin_key_path(directory: &Path) -> PathBuf {
    directory.join(GARMIN_KEY_NAME)
}

/// Create the data directory, with `0700`, if it is not there.
pub fn ensure_data_dir(directory: &Path) -> Result<()> {
    if !directory.exists() {
        std::fs::create_dir_all(directory).map_err(|source| Error::io(directory, source))?;
    }
    set_private_mode(directory)?;
    Ok(())
}

/// Create a file with `0600`, if it is not there already.
///
/// The database is created this way, so that the file the rig writes into
/// is readable by its owner only — and so that the write-ahead log beside
/// it, which SQLite makes with the same permissions, is private too.
pub fn ensure_private_file(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    create_private(path)
}

/// Create a file that only its owner can read.
#[cfg(unix)]
fn create_private(path: &Path) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|source| Error::io(path, source))?;
    Ok(())
}

#[cfg(not(unix))]
fn create_private(path: &Path) -> Result<()> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| Error::io(path, source))?;
    set_private_mode(path)
}

/// Make a directory or file readable by its owner only.
///
/// On the platforms that have POSIX modes, that is `0700` for a directory
/// and `0600` for a file. On the platforms that do not, the directory's own
/// permissions are inherited — the account's own profile directory is
/// already private there — and this does nothing.
pub fn set_private_mode(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(path, permissions).map_err(|source| Error::io(path, source))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// An environment variable that is set and not empty.
fn non_empty_var(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}
