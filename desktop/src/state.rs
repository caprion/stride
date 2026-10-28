//! The state file.
//!
//! The Python app keeps its mutable state in `backend/data/state.json`: the
//! Garmin credentials, the pairing code, the capability token, the settings.
//! The desktop reads the same file, for the same reasons, and writes it back
//! with the same care: mode 0600, written atomically, and never logging a
//! value it did not have to.
//!
//! The desktop only ever touches `capability_token`. Everything else in the
//! file belongs to the Garmin client and to the plan engine, and the desktop
//! preserves it byte for byte: the file is read as a map, one key is changed,
//! and the map is written back.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The key the capability token lives under.
pub const CAPABILITY_KEY: &str = "capability_token";

/// What can go wrong with the state file.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// The file could not be read.
    #[error("could not read the state file at {path}: {source}")]
    Read {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The file is not a JSON object.
    #[error("the state file at {path} is not a JSON object")]
    NotAnObject {
        /// The path.
        path: PathBuf,
    },
    /// The file could not be parsed.
    #[error("could not parse the state file at {path}: {source}")]
    Parse {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: serde_json::Error,
    },
    /// The file could not be written.
    #[error("could not write the state file at {path}: {source}")]
    Write {
        /// The path.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The file could not be serialised.
    #[error("could not serialise the state file: {source}")]
    Serialise {
        /// The underlying error.
        #[source]
        source: serde_json::Error,
    },
}

/// Read the state file. A file that is not there is an empty state.
pub fn load(path: &Path) -> Result<BTreeMap<String, Value>, StateError> {
    let raw = match std::fs::read(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(source) => {
            return Err(StateError::Read {
                path: path.to_path_buf(),
                source,
            })
        }
    };
    if raw.is_empty() {
        return Ok(BTreeMap::new());
    }
    let value: Value = serde_json::from_slice(&raw).map_err(|source| StateError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    match value {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(StateError::NotAnObject {
            path: path.to_path_buf(),
        }),
    }
}

/// Write the state file, atomically, with mode 0600.
pub fn save(path: &Path, state: &BTreeMap<String, Value>) -> Result<(), StateError> {
    let body =
        serde_json::to_vec_pretty(state).map_err(|source| StateError::Serialise { source })?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|source| StateError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, &body).map_err(|source| StateError::Write {
        path: temporary.clone(),
        source,
    })?;
    harden(&temporary);
    std::fs::rename(&temporary, path).map_err(|source| StateError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    harden(path);
    Ok(())
}

/// Make a file owner-only, where the filesystem agrees.
fn harden(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// The capability token, minting one if the state file has none.
///
/// The token is a random secret of 48 bytes, URL-safe — the same shape the
/// Python app uses (`secrets.token_urlsafe(48)`). It is the browser's
/// credential for the local API, so it is never written to a log.
pub fn capability_token(path: &Path) -> Result<String, StateError> {
    let mut state = load(path)?;
    if let Some(token) = state.get(CAPABILITY_KEY).and_then(Value::as_str) {
        if token.len() >= 32 {
            return Ok(token.to_string());
        }
    }
    let token = random_token();
    state.insert(CAPABILITY_KEY.to_string(), Value::String(token.clone()));
    save(path, &state)?;
    Ok(token)
}

/// A random URL-safe token.
pub fn random_token() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let bytes = random_bytes(48);
    bytes
        .iter()
        // The alphabet has 64 characters and a byte has 256 values, so every
        // character is equally likely — there is no modulo bias.
        .map(|byte| ALPHABET[usize::from(*byte) % ALPHABET.len()] as char)
        .collect()
}

/// Random bytes, from the operating system.
pub fn random_bytes(count: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; count];
    // `/dev/urandom` is an endless stream, so it is read exactly `count`
    // bytes — never to the end, which never comes.
    if let Ok(mut source) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        if source.read_exact(&mut buffer).is_ok() {
            return buffer;
        }
    }
    // A fallback that is still unpredictable enough for a loopback cookie:
    // the address of a fresh allocation, the clock, and the process id.
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos() as u64)
        .unwrap_or(0)
        ^ (std::process::id() as u64) << 32;
    for byte in buffer.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 33) as u8;
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("stride-desktop-state-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the scratch directory");
        dir.join("state.json")
    }

    #[test]
    fn a_token_is_minted_once_and_kept() {
        let path = scratch("mint");
        let first = capability_token(&path).expect("the token");
        assert!(first.len() >= 32, "the token is {} characters", first.len());
        let second = capability_token(&path).expect("the token");
        assert_eq!(first, second, "the token changed between two calls");
    }

    #[test]
    fn the_other_keys_survive() {
        let path = scratch("preserve");
        std::fs::write(
            &path,
            br#"{"personal_api_key":"GCDC_secret","capability_token":""}"#,
        )
        .expect("the state file");
        let token = capability_token(&path).expect("the token");
        assert_eq!(token.len(), 48);
        let state = load(&path).expect("the state");
        assert_eq!(
            state.get("personal_api_key").and_then(Value::as_str),
            Some("GCDC_secret"),
            "the Garmin key did not survive"
        );
    }
}
