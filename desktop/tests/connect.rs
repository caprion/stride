//! The first-run page.
//!
//! It stands alone, it remembers the choice, and it never carries the key
//! anywhere but the one field.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use stride_desktop::dbkey::DatabaseSecret;
use stride_desktop::key::KeyStore;
use stride_desktop::server::Server;
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;
use stride_desktop::workspace::Workspace;
use stride_store::Store;

use common::{body_of, loopback, request, request_with_body};

fn scratch(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("stride-connect-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn make_workspace(
    directory: &std::path::Path,
) -> Result<(Arc<Workspace>, Arc<KeyStore>), Box<dyn std::error::Error>> {
    let database_key = Arc::new(DatabaseSecret::load_or_create(&directory.join("db.key"))?);
    let store = Arc::new(Store::open_and_migrate(
        &directory.join("stride.db"),
        &database_key.store_key()?,
    )?);
    let keys = Arc::new(KeyStore::with_persist_path(directory.join("garmin.key")));
    let workspace = Arc::new(Workspace::new(
        store,
        database_key,
        Arc::clone(&keys),
        directory.to_path_buf(),
    ));
    Ok((workspace, keys))
}

fn server(workspace: Arc<Workspace>) -> Server {
    Server::with_workspace(
        Trust::loopback_only(),
        Arc::new(common::FakeRouter),
        Spa::new(None),
        workspace,
    )
}

#[test]
fn a_fresh_rig_opens_the_standalone_connect_page() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("fresh")?;
    let (workspace, _keys) = make_workspace(&directory)?;
    let rig = server(workspace);

    let response = rig.handle(&request("GET", "/"), loopback());
    assert_eq!(response.status, 200);
    let body = body_of(&response);
    assert!(body.contains("Your data stays on this machine"), "{body}");
    assert!(body.contains("Garmin Personal API key"), "{body}");
    assert!(body.contains("Test the key"), "{body}");
    assert!(body.contains("Not now"), "{body}");
    assert!(body.contains("Checking…"), "{body}");
    assert!(body.contains("Syncing…"), "{body}");
    assert!(body.contains("Connected ·"), "{body}");
    assert!(
        body.contains("developer.garmin.com/gc-developer-program/personal"),
        "{body}"
    );

    // There is one key field, and it is a password field. No app navigation
    // stands around it.
    assert_eq!(body.matches("type=\"password\"").count(), 1, "{body}");
    assert!(!body.contains("<nav"), "{body}");
    assert!(!body.contains("data-nav=\"rail\""), "{body}");

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn not_now_opens_an_honest_empty_app_and_is_remembered() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("not-now")?;
    let (workspace, _keys) = make_workspace(&directory)?;
    let rig = server(Arc::clone(&workspace));

    let skipped = rig.handle(
        &request_with_body("POST", "/rig/first-run/not-now", "{}"),
        loopback(),
    );
    assert_eq!(skipped.status, 200);

    let root = rig.handle(&request("GET", "/"), loopback());
    assert_eq!(root.status, 200);
    let body = body_of(&root);
    assert!(!body.contains("Your data stays on this machine"), "{body}");

    // The next launch reads the marker and opens the app as well.
    drop(rig);
    drop(workspace);
    let (workspace, _keys) = make_workspace(&directory)?;
    let rig = server(workspace);
    let root = rig.handle(&request("GET", "/"), loopback());
    assert_eq!(root.status, 200);
    let body = body_of(&root);
    assert!(!body.contains("Your data stays on this machine"), "{body}");

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}

#[test]
fn a_connected_rig_has_no_connect_page_left() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("connected")?;
    let (workspace, keys) = make_workspace(&directory)?;
    keys.set("a-short-test-value")?;
    let rig = server(workspace);

    let response = rig.handle(&request("GET", "/connect"), loopback());
    assert_eq!(response.status, 302);
    let location = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    assert_eq!(location, "/");

    let _ = std::fs::remove_dir_all(&directory);
    Ok(())
}
