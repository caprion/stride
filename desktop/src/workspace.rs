//! The machine the rig works on.
//!
//! The rig has three things that belong to the machine rather than to the
//! router: the encrypted database, the key that opens it, and the Garmin
//! key. This module holds them together with the small amount of first-run
//! state the rig remembers between launches.
//!
//! ## What is remembered
//!
//! The first-run marker says only **Fresh**, **Skipped**, or **Connected**.
//! It contains no credential and no personal data. It exists so that a
//! person who chose *Not now* gets the honest empty app on the next launch
//! instead of being asked the same first question every time.
//!
//! The Garmin key has its own rules and its own file. See [`crate::key`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use stride_store::Store;

use crate::dbkey::DatabaseSecret;
use crate::error::{Error, Result};
use crate::key::KeyStore;
use crate::paths;

/// What the rig remembers about its first run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstRun {
    /// Nobody has opened the rig yet.
    Fresh,
    /// The person chose *Not now*. The app is open and honestly empty.
    Skipped,
    /// A key was accepted. The next launch opens the app.
    Connected,
}

/// The machine-side services the rig uses.
pub struct Workspace {
    store: Arc<Store>,
    database_key: Arc<DatabaseSecret>,
    keys: Arc<KeyStore>,
    data_dir: PathBuf,
    first_run: Mutex<FirstRun>,
    uploads: UploadRegistry,
}

impl Workspace {
    /// The workspace, and the first-run state already on disk.
    pub fn new(
        store: Arc<Store>,
        database_key: Arc<DatabaseSecret>,
        keys: Arc<KeyStore>,
        data_dir: PathBuf,
    ) -> Workspace {
        let first_run = read_first_run(&data_dir).unwrap_or(FirstRun::Fresh);
        Workspace {
            store,
            database_key,
            keys,
            data_dir,
            first_run: Mutex::new(first_run),
            uploads: UploadRegistry::default(),
        }
    }

    /// The database.
    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    /// The key that opens the database.
    pub fn database_key(&self) -> &Arc<DatabaseSecret> {
        &self.database_key
    }

    /// The Garmin key holder.
    pub fn keys(&self) -> &Arc<KeyStore> {
        &self.keys
    }

    /// The directory the rig keeps its files in.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// The file the Garmin key may be kept in.
    pub fn garmin_key_path(&self) -> PathBuf {
        paths::garmin_key_path(&self.data_dir)
    }

    /// The registry for imports uploaded from a browser.
    pub(crate) fn uploads(&self) -> &UploadRegistry {
        &self.uploads
    }

    /// The state of first run.
    pub fn first_run(&self) -> FirstRun {
        match self.first_run.lock() {
            Ok(first_run) => *first_run,
            Err(_) => FirstRun::Fresh,
        }
    }

    /// Should the first-run page stand in front of the app?
    ///
    /// A held key means the person is already connected. The *Not now*
    /// marker means they asked for the honest empty app. Only a genuinely
    /// fresh rig shows the page.
    pub fn should_show_connect(&self) -> bool {
        !self.keys.is_set() && self.first_run() == FirstRun::Fresh
    }

    /// Remember that the person chose *Not now*.
    pub fn remember_skipped(&self) -> Result<()> {
        self.remember_first_run(FirstRun::Skipped)
    }

    /// Remember that a key was accepted.
    pub fn remember_connected(&self) -> Result<()> {
        self.remember_first_run(FirstRun::Connected)
    }

    /// Write the first-run state down.
    fn remember_first_run(&self, state: FirstRun) -> Result<()> {
        paths::write_private_atomic(&paths::first_run_path(&self.data_dir), state.as_bytes())?;
        if let Ok(mut current) = self.first_run.lock() {
            *current = state;
        }
        Ok(())
    }
}

/// One import being uploaded.
#[derive(Debug)]
struct UploadSession {
    directory: PathBuf,
    bytes: usize,
}

/// Uploads that are being assembled into an export directory.
///
/// A browser cannot tell the rig an absolute path. It can, however, send
/// the three files of an export one at a time. The registry gives that
/// upload a bounded, private directory and an allow-list of names; no name
/// from the browser is ever used as a path.
#[derive(Debug, Default)]
pub(crate) struct UploadRegistry {
    sessions: Mutex<HashMap<String, UploadSession>>,
}

