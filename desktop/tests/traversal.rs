//! The traversal guard.
//!
//! `GET /../../etc/passwd` must not serve a file. Three rules keep that
//! true, and each of them is checked here:
//!
//! * the request path is refused before anything is looked up;
//! * a file served from a directory on disk has to be inside that
//!   directory, after the path has been canonicalised;
//! * and a symbolic link out of that directory is caught by the second
//!   rule, because the path is compared after the link has been followed.

mod common;

use std::path::PathBuf;

use stride_desktop::error::Result;
use stride_desktop::server::Server;
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;

use common::{body_of, loopback, request};

/// A server that also serves a directory from disk.
fn server_with(directory: PathBuf) -> Server {
    Server::new(
        Trust::loopback_only(),
        std::sync::Arc::new(common::FakeRouter),
        Spa::new(Some(directory)),
    )
}

/// The text of a file that must never be served.
const PASSWD: &str = "root:x:0:0:root:/root:/bin/bash";

#[test]
fn a_path_that_walks_upwards_serves_no_file() {
    let server = server_with(PathBuf::from("/tmp"));
    for path in [
        "/../../etc/passwd",
        "/..%2f..%2fetc%2fpasswd",
        "/a/../../../etc/passwd",
        "/%2e%2e/%2e%2e/etc/passwd",
    ] {
        let response = server.handle(&request("GET", path), loopback());
        let body = body_of(&response);
        assert!(
            !body.contains("root:"),
            "{path} served a file it should not have: {body}"
        );
        assert_ne!(response.status, 200, "{path} was answered with a file");
    }
}

#[test]
fn a_path_that_walks_upwards_is_refused_outright() {
    let server = server_with(PathBuf::from("/tmp"));
    let response = server.handle(&request("GET", "/../../etc/passwd"), loopback());
    assert_eq!(response.status, 400);
    let body = body_of(&response);
    assert!(!body.contains(PASSWD), "{body}");
}

#[test]
fn a_backslash_is_not_a_separator() {
    let server = server_with(PathBuf::from("/tmp"));
    let response = server.handle(&request("GET", "/..\\..\\etc\\passwd"), loopback());
    assert_ne!(response.status, 200);
    assert!(!body_of(&response).contains("root:"));
}

#[test]
fn a_file_outside_the_directory_is_not_served() -> Result<()> {
    // A directory with one file in it.
    let root = std::env::temp_dir().join("stride-traversal-test");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("inside.txt"), "inside the directory")?;

    let server = server_with(root.clone());
    let response = server.handle(&request("GET", "/inside.txt"), loopback());
    assert_eq!(response.status, 200);
    assert!(body_of(&response).contains("inside the directory"));

    // A symbolic link out of the directory: the file it points at is not in
    // the directory, and the rig compares the path after the link has been
    // followed.
    let link = root.join("escape.txt");
    let _ = std::fs::remove_file(&link);
    #[cfg(unix)]
    {
        let _ = std::os::unix::fs::symlink("/etc/passwd", &link);
        let response = server.handle(&request("GET", "/escape.txt"), loopback());
        let body = body_of(&response);
        assert!(
            !body.contains("root:"),
            "a link out of the directory served a file: {body}"
        );
        let _ = std::fs::remove_file(&link);
    }

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_path_that_is_not_a_path_is_refused() {
    let server = server_with(PathBuf::from("/tmp"));
    let response = server.handle(&request("GET", "etc/passwd"), loopback());
    assert_ne!(response.status, 200);
}

#[test]
fn the_guard_is_the_rule_the_spa_states() {
    // The guard is a function of its own, and it is what the server uses.
    assert!(!stride_desktop::spa::is_safe_path("/../../etc/passwd"));
    assert!(stride_desktop::spa::is_safe_path("/assets/index.js"));
    assert!(!stride_desktop::spa::is_safe_path("/a\0b"));
}
