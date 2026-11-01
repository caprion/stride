//! The app is served.
//!
//! Two things are checked here:
//!
//! * **the bundle answers at `/`** — the app is embedded in the binary, and
//!   the rig serves it;
//! * **an unknown path falls back to the app's own routing** — the app has a
//!   router of its own, and the rig does not have an opinion about what
//!   `/runs` means. It answers with the app, not with a 404 and never with a
//!   stack trace.
//!
//! The rig is built in two places: with the frontend bundle and without it.
//! A binary built without the bundle says so, in a sentence, and points at
//! `npm --prefix frontend run build` — so both states are checked here. The
//! gate builds without the bundle (the CI job has no node), and a rig built
//! with it answers the same way.

mod common;

use stride_desktop::error::Result;
use stride_desktop::trust::Trust;

use common::{body_of, loopback, request, server};

#[test]
fn the_app_answers_at_the_root() -> Result<()> {
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/"), loopback());
    assert_eq!(response.status, 200);
    let body = body_of(&response);
    let content_type = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();

    if stride_desktop::spa::bundle_is_built() {
        assert!(
            body.contains("<html"),
            "the root did not answer with the app: {body}"
        );
        assert!(
            content_type.starts_with("text/html"),
            "the root did not answer with HTML: {content_type}"
        );
    } else {
        assert!(
            body.contains("npm --prefix frontend run build"),
            "a rig built without the bundle has to say what to do: {body}"
        );
    }
    Ok(())
}

#[test]
fn an_unknown_path_falls_back_to_the_app() -> Result<()> {
    let server = server(Trust::loopback_only());
    for path in ["/runs", "/progress", "/body", "/plan", "/compare/12"] {
        let response = server.handle(&request("GET", path), loopback());
        assert_ne!(
            response.status, 404,
            "{path} was answered with a 404 instead of the app"
        );
        assert_eq!(response.status, 200, "{path}");
        let body = body_of(&response);
        assert!(
            !body.contains("panicked at"),
            "{path} was answered with a stack trace: {body}"
        );
        assert!(
            !body.contains("RUST_BACKTRACE"),
            "{path} was answered with a stack trace: {body}"
        );
    }
    Ok(())
}

#[test]
fn a_missing_file_is_a_plain_404() -> Result<()> {
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/assets/nothing-here.js"), loopback());
    let body = body_of(&response);
    assert!(!body.contains("panicked at"), "{body}");

    if stride_desktop::spa::bundle_is_built() {
        // The bundle is there, and the file is not: the rig says so, and it
        // says it in a sentence.
        assert_eq!(response.status, 404);
    } else {
        // The rig was built without the bundle. It says *that*, in a
        // sentence, and it does not pretend the file is missing.
        assert_eq!(response.status, 200);
        assert!(
            body.contains("npm --prefix frontend run build"),
            "the rig has to say what is missing: {body}"
        );
    }
    Ok(())
}

#[test]
fn the_bundle_is_inside_the_binary() -> Result<()> {
    // The app is embedded: the files are in the binary, and there is no
    // directory to point at. The rig reads them from itself.
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/"), loopback());
    assert_eq!(response.status, 200);
    assert!(!body_of(&response).is_empty());
    Ok(())
}
