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
//! * **an environment variable** — `STRIDE_GARMIN_KEY` (or the
//!   `GARMIN_API_KEY` that `cf-garmin env` provides) — for a rig that a
//!   script starts;
//! * **a file with `0600`** in the data directory — for a rig that is
//!   restarted often. The first-run page writes this file only after Garmin
//!   has accepted the key and the athlete has asked to connect.
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
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::{Error, Result};

/// The environment variable the Garmin key may be in.
pub const GARMIN_KEY_VARIABLE: &str = "STRIDE_GARMIN_KEY";

/// The environment variable `cf-garmin env` provides.
///
/// It is accepted as a second name so the development helper works without
/// a wrapper that copies the secret from one variable to another.
pub const GARMIN_API_KEY_VARIABLE: &str = "GARMIN_API_KEY";

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
    persist_path: Option<PathBuf>,
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
            persist_path: None,
        }
    }

    /// The store that may keep the key in a private file.
    ///
    /// The path is only a destination. Nothing is written until
    /// [`KeyStore::persist`] is called, and that happens only after the
    /// first-run flow has accepted the key.
    pub fn with_persist_path(path: impl Into<PathBuf>) -> KeyStore {
        let mut store = KeyStore::empty();
        store.persist_path = Some(path.into());
        store
    }

    /// A store that holds the key in `STRIDE_GARMIN_KEY`.
    ///
    /// An unset or empty variable means the rig holds no key — that is not
    /// an error, it is the state before the athlete connects.
    pub fn from_environment() -> Result<KeyStore> {
        let store = KeyStore::empty();
        let value = match environment_value() {
            Some(value) => value,
            None => return Ok(store),
        };
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
        let store = KeyStore::with_persist_path(path);
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

    /// Keep the key in the private file this store was given.
    ///
    /// A store with no destination keeps the key in memory, which is what a
    /// test and a short-lived process want. The bytes are written through
    /// the paths helper, which creates the file with `0600` and replaces it
    /// atomically.
    pub fn persist(&self) -> Result<()> {
        let Some(path) = &self.persist_path else {
            return Ok(());
        };
        let value = match self.with_secret(|secret| secret.to_string()) {
            Some(value) => value,
            None => return Err(Error::key("there is no Garmin key to keep")),
        };
        crate::paths::write_private_atomic(path, value.as_bytes())?;
        self.mark(Source::File);
        Ok(())
    }

    /// Remove the private file this store may have written.
    ///
    /// A file that is not there is not an error: forgetting a key that was
    /// never written down is complete.
    pub fn remove_persisted(&self) -> Result<()> {
        let Some(path) = &self.persist_path else {
            return Ok(());
        };
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(Error::io(path, source)),
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

/// The first non-empty environment variable that may hold the key.
///
/// `STRIDE_GARMIN_KEY` is the rig's own name. `GARMIN_API_KEY` is what
/// `cf-garmin env` sets, and accepting it means the development helper
/// works with the command exactly as its help says.
fn environment_value() -> Option<String> {
    for name in [GARMIN_KEY_VARIABLE, GARMIN_API_KEY_VARIABLE] {
        if let Ok(value) = std::env::var(name) {
            if !value.trim().is_empty() {
                return Some(value);
            }
        }
    }
    None
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

    #[test]
    fn a_kept_key_is_private() -> Result<()> {
        let directory = std::env::temp_dir().join("stride-key-persist-test");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory)?;
        let path = directory.join("garmin.key");
        let store = KeyStore::with_persist_path(&path);
        store.set("test-value")?;
        store.persist()?;

        assert_eq!(std::fs::read_to_string(&path)?, "test-value");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)?.permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        store.remove_persisted()?;
        assert!(!path.exists());

        let _ = std::fs::remove_dir_all(&directory);
        Ok(())
    }
}
