//! The export and the import of the database.
//!
//! The store has both: an **export** is a directory with the database and a
//! manifest, and an **import** reads one back. The rig needs to reach them,
//! so the router has the two routes.
//!
//! ## The rules the store keeps, and the router does not touch
//!
//! * **the path is confined** — the manifest names the database, and the
//!   name must be a plain file name that resolves **inside** the export
//!   directory. A manifest that says "../../.ssh/id_rsa" does not make the
//!   store open it.
//! * **the digest is verified** — the size and the SHA-256 are in the
//!   manifest, and they are checked against the file before anything opens
//!   it.
//! * **the schema version is checked** — an export made by another version
//!   is refused, not guessed at.
//!
//! The router adds one rule of its own, and it is the same one: **the
//! export the caller names lives inside the export directory the host
//! chose.** A name is a plain name — no directory part, no `..`, no
//! separator — and the path it resolves to has to be a child of the export
//! directory. There is no way around the store's rule, and no way around
//! this one either.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::dto::{ExportWritten, ImportRead};
use crate::error::{ApiError, ApiResult};
use crate::router::Request;
use crate::state::Api;

use super::ok;

/// Write an export of the database.
///
/// The export goes into the host's own export directory, in a directory of
/// its own: the athlete moves that directory to the phone. The answer says
/// what was written — how many rows, and the digest the import will check.
pub fn export(api: &Api, request: &Request) -> ApiResult<Value> {
    let root = export_root(api)?;
    let name = export_name(api, request)?;
    let directory = plain_name(&name)
        .map_err(|_| ApiError::bad_request("the name of the export is not a plain name"))?;
    let path = root.join(directory);

    // The directory is made, and then the path is resolved: a name that is
    // a plain name is still a name that could be a link out of the export
    // directory, and the resolved path is what has to be a child of it.
    std::fs::create_dir_all(&path)
        .map_err(|_| ApiError::internal("the export directory could not be made"))?;
    let resolved = std::fs::canonicalize(&path)
        .map_err(|_| ApiError::internal("the export directory could not be made"))?;
    let resolved_root = std::fs::canonicalize(&root)
        .map_err(|_| ApiError::internal("the export directory could not be read"))?;
    if resolved.parent() != Some(resolved_root.as_path()) {
        return Err(ApiError::bad_request(
            "the export is not inside the export directory",
        ));
    }

    let manifest = api.with_export_key(|key| {
        stride_store::export::export(api.store(), &resolved, key)
            .map_err(|error| export_error(api, &error))
    })?;

    let response = ExportWritten {
        directory: directory.to_string(),
        rows: manifest.rows,
        tables: manifest.tables,
        schema_version: manifest.schema_version,
        database_bytes: manifest.database_bytes,
        database_sha256: manifest.database_sha256,
        created_at: crate::state::format_instant(manifest.created_at),
        generated_at: api.generated_at(),
    };
    ok(&response)
}

/// Read an export back.
///
/// The body names the export directory. The name is a plain name inside the
/// export directory the host chose, and the store checks the manifest, the
/// schema version and the digest before it opens anything.
///
/// The import is a **merge**: a run that is already there is updated, and a
/// run that is not is added. Importing twice adds nothing twice.
pub fn import(api: &Api, request: &Request) -> ApiResult<Value> {
    let root = export_root(api)?;
    let name = request
        .body_str("directory")
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| ApiError::bad_request("the import names no export"))?;
    let directory = confined(&root, &name)?;

    let report = api.with_export_key(|key| {
        stride_store::export::import(api.store(), &directory, key)
            .map_err(|error| export_error(api, &error))
    })?;

    let response = ImportRead {
        activities: report.activities,
        wellness: report.wellness,
        plan_blocks: report.plan_blocks,
        plan_sessions: report.plan_sessions,
        notes: report.notes,
        rows: report.rows,
        generated_at: api.generated_at(),
    };
    ok(&response)
}

// ------------------------------------------------------------------ helpers

/// The export directory the host chose.
fn export_root(api: &Api) -> ApiResult<PathBuf> {
    match api.config().export_root.clone() {
        Some(root) => Ok(root),
        None => Err(ApiError::bad_request(
            "this device has no directory to write an export into",
        )),
    }
}

/// The name of the export: the caller's, or one from the clock.
fn export_name(api: &Api, request: &Request) -> ApiResult<String> {
    let given = request
        .body_str("directory")
        .or_else(|| request.body_str("name"))
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    match given {
        Some(name) => Ok(name),
        None => Ok(format!("export-{}", api.now().format("%Y%m%d-%H%M%S"))),
    }
}

/// A name that is a plain name: no directory part, no `..`, no separator.
fn plain_name(name: &str) -> ApiResult<&str> {
    let named = Path::new(name);
    let plain = named
        .file_name()
        .map(|file_name| file_name == named.as_os_str())
        .unwrap_or(false);
    if !plain || name.trim().is_empty() {
        return Err(ApiError::bad_request(
            "the name of the export is not a plain name",
        ));
    }
    Ok(name)
}

/// The export the caller names, resolved inside the export directory.
///
/// The name has to be a plain name, and the path it resolves to has to be a
/// child of the export directory: the path is resolved before it is
/// compared, so a link out of the directory is caught here.
fn confined(root: &Path, name: &str) -> ApiResult<PathBuf> {
    plain_name(name)?;
    let resolved_root = std::fs::canonicalize(root)
        .map_err(|_| ApiError::internal("the export directory could not be read"))?;
    let path = resolved_root.join(name);
    let resolved = std::fs::canonicalize(&path)
        .map_err(|_| ApiError::not_found("there is no export with that name"))?;
    if resolved.parent() != Some(resolved_root.as_path()) {
        return Err(ApiError::bad_request(
            "the export is not inside the export directory",
        ));
    }
    Ok(resolved)
}

/// A failure of the export, as the router reports it.
///
/// The store's own message carries the path it was about, and a path is
/// not something an answer carries. The reasons that say *what* was wrong
/// with the file are kept — they are the athlete's own file, and they are
/// the difference between "it is not an export" and "it is not this
/// version's export".
fn export_error(api: &Api, error: &stride_store::StoreError) -> ApiError {
    use stride_store::StoreError;
    match error {
        StoreError::InvalidKey { .. } => ApiError::locked("the database key is not available"),
        StoreError::SchemaVersion { .. } => {
            ApiError::bad_request("the export was made by another version of the app")
        }
        StoreError::Export { reason, .. } => {
            ApiError::bad_request(format!("the export could not be read: {reason}"))
        }
        _ => api.store_error("the export could not be read", error),
    }
}
