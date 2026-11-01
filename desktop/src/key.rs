//! The Garmin key.
//!
//! **One credential for the whole product**: the Garmin Personal API key. It
//! is the thing the athlete pastes into the app's own field, and it is the
//! only secret the athlete ever types.
//!
//! ## Where it lives at runtime
//!
//! Three places, and no more:
//!
//! * **in memory**, for the length of the session — the app's field posts
//!   it, and the rig holds it until it is forgotten;
//! * **an environment variable** — `STRIDE_GARMIN_KEY` — for a rig that a
//!   script starts;
//! * **a file with `0600`** in the data directory — for a rig that is
//!   restarted often.
//!
//! ## What it never does
//!
//! * it is never **logged**: the key type has no `Display`, no accessor and
//!   a fixed `Debug`, so there is nothing to print;
//! * it is never **returned by an endpoint**: no route in the frozen table
//!   answers with it, and the rig's own responses never carry it;
//! * it is never **written into the database**: the database holds runs and
//!   days, not credentials;
//! * and it is never written down in the clear by the rig itself — the file
//!   with `0600` is there because the athlete put it there.
//!
//! ## The rule the type keeps
//!
//! The key is reached through [`GarminKey::with_secret`], which hands it to
//! a closure and forgets it. There is no `&str` getter: a getter is a
//! string that can be printed, and a printed key is a leaked key.

use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use crate::error::{Error, Result};

/// The environment variable the Garmin key may be in.
pub const GARMIN_KEY_VARIABLE: &str = "STRIDE_GARMIN_KEY";

/// A Garmin Personal API key.
///
/// The bytes are private. There is no way to read them out of this type
/// except through [`GarminKey::with_secret`], and the `Debug` output is a
/// fixed phrase.
pub struct GarminKey {
    value: String,
}

impl GarminKey {
    /// Take a key from what the athlete typed.
    ///
    /// An empty key is refused: Garmin would refuse it too, and a rig that
    /// stored it would then hold a credential that opens nothing.
    pub fn new(value: &str) -> Result<GarminKey> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(Error::key("the Garmin key is empty"));
        }
        Ok(GarminKey {
            value: trimmed.to_string(),
        })
    }

    /// Hand the key to something that needs it, and forget it.
    ///
    /// This is the only way out. The Garmin client needs the key to exchange
    /// it for a token, and the exchange happens inside the closure.
    pub fn with_secret<R>(&self, use_it: impl FnOnce(&str) -> R) -> R {
        use_it(&self.value)
    }
}

impl fmt::Debug for GarminKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The key is not printed. Not in full, not hashed, not in part.
        f.write_str("GarminKey([redacted])")
    }
}

/// Where the key came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// It was typed into the app's own field, and it is in memory.
    Memory,
    /// It was read from `STRIDE_GARMIN_KEY`.
    Environment,
    /// It was read from a file with `0600`.
    File,
}

impl Source {
    /// A name for it, for the log and for `doctor`.
    pub fn name(&self) -> &'static str {
        match self {
            Source::Memory => "in memory",
            Source::Environment => "the environment",
            Source::File => "a file",
        }
    }
}

/// The key the rig holds.
///
/// The store is the desktop's answer to "where does the key live at
/// runtime", and it is deliberately small: one key, one source, one way to
/// forget it.
#[derive(Debug, Default)]
pub struct KeyStore {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    key: Option<GarminKey>,
    source: Option<Source>,
}

impl KeyStore {
    /// A store that holds nothing.
    pub fn empty() -> KeyStore {
        KeyStore {
            inner: Mutex::new(Inner::default()),
        }
    }

    /// A store that holds the key in `STRIDE_GARMIN_KEY`.
    ///
    /// An unset or empty variable means the rig holds no key — that is not
    /// an error, it is the state before the athlete connects.
    pub fn from_environment() -> Result<KeyStore> {
        let store = KeyStore::empty();
        let value = match std::env::var(GARMIN_KEY_VARIABLE) {
            Ok(value) => value,
            Err(_) => return Ok(store),
        };
        if value.trim().is_empty() {
            return Ok(store);
        }
        store.set(&value)?;
        store.mark(Source::Environment);
        Ok(store)
    }

    /// A store that holds the key in a file.
    ///
    /// The file must be readable by its owner only. A key in a file other
    /// people can read is a key that is not a secret, and the rig says so
    /// rather than reading it.
    pub fn from_file(path: &Path) -> Result<KeyStore> {
        check_private_file(path)?;
        let text = std::fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        let store = KeyStore::empty();
        store.set(&text)?;
        store.mark(Source::File);
        Ok(store)
    }

    /// Put a key in, in memory.
    ///
    /// The key that was there is replaced, and the new one is in memory only.
    pub fn set(&self, value: &str) -> Result<()> {
        let key = GarminKey::new(value)?;
        let mut inner = self.lock()?;
        inner.key = Some(key);
        inner.source = Some(Source::Memory);
        Ok(())
    }

    /// Forget the key.
    pub fn forget(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.key = None;
            inner.source = None;
        }
    }

    /// Does the rig hold a key?
    pub fn is_set(&self) -> bool {
        match self.inner.lock() {
            Ok(inner) => inner.key.is_some(),
            Err(_) => false,
        }
    }

    /// Where the key came from, if there is one.
    pub fn source(&self) -> Option<Source> {
        match self.inner.lock() {
            Ok(inner) => inner.source,
            Err(_) => None,
        }
    }

    /// Hand the key to something that needs it, and forget it.
    pub fn with_secret<R>(&self, use_it: impl FnOnce(&str) -> R) -> Option<R> {
        let inner = self.lock().ok()?;
        let key = inner.key.as_ref()?;
        Some(key.with_secret(use_it))
    }

    /// Say where the key came from, without changing it.
    fn mark(&self, source: Source) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.source = Some(source);
        }
    }

    /// The lock on the store.
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>> {
        self.inner
            .lock()
            .map_err(|_| Error::key("the key store's lock was poisoned"))
    }
}

/// Is this file readable by its owner only?
///
/// On the platforms that have POSIX modes, the answer is the mode. On the
/// platforms that do not, there is no mode to read, and the file inherits
/// the account's own permissions — which is the answer the platform gives.
#[cfg(unix)]
fn check_private_file(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(Error::key(format!(
            "the Garmin key file {} can be read by other people; run `chmod 600 {}` on it",
            path.display(),
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_private_file(path: &Path) -> Result<()> {
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_key_is_refused() {
        assert!(GarminKey::new("").is_err());
        assert!(GarminKey::new("   ").is_err());
    }

    #[test]
    fn the_debug_output_is_a_fixed_phrase() -> Result<()> {
        let key = GarminKey::new("a key that must not be printed")?;
        assert_eq!(format!("{key:?}"), "GarminKey([redacted])");
        assert!(!format!("{key:?}").contains("printed"));
        Ok(())
    }

    #[test]
    fn a_key_can_be_forgotten() -> Result<()> {
        let store = KeyStore::empty();
        assert!(!store.is_set());
        store.set("a key")?;
        assert!(store.is_set());
        store.forget();
        assert!(!store.is_set());
        Ok(())
    }
}
