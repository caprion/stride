//! `stride-desktop` — the desktop application.
//!
//! One binary that serves the React app and the API out of the Rust core:
//!
//! ```text
//! cargo run --release -p stride-desktop
//! ```
//!
//! It serves the SPA from `frontend/dist` and the API beside it, on the
//! loopback interface, and it opens the athlete's own database — the one
//! the Python app wrote — with SQLCipher.
//!
//! ## The shape of it
//!
//! * The API is `stride-api`'s router, the same one the phone calls through
//!   the FFI. One router, two hosts.
//! * The SPA is the built React app: the files in `frontend/dist`, the same
//!   ones the Python server serves.
//! * The database is `backend/data/stride.db`, opened through
//!   `stride-store`, with the key from `STRIDE_DB_KEY`, from Azure Key
//!   Vault, or from a local key file.
//! * The capability cookie is minted for the loopback browser, exactly as
//!   the Python server does it.
//!
//! ## The modules
//!
//! | module | what it does |
//! |---|---|
//! | [`config`] | the environment: the bind address, the paths, the key |
//! | [`db`] | the database: the format, the conversion, the key |
//! | [`http`] | a small HTTP/1.1 server, written in-crate |
//! | [`security`] | the capability gate, the cookies, the rate limits |
//! | [`server`] | the routing: the API, the SPA, the headers |
//! | [`spa`] | the static files, and the traversal hole that is not there |
//! | [`state`] | the state file and the capability token |

pub mod config;
pub mod db;
pub mod http;
pub mod security;
pub mod server;
pub mod spa;
pub mod state;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use stride_api::{Api, CoreBackend};
use stride_store::Store;

use crate::config::{Config, KeySource};
use crate::security::Security;
use crate::server::App;

/// What was built.
pub struct Runtime {
    /// The application.
    pub app: App,
    /// The store the router reads.
    pub store: Arc<Store>,
    /// Where the database key came from.
    pub key_source: KeySource,
    /// What the conversion did, if the database was the plain one.
    pub conversion: Option<db::ConversionReport>,
}

/// The origins that may call the API from a page.
///
/// The reference reads `STRIDE_CORS_ORIGINS` and defaults to the development
/// server's origin. A `*` is refused: the API carries health data.
pub fn allowed_origins() -> Vec<String> {
    let raw = std::env::var("STRIDE_CORS_ORIGINS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://localhost:5173".to_string());
    raw.split(',')
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty() && value != "*")
        .collect()
}

/// Build the application: the database, the router and the gate.
pub fn build(config: &Config) -> anyhow::Result<Runtime> {
    let (key, key_source) =
        db::resolve_key(config.db_key.clone(), &config.data_dir).context("the database key")?;

    let (store, conversion) = db::open_database(&config.db_path, &key)
        .with_context(|| format!("the database at {}", config.db_path.display()))?;
    let store = Arc::new(store);

    let backend = stride_api::store::backend(Arc::clone(&store));
    let api = Api::new(Arc::new(CoreBackend::new(backend)));

    let token = state::capability_token(&config.state_path)
        .with_context(|| format!("the state file at {}", config.state_path.display()))?;
    let security = Security::new(token, config.auth_enabled, config.cookie_secure);

    let app = App::new(api, security, config.dist.clone(), allowed_origins());

    Ok(Runtime {
        app,
        store,
        key_source,
        conversion,
    })
}

/// The directory the built app lives in, for the log.
pub fn describe(config: &Config) -> String {
    let mut parts = vec![format!(
        "http://{}:{}",
        config.bind.ip(),
        config.bind.port()
    )];
    parts.push(format!("spa={}", config.dist.display()));
    parts.push(format!("db={}", config.db_path.display()));
    parts
        .iter()
        .map(|part| part.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The path of the built app, for the log.
pub fn dist_path(config: &Config) -> PathBuf {
    config.dist.clone()
}