impl UploadRegistry {
    /// Start an upload and return its private identifier.
    pub(crate) fn start(&self, root: &Path) -> Result<String> {
        std::fs::create_dir_all(root).map_err(|source| Error::io(root, source))?;
        paths::set_private_mode(root)?;

        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| Error::config("the upload registry's lock was poisoned"))?;
        if sessions.len() >= 8 {
            return Err(Error::config(
                "there are already eight imports waiting; finish one before starting another",
            ));
        }

        // A browser can disappear between two uploads. Its staged files are
        // not evidence and must not sit on the disk. Active uploads stay.
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !sessions.contains_key(name.as_ref()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(path);
                    }
                }
            }
        }

        let id = uuid::Uuid::new_v4().to_string();
        let directory = root.join(&id);
        std::fs::create_dir(&directory).map_err(|source| Error::io(&directory, source))?;
        paths::set_private_mode(&directory)?;
        sessions.insert(
            id.clone(),
            UploadSession {
                directory,
                bytes: 0,
            },
        );
        Ok(id)
    }

    /// Put one file into an upload.
    pub(crate) fn accept(&self, id: &str, name: &str, bytes: &[u8]) -> Result<()> {
        if !matches!(name, "manifest.json" | "stride.db" | "db.key") {
            return Err(Error::config(
                "an export may contain manifest.json, stride.db and db.key only",
            ));
        }
        if bytes.len() > crate::http::MAX_UPLOAD_BODY_BYTES {
            return Err(Error::config("the uploaded file is larger than the limit"));
        }
        if name == "manifest.json" && bytes.len() > 1024 * 1024 {
            return Err(Error::config(
                "the uploaded manifest is larger than the limit",
            ));
        }
        if name == "db.key" && bytes.len() > 4096 {
            return Err(Error::config(
                "the uploaded key file is larger than the limit",
            ));
        }

        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| Error::config("the upload registry's lock was poisoned"))?;
        let upload = sessions.get_mut(id).ok_or_else(|| {
            Error::config("that import is not waiting for files; start the import again")
        })?;
        if upload.bytes.saturating_add(bytes.len()) > crate::http::MAX_UPLOAD_BODY_BYTES {
            return Err(Error::config(
                "the uploaded export is larger than the limit",
            ));
        }

        let destination = upload.directory.join(name);
        if destination.exists() {
            let old = std::fs::metadata(&destination)
                .map_err(|source| Error::io(&destination, source))?
                .len() as usize;
            upload.bytes = upload.bytes.saturating_sub(old);
        }
        paths::write_private_atomic(&destination, bytes)?;
        upload.bytes = upload.bytes.saturating_add(bytes.len());
        Ok(())
    }

    /// Finish an upload and return its directory.
    pub(crate) fn take(&self, id: &str) -> Result<PathBuf> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| Error::config("the upload registry's lock was poisoned"))?;
        let upload = sessions.remove(id).ok_or_else(|| {
            Error::config("that import is not waiting for files; start the import again")
        })?;
        Ok(upload.directory)
    }
}

impl FirstRun {
    /// The bytes on disk.
    fn as_bytes(self) -> &'static [u8] {
        match self {
            FirstRun::Fresh => b"fresh",
            FirstRun::Skipped => b"skipped",
            FirstRun::Connected => b"connected",
        }
    }

    /// The state named by the bytes on disk.
    fn from_bytes(bytes: &[u8]) -> FirstRun {
        match bytes {
            b"skipped" => FirstRun::Skipped,
            b"connected" => FirstRun::Connected,
            _ => FirstRun::Fresh,
        }
    }
}

/// Read the first-run state.
fn read_first_run(directory: &Path) -> Result<FirstRun> {
    let path = paths::first_run_path(directory);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FirstRun::Fresh);
        }
        Err(source) => return Err(Error::io(&path, source)),
    };
    Ok(FirstRun::from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn the_first_run_states_round_trip() {
        assert_eq!(FirstRun::from_bytes(b"fresh"), FirstRun::Fresh);
        assert_eq!(FirstRun::from_bytes(b"skipped"), FirstRun::Skipped);
        assert_eq!(FirstRun::from_bytes(b"connected"), FirstRun::Connected);
    }

    #[test]
    fn an_unknown_marker_is_treated_as_fresh() {
        assert_eq!(FirstRun::from_bytes(b"something else"), FirstRun::Fresh);
    }
}
