//! The configuration of the desktop.
//!
//! Everything comes from the environment, and the defaults are the paths the
//! repository uses. The desktop is an app on the operator's own machine, so
//! there is one configuration file and there is none: `STRIDE_PORT`,
//! `STRIDE_DATA_DIR`, `STRIDE_DB_KEY` and a handful of others.
//!
//! ## Loopback only
//!
//! The desktop binds the loopback interface and nothing else. The browser
//! that talks to it is on the same machine, and the phone has its own
//! database and its own process. A non-loopback bind is refused, rather
//! than warned about: the reference server has a tailnet listener, and this
//! binary has no business being one.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

/// The root of the repository, as the build saw it.
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// The configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// The address to bind.
    pub bind: SocketAddr,
    /// The directory that holds the database and the state file.
    pub data_dir: PathBuf,
    /// The database.
    pub db_path: PathBuf,
    /// The state file.
    pub state_path: PathBuf,
    /// The built SPA.
    pub dist: PathBuf,
    /// The key the database is encrypted with, if one was found.
    pub db_key: Option<Vec<u8>>,
    /// Where the key came from.
    pub db_key_source: KeySource,
    /// Whether the capability gate is closed.
    pub auth_enabled: bool,
    /// Whether the capability cookie is marked `Secure`.
    pub cookie_secure: bool,
}

/// Where the database key came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// The environment.
    Environment,
    /// Azure Key Vault.
    KeyVault,
    /// A local key file, written because there was nothing else.
    LocalFile,
    /// There is no key.
    None,
}

impl KeySource {
    /// A human-readable name, for the log.
    pub fn name(&self) -> &'static str {
        match self {
            KeySource::Environment => "the environment",
            KeySource::KeyVault => "Key Vault",
            KeySource::LocalFile => "a local key file",
            KeySource::None => "nowhere",
        }
    }
}

/// What can go wrong with the configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A value in the environment is not what it should be.
    #[error("{name} is not valid: {reason}")]
    Invalid {
        /// The variable.
        name: &'static str,
        /// Why.
        reason: String,
    },
    /// The desktop was asked to bind an address that is not loopback.
    #[error(
        "the desktop binds the loopback interface only: {address} is not a loopback address. \
         The phone has its own process and its own database."
    )]
    NotLoopback {
        /// The address that was asked for.
        address: String,
    },
}

/// Is a name set to a true value?
pub fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Read the configuration from the environment.
pub fn from_env() -> Result<Config, ConfigError> {
    let root = repository_root();
    let bind = std::env::var("STRIDE_BIND")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let address = bind_address(&bind)?;
    let port: u16 = match std::env::var("STRIDE_PORT") {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim().parse().map_err(|error| ConfigError::Invalid {
                name: "STRIDE_PORT",
                reason: format!("{error}"),
            })?
        }
        _ => 8000,
    };

    let data_dir = std::env::var("STRIDE_DATA_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("backend").join("data"));
    let db_path = std::env::var("STRIDE_DB_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("stride.db"));
    let state_path = std::env::var("STRIDE_STATE_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("state.json"));
    let dist = std::env::var("STRIDE_FRONTEND_DIST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("frontend").join("dist"));

    let (db_key, db_key_source) = match std::env::var("STRIDE_DB_KEY") {
        Ok(raw) if !raw.trim().is_empty() => {
            (Some(raw.trim().as_bytes().to_vec()), KeySource::Environment)
        }
        _ => (None, KeySource::None),
    };

    Ok(Config {
        bind: SocketAddr::new(address, port),
        data_dir,
        db_path,
        state_path,
        dist,
        db_key,
        db_key_source,
        auth_enabled: !env_truthy("STRIDE_DISABLE_AUTH"),
        cookie_secure: env_truthy("STRIDE_COOKIE_SECURE"),
    })
}

/// The address to bind, and the check that it is loopback.
///
/// The desktop binds the loopback interface and nothing else: the browser
/// that talks to it is on this machine, and the phone has its own process
/// and its own database. A name that is not an address is a mistake and a
/// message; an address that is not loopback is refused.
pub fn bind_address(raw: &str) -> Result<IpAddr, ConfigError> {
    let value = raw.trim().trim_start_matches('[').trim_end_matches(']');
    let address: IpAddr = value.parse().map_err(|error| ConfigError::Invalid {
        name: "STRIDE_BIND",
        reason: format!("{error}"),
    })?;
    if !is_loopback(&address) {
        return Err(ConfigError::NotLoopback {
            address: value.to_string(),
        });
    }
    Ok(address)
}

/// Is this address on the loopback interface?
pub fn is_loopback(address: &IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn a_loopback_address_is_accepted() {
        assert!(is_loopback(
            &"127.0.0.1"
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
        ));
        assert!(is_loopback(
            &"::1".parse().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
        ));
        assert!(!is_loopback(
            &"0.0.0.0".parse().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
        ));
    }

    #[test]
    fn the_desktop_binds_the_loopback_interface_and_nothing_else() {
        assert!(bind_address("127.0.0.1").is_ok());
        assert!(bind_address(" 127.0.0.1 ").is_ok());
        assert!(bind_address("::1").is_ok());
        assert!(
            bind_address("localhost").is_err(),
            "a name is not an address"
        );
        assert!(
            bind_address("0.0.0.0").is_err(),
            "every interface is not loopback"
        );
        assert!(
            bind_address("100.89.222.50").is_err(),
            "the tailnet is not loopback"
        );
        assert!(
            bind_address("::").is_err(),
            "every interface is not loopback"
        );
    }
}
