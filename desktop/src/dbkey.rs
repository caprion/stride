//! The key that opens the local database.
//!
//! **It is not the Garmin key, and it is not derived from it.** Two
//! different secrets for two different jobs: one lets the rig talk to
//! Garmin, the other lets the rig read the athlete's own runs. A rig that
//! used one for both would leak the runs to Garmin the moment it asked for
//! a token, and would leak the Garmin key to anyone who had the database.
//!
//! ## What it does
//!
//! * **it is generated on the first run**, from the operating system's
//!   random bytes;
//! * **it is written to `db.key`** in the data directory, with `0600`, and
//!   it is read back from there on the next run;
//! * **it is never returned by an endpoint and never logged** — the type has
//!   no `Display`, no accessor and a fixed `Debug`;
//! * and **it is separate from the Garmin key on disk**: `db.key` and
//!   `garmin.key` are different files, and neither opens the other.
//!
//! ## The database that predates the encryption
//!
//! The old desktop wrote a plain SQLite file. The first run of this rig
//! converts it — once — into an encrypted database with the frozen schema,
//! and **the plain copy is kept beside it**:
//!
//! ```text
//! ~/.local/share/stride/stride.db                 the encrypted database
//! ~/.local/share/stride/stride.db.plaintext-backup  the plain copy, kept
//! ```
//!
//! The plain copy is not a leak: the file was plain before the conversion,
//! and the copy is the way back if the conversion is wrong. It is the
//! athlete's file to delete once the encrypted database opens, and the
//! README says so.

use std::path::Path;

use stride_store::export::{ConversionReport, Format};
use stride_store::DatabaseKey;

use crate::error::{Error, Result};

/// The length of the raw SQLCipher key, in bytes.
pub const KEY_LENGTH: usize = 32;

/// The name of the file the plain copy is kept under.
pub const PLAIN_BACKUP_SUFFIX: &str = ".plaintext-backup";

/// The key that opens the local database.
///
/// The bytes are private. The only way out is [`DatabaseSecret::store_key`],
/// which hands them to the store, and the store never gives them back.
pub struct DatabaseSecret {
    bytes: Vec<u8>,
}

impl std::fmt::Debug for DatabaseSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The bytes are not printed. Not in full, not hashed, not in part.
        f.write_str("DatabaseSecret([redacted])")
    }
}

impl DatabaseSecret {
    /// Make a new key, from the operating system's random bytes.
    ///
    /// Two version-4 UUIDs: 244 bits of entropy, used as a raw 32-byte
    /// SQLCipher key. The generator reads the operating system's random
    /// bytes on every platform the rig runs on.
    pub fn generate() -> Result<DatabaseSecret> {
        let mut bytes = Vec::with_capacity(KEY_LENGTH);
        bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
        bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
        DatabaseSecret::from_bytes(bytes)
    }

    /// Take a key from bytes the caller has.
    ///
    /// A key of the wrong length is refused: SQLCipher's raw keys are 32
    /// bytes, and a shorter one would be a passphrase the rig did not mean
    /// to write.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DatabaseSecret> {
        if bytes.len() != KEY_LENGTH {
            return Err(Error::key(format!(
                "the database key is {} bytes, and it has to be {KEY_LENGTH}",
                bytes.len()
            )));
        }
        Ok(DatabaseSecret { bytes })
    }

    /// Read a key from a file.
    pub fn load(path: &Path) -> Result<DatabaseSecret> {
        let text = std::fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        let bytes = hex_decode(text.trim()).ok_or_else(|| {
            Error::key(format!(
                "the database key at {} is not readable",
                path.display()
            ))
        })?;
        DatabaseSecret::from_bytes(bytes)
    }

    /// Read the key, or make one and write it down.
    ///
    /// The file is written with `0600`, and it is written **once**: a key
    /// that changed would lock the athlete out of their own runs.
    pub fn load_or_create(path: &Path) -> Result<DatabaseSecret> {
        if path.exists() {
            return DatabaseSecret::load(path);
        }
        let database_key = DatabaseSecret::generate()?;
        database_key.write(path)?;
        Ok(database_key)
    }

    /// Write the key down, with `0600`.
    ///
    /// The file must not be there already: overwriting a key is how a
    /// database becomes unreadable.
    pub fn write(&self, path: &Path) -> Result<()> {
        let text = hex_encode(&self.bytes);
        write_private(path, text.as_bytes())
    }

    /// The key, as the store wants it.
    ///
    /// This is the only way out of the type. The store applies it to the
    /// connection and keeps no copy, so nothing can read it back.
    pub fn store_key(&self) -> Result<DatabaseKey> {
        DatabaseKey::from_bytes(self.bytes.clone()).map_err(|error| Error::key(error.to_string()))
    }
}

/// Convert a database that predates the encryption.
///
/// A plain file is converted once, into an encrypted file with the frozen
/// schema, and the plain copy is kept beside it as `<name>.plaintext-backup`.
/// An encrypted file is left alone, and a file that is not there is left
/// alone.
///
/// The answer says whether anything was converted.
pub fn convert_plaintext_database(
    database: &Path,
    database_key: &DatabaseSecret,
) -> Result<Option<ConversionReport>> {
    match stride_store::export::detect_format(database)? {
        Format::Plaintext => {
            let key = database_key.store_key()?;
            let report = stride_store::export::convert_plaintext_to_encrypted(database, &key)?;
            Ok(Some(report))
        }
        Format::Encrypted | Format::Missing => Ok(None),
    }
}

/// Where the plain copy of a converted database is kept.
pub fn plaintext_backup_path(database: &Path) -> std::path::PathBuf {
    let mut name = database.as_os_str().to_os_string();
    name.push(PLAIN_BACKUP_SUFFIX);
    std::path::PathBuf::from(name)
}

/// The bytes of the key, as they are written down: hexadecimal.
fn hex_encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
        text.push(char::from_digit(u32::from(byte & 0x0f), 16).unwrap_or('0'));
    }
    text
}

/// The bytes of a key that was written down.
fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(text.len() / 2);
    let characters: Vec<char> = text.chars().collect();
    for pair in characters.chunks(2) {
        let high = pair.first()?.to_digit(16)?;
        let low = pair.get(1)?.to_digit(16)?;
        bytes.push(((high << 4) | low) as u8);
    }
    Some(bytes)
}

/// Write a file that only its owner can read.
#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|source| Error::io(path, source))?;
    file.write_all(bytes)
        .map_err(|source| Error::io(path, source))?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| Error::io(path, source))?;
    file.write_all(bytes)
        .map_err(|source| Error::io(path, source))?;
    crate::paths::set_private_mode(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_key_is_the_right_length() -> Result<()> {
        let database_key = DatabaseSecret::generate()?;
        let key = database_key.store_key()?;
        assert_eq!(format!("{key:?}"), "DatabaseKey([redacted])");
        Ok(())
    }

    #[test]
    fn a_key_survives_a_round_trip_through_hex() -> Result<()> {
        let database_key = DatabaseSecret::generate()?;
        let text = hex_encode(&database_key.bytes);
        let bytes = hex_decode(&text).ok_or_else(|| Error::key("the hex did not decode"))?;
        assert_eq!(bytes, database_key.bytes);
        Ok(())
    }

    #[test]
    fn a_key_of_the_wrong_length_is_refused() {
        assert!(DatabaseSecret::from_bytes(vec![0; 16]).is_err());
        assert!(DatabaseSecret::from_bytes(vec![0; 32]).is_ok());
    }
}
