//! Scratch directories.
//!
//! A scratch directory is a temporary directory that removes itself when it
//! is dropped. Tests use one for the temporary SQLCipher database they open;
//! the export uses one for the file it stages before it puts it in place.
//!
//! The removal happens in [`Drop`], which means it happens on the way out of a
//! panic as well as on the way out of a normal return: a test that fails does
//! not leave a database full of somebody's training behind.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Result, StoreError};

/// A counter, so that two scratch directories in the same process never share
/// a name.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A temporary directory that removes itself.
#[derive(Debug)]
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    /// Make a scratch directory under the system's temporary directory.
    ///
    /// The name is unique per process and per call, so two runs of the same
    /// test never collide.
    pub fn new(prefix: &str) -> Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.as_nanos())
            .unwrap_or(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let name = format!("stride-{prefix}-{pid}-{nanos}-{count}");
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&path).map_err(|source| StoreError::io(&path, source))?;
        Ok(ScratchDir { path })
    }

    /// The directory's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A path inside the directory. The file is not created.
    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        // A failure to remove the directory is not worth failing a test over;
        // the directory is under the system's temporary directory and the
        // operating system will clean it up.
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
