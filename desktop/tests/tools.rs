//! Export, import, and the FIT drop target.

mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use stride_desktop::api::{Request, Response, Router};
use stride_desktop::dbkey::DatabaseSecret;
use stride_desktop::key::KeyStore;
use stride_desktop::server::Server;
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;
use stride_desktop::workspace::Workspace;
use stride_store::Store;

use common::{body_of, loopback, request_with_body};

fn scratch(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("stride-tools-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn workspace(directory: &std::path::Path) -> Result<Arc<Workspace>, Box<dyn std::error::Error>> {
    let database_key = Arc::new(DatabaseSecret::generate()?);
    let store = Arc::new(Store::open_and_migrate(
        &directory.join("stride.db"),
        &database_key.store_key()?,
    )?);
    let keys = Arc::new(KeyStore::with_persist_path(directory.join("garmin.key")));
    Ok(Arc::new(Workspace::new(
        store,
        database_key,
        keys,
        directory.to_path_buf(),
    )))
}

#[cfg(unix)]
fn mode_of(path: &std::path::Path) -> Result<u32, Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::metadata(path)?.permissions().mode() & 0o777)
}

#[test]
fn an_export_says_where_it_is_and_imports_back() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("export")?;
    let workspace = workspace(&directory)?;
    let rig = Server::with_workspace(
        Trust::loopback_only(),
        Arc::new(common::FakeRouter),
        Spa::new(None),
        Arc::clone(&workspace),
    );

    let response = rig.handle(&request_with_body("POST", "/rig/export", "{}"), loopback());
    assert_eq!(response.status, 200);
    let body: serde_json::Value = serde_json::from_str(&body_of(&response))?;
    let path = PathBuf::from(
        body.get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or("the export answer had no path")?,
    );
    assert!(path.join("manifest.json").is_file());
    assert!(path.join("stride.db").is_file());
    assert!(path.join("db.key").is_file());
    assert_eq!(
        body.get("format_version")
            .and_then(serde_json::Value::as_i64),
        Some(1)
    );

    #[cfg(unix)]
    {
        assert_eq!(mode_of(&path)?, 0o700);
        assert_eq!(mode_of(&path.join("manifest.json"))?, 0o600);
        assert_eq!(mode_of(&path.join("stride.db"))?, 0o600);
        assert_eq!(mode_of(&path.join("db.key"))?, 0o600);
    }

    let import_body = serde_json::json!({"path": path.to_string_lossy()}).to_string();
    let imported = rig.handle(
        &request_with_body("POST", "/rig/import", &import_body),
        loopback(),
    );
    assert_eq!(imported.status, 200);
    let answer: serde_json::Value = serde_json::from_str(&body_of(&imported))?;
    assert_eq!(
        answer
            .get("format_version")
            .and_then(serde_json::Value::as_i64),
        Some(1)
    );
    assert_eq!(
        answer
            .get("refused")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.len()),
        Some(0)
    );

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn a_refused_import_is_a_sentence_and_not_a_stack_trace() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = scratch("refused")?;
    let workspace = workspace(&directory)?;
    let rig = Server::with_workspace(
        Trust::loopback_only(),
        Arc::new(common::FakeRouter),
        Spa::new(None),
        workspace,
    );
    let body = serde_json::json!({"path": directory.join("not-an-export")}).to_string();
    let response = rig.handle(&request_with_body("POST", "/rig/import", &body), loopback());
    assert_eq!(response.status, 400);
    let answer = body_of(&response);
    assert!(answer.contains("import_refused"), "{answer}");
    assert!(!answer.contains("panicked at"), "{answer}");
    assert!(!answer.contains("RUST_BACKTRACE"), "{answer}");

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

/// A router that records the route it was asked for and answers a FIT
/// import without decoding anything itself.
#[derive(Debug, Default)]
struct FitRouter {
    calls: Mutex<Vec<String>>,
}

impl Router for FitRouter {
    fn call(&self, request: &Request<'_>) -> Response {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(request.path.to_string());
        }
        match request.path {
            "/api/v1/auth/garmin/fit-import" => {
                Response::json(200, "{\"imported\":true,\"activity_id\":7}")
            }
            "/api/v1/activities/7" => Response::json(
                200,
                "{\"local_date\":\"2026-10-12\",\"distance\":{\"text\":\"10.2 km\",\"raw\":10200}}",
            ),
            _ => Response::json(404, "{\"code\":\"not_found\",\"detail\":\"no route\"}"),
        }
    }
}

#[test]
fn a_fit_import_goes_through_the_api_route_and_reports_a_duplicate(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("fit")?;
    let workspace = workspace(&directory)?;
    let router = Arc::new(FitRouter::default());
    let rig = Server::with_workspace(
        Trust::loopback_only(),
        Arc::clone(&router) as Arc<dyn Router>,
        Spa::new(None),
        workspace,
    );

    let response = rig.handle(
        &request_with_body("POST", "/rig/fit-import", "not a real fit file"),
        loopback(),
    );
    assert_eq!(response.status, 200);
    let answer: serde_json::Value = serde_json::from_str(&body_of(&response))?;
    assert_eq!(
        answer.get("duplicate").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        answer.get("date").and_then(serde_json::Value::as_str),
        Some("2026-10-12")
    );
    assert_eq!(
        answer.get("distance").and_then(serde_json::Value::as_str),
        Some("10.2 km")
    );

    let calls = router
        .calls
        .lock()
        .map(|calls| calls.clone())
        .unwrap_or_default();
    assert!(
        calls.contains(&"/api/v1/auth/garmin/fit-import".to_string()),
        "the rig did not call the API route: {calls:?}"
    );

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}
