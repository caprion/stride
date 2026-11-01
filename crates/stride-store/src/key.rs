//! The database key.
//!
//! The key comes from the caller and from nowhere else: the Android Keystore
//! on the phone, Azure Key Vault on the desktop, the environment in a test.
//! The store never reads a key from the environment, from a file, from a
//! prompt, or from anywhere but the argument it is given.
//!
//! ## The rule
//!
//! **The key never comes back out.** There is no accessor, no `Deref`, no
//! `Display`, and the `Debug` output is a fixed phrase. The only thing that
//! can reach the bytes is [`DatabaseKey::apply`], which hands them to SQLite
//! and forgets them. A repository cannot return the key, because no
//! repository can reach it in the first place.
//!
//! ## How it is applied
//!
//! A key of exactly 32 bytes is used as a **raw key** — the bytes are used as
//! they are, with no key derivation, which is what a key out of a Keystore or
//! out of a vault is. Anything else is a **passphrase**, and must be text.
//!
//! The statement that applies the key contains the key, so it is built here,
//! used once, and never logged, never returned and never put in an error. The
//! errors it can cause are rewritten into [`StoreError::InvalidKey`] with a
//! fixed reason.

use std::fmt;

use rusqlite::Connection;

use crate::error::{Result, StoreError};

/// The length of a raw SQLCipher key, in bytes.
pub const RAW_KEY_LEN: usize = 32;

/// An opaque database key.
///
/// The bytes are private. There is no way to read them out of this type from
/// outside the crate, and the only method that uses them hands them to
/// SQLite.
pub struct DatabaseKey {
    bytes: Vec<u8>,
}

impl DatabaseKey {
    /// Take a key from the bytes the caller has.
    ///
    /// An empty key is refused: SQLCipher would accept it as an empty
    /// passphrase, and an unencrypted-looking database with an "empty key" is
    /// a trap.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self> {
        let bytes = bytes.into();
        if bytes.is_empty() {
            return Err(StoreError::InvalidKey {
                reason: "the key is empty".to_string(),
            });
        }
        Ok(DatabaseKey { bytes })
    }

    /// Take a key from text.
    ///
    /// The text is used as a passphrase. It must not be empty.
    pub fn from_text(text: &str) -> Result<Self> {
        if text.is_empty() {
            return Err(StoreError::InvalidKey {
                reason: "the key is empty".to_string(),
            });
        }
        Ok(DatabaseKey {
            bytes: text.as_bytes().to_vec(),
        })
    }

    /// Apply the key to a connection.
    ///
    /// This is the only path from the key to SQLite. The key has to be the
    /// first thing the connection sees: SQLCipher decrypts the file from here
    /// on, and every later statement reads the decrypted pages.
    ///
    /// Any error is rewritten: the statement carries the key, and an error
    /// message must not.
    pub(crate) fn apply(&self, conn: &Connection) -> Result<()> {
        let statement = self.statement();
        conn.execute_batch(&statement).map_err(|source| {
            let _ = &source;
            StoreError::InvalidKey {
                reason: "the key was refused by the database".to_string(),
            }
        })
    }

    /// The `PRAGMA key` statement for this key.
    ///
    /// Private to the crate, and never returned to a caller: the string holds
    /// the key.
    fn statement(&self) -> String {
        if self.bytes.len() == RAW_KEY_LEN {
            use fmt::Write as _;
            let mut hex = String::with_capacity(RAW_KEY_LEN * 2);
            for byte in &self.bytes {
                let _ = write!(hex, "{byte:02x}");
            }
            return format!("PRAGMA key = \"x'{hex}'\";");
        }

        let text = String::from_utf8_lossy(&self.bytes);
        let escaped = text.replace('\'', "''");
        format!("PRAGMA key = '{escaped}';")
    }
}

impl fmt::Debug for DatabaseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The bytes are not printed. Not in full, not hashed, not in part.
        f.write_str("DatabaseKey([redacted])")
    }
}

/// Turn a SQLite error into a key error, without saying anything about the
/// key.
///
/// SQLite's own words for a wrong key are "file is not a database" and "file
/// is encrypted"; they are mapped to a fixed phrase, and everything else to
/// another fixed phrase. The source error is dropped, because a future
/// SQLite might put something else in it.
pub(crate) fn key_failure(source: &rusqlite::Error) -> StoreError {
    let text = source.to_string();
    let reason = if text.contains("not a database") || text.contains("encrypted") {
        "the key does not open this file"
    } else if text.contains("file is empty") || text.contains("empty") {
        "the file is empty"
    } else {
        "the key was refused by the database"
    };
    StoreError::InvalidKey {
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_key_is_refused() {
        assert!(DatabaseKey::from_bytes(Vec::new()).is_err());
        assert!(DatabaseKey::from_text("").is_err());
    }

    #[test]
    fn a_raw_key_is_hexadecimal() -> Result<()> {
        let key = DatabaseKey::from_bytes([0xabu8; 32])?;
        let statement = key.statement();
        assert!(statement.starts_with("PRAGMA key = \"x'"), "{statement}");
        assert!(statement.contains(&"ab".repeat(32)), "{statement}");
        Ok(())
    }

    #[test]
    fn a_passphrase_is_escaped() -> Result<()> {
        let key = DatabaseKey::from_text("it's a passphrase")?;
        let statement = key.statement();
        assert!(statement.starts_with("PRAGMA key = '"), "{statement}");
        assert!(statement.contains("it''s a passphrase"), "{statement}");
        Ok(())
    }

    #[test]
    fn the_debug_output_is_a_fixed_phrase() -> Result<()> {
        let key = DatabaseKey::from_text("correct horse battery staple")?;
        assert_eq!(format!("{key:?}"), "DatabaseKey([redacted])");
        assert!(!format!("{key:?}").contains("horse"));
        Ok(())
    }
}
