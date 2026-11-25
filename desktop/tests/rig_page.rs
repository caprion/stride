//! The rig's own page.
//!
//! It is the host's face: the version being served, where the athlete's
//! data is, whether a Garmin key is kept, and the four decisions that
//! only the athlete can make.
//!
//! The tests here hold it to four rules:
//!
//!   - **it says facts** — the version, the schema, the files, the counts;
//!   - **the four decisions are the athlete's** — they are offered side by
//!     side, unranked, and nothing on the page grades or ranks them;
//!   - **a fresh rig is honest** — an empty race, an empty threshold, an
//!     empty programme and an empty pair of heart rates are named, with the
//!     reason, and the page is not a wizard that presumes any of them;
//!   - **the key never reaches the page**, and neither does the database
//!     key.

mod common;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use stride_desktop::api::{ApiRouter, Request, Response, Router};
use stride_desktop::dbkey::DatabaseSecret;
use stride_desktop::key::KeyStore;
use stride_desktop::server::{DesktopHost, Server};
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;
use stride_desktop::workspace::Workspace;
use stride_store::Store;

use common::{body_of, loopback, request, request_with_body};

/// A Garmin key that is recognisable if it ever turns up anywhere.
///
/// It is not credential-shaped on purpose: it is a test fixture, and the
/// repository's secret scanner would (rightly) refuse a string that looked
/// like a real key.
const THE_GARMIN_KEY: &str = "stride-garmin-key-4f2c9a7e1b6d8305-do-not-print";

/// A scratch directory for one test.
fn scratch(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("stride-rig-page-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

/// The workspace, the keys and the router over them.
type Rig = (Arc<Workspace>, Arc<KeyStore>, Arc<dyn Router>);

/// The workspace, the keys and the router over them.
fn rig(directory: &Path) -> Result<Rig, Box<dyn std::error::Error>> {
    let database_key = Arc::new(DatabaseSecret::load_or_create(&directory.join("db.key"))?);
    let store = Arc::new(Store::open_and_migrate(
        &directory.join("stride.db"),
        &database_key.store_key()?,
    )?);
    let keys = Arc::new(KeyStore::with_persist_path(directory.join("garmin.key")));
    let workspace = Arc::new(Workspace::new(
        Arc::clone(&store),
        database_key,
        Arc::clone(&keys),
        directory.to_path_buf(),
    ));
    let host = DesktopHost::new(Arc::clone(&store), Arc::clone(&keys));
    let router: Arc<dyn Router> = Arc::new(ApiRouter::over(&host)?);
    Ok((workspace, keys, router))
}

/// The server, over a workspace and a router.
fn server(workspace: Arc<Workspace>, router: Arc<dyn Router>) -> Server {
    Server::with_workspace(Trust::loopback_only(), router, Spa::new(None), workspace)
}

/// The rig's own page, as text.
fn page_of(response: &stride_desktop::http::Response) -> String {
    body_of(response)
}

#[test]
fn the_page_says_the_version_the_schema_and_where_the_files_are(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("status")?;
    let (workspace, _keys, router) = rig(&directory)?;
    let rig = server(workspace, router);

    let response = rig.handle(&request("GET", "/rig"), loopback());
    assert_eq!(response.status, 200);
    let page = page_of(&response);

    // The version being served.
    assert!(page.contains("0.1.0"), "the page does not name the version");
    // The database's schema version.
    assert!(
        page.contains("schema version"),
        "the page does not name the schema version"
    );
    // Where the files are.
    assert!(
        page.contains(&directory.display().to_string()),
        "the page does not name the data directory"
    );
    assert!(
        page.contains("stride.db"),
        "the page does not name the database"
    );
    // That the data is the athlete's own and local.
    assert!(
        page.contains("Stored on this device"),
        "the page does not say the data is stored on this device"
    );
    Ok(())
}

#[test]
fn the_page_offers_the_four_decisions_and_ranks_none_of_them(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("decisions")?;
    let (workspace, _keys, router) = rig(&directory)?;
    let rig = server(workspace, router);

    let response = rig.handle(&request("GET", "/rig"), loopback());
    let page = page_of(&response);

    // The four decisions, each named.
    assert!(page.contains("A race"), "the page does not offer a race");
    assert!(
        page.contains("A threshold"),
        "the page does not offer a threshold"
    );
    assert!(
        page.contains("A programme"),
        "the page does not offer a programme"
    );
    assert!(
        page.contains("Your heart rates"),
        "the page does not offer the heart rates"
    );

    // Nothing is ranked, recommended or graded. The ban list is the
    // contract's, and the page is a surface like any other.
    for banned in [
        "recommend",
        "should",
        "must",
        "better to",
        "need to",
        "first choice",
        "best",
        "worst",
        "score",
    ] {
        assert!(
            !page.to_lowercase().contains(banned),
            "the page says \"{banned}\", and the page states facts"
        );
    }
    Ok(())
}

#[test]
fn a_fresh_rig_names_the_four_empty_decisions_and_presumes_nothing(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("fresh")?;
    let (workspace, _keys, router) = rig(&directory)?;
    let rig = server(workspace, router);

    let response = rig.handle(&request("GET", "/rig"), loopback());
    let page = page_of(&response);

    // The four empties, each named: no race typed, no threshold chosen, no
    // programme adopted, and no heart rates entered.
    assert!(
        page.contains("No race time entered."),
        "the page does not say that no race has been typed"
    );
    assert!(
        page.contains("No threshold is in force."),
        "the page does not say that no threshold is in force"
    );
    assert!(
        page.contains("No programme has been adopted."),
        "the page does not say that no programme has been adopted"
    );
    assert!(
        page.contains("No heart rates entered."),
        "the page does not say that no heart rates have been entered"
    );

    // The page is not a wizard: the race is offered, and the other three
    // are offered as a link into the app rather than as a step.
    assert!(
        page.contains("action=\"/rig/race\""),
        "the page does not offer the race to be typed"
    );
    assert!(
        page.contains("/settings?section=training"),
        "the page does not say where the threshold is chosen"
    );
    assert!(
        page.contains("/plan/bases"),
        "the page does not say where the programme is adopted"
    );
    Ok(())
}

#[test]
fn the_key_never_reaches_the_page() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("key")?;
    let (workspace, keys, router) = rig(&directory)?;
    keys.set(THE_GARMIN_KEY)?;
    let rig = server(workspace, router);

    let response = rig.handle(&request("GET", "/rig"), loopback());
    let page = page_of(&response);

    assert!(
        !page.contains(THE_GARMIN_KEY),
        "the page carried the Garmin key"
    );
    // The page says that a key is kept, and where the file is. That is
    // the whole of what it says about it.
    assert!(
        page.contains("a key is kept on this machine"),
        "the page does not say that a key is kept"
    );
    Ok(())
}

#[test]
fn a_race_is_written_through_the_router() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("race")?;
    let (workspace, _keys, router) = rig(&directory)?;
    let rig = server(Arc::clone(&workspace), Arc::clone(&router));

    let response = rig.handle(
        &request_with_body("POST", "/rig/race", "label=10K&time=43:10&date=2026-03-03"),
        loopback(),
    );
    // The write went through, and the browser is sent back to the page.
    assert_eq!(response.status, 303);
    let location = response
        .headers
        .iter()
        .find(|(name, _)| name == "location")
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    assert_eq!(location, "/rig");

    // The race is in the log, and the app's own route reads it.
    let rig = server(Arc::clone(&workspace), Arc::clone(&router));
    let races = rig.handle(&request("GET", "/api/v1/athlete/races"), loopback());
    let body = body_of(&races);
    assert!(body.contains("10K"), "the race was not written: {body}");
    assert!(
        body.contains("43:10"),
        "the race's time was not written: {body}"
    );

    // And the rig's own page lists it.
    let page = page_of(&rig.handle(&request("GET", "/rig"), loopback()));
    assert!(
        page.contains("43:10"),
        "the page does not list the race that was written"
    );
    Ok(())
}

#[test]
fn a_race_that_cannot_be_written_is_refused_in_a_sentence() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = scratch("refused")?;
    let (workspace, _keys, router) = rig(&directory)?;
    let rig = server(workspace, router);

    // A label the app does not know.
    let response = rig.handle(
        &request_with_body("POST", "/rig/race", "label=100K&time=43:10&date=2026-03-03"),
        loopback(),
    );
    assert_eq!(response.status, 200);
    let page = page_of(&response);
    assert!(
        page.contains("The race was not added."),
        "the refusal is not a sentence: {page}"
    );
    assert!(
        !page.contains("100K"),
        "the refused race was written anyway"
    );

    // A time that is not a time.
    let response = rig.handle(
        &request_with_body("POST", "/rig/race", "label=10K&time=soon&date=2026-03-03"),
        loopback(),
    );
    let page = page_of(&response);
    assert!(
        page.contains("it needs a time"),
        "the refusal is not a sentence: {page}"
    );
    Ok(())
}

#[test]
fn the_tools_page_is_still_served_and_the_rig_answers_its_own_paths(
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("tools")?;
    let (workspace, _keys, router) = rig(&directory)?;
    let rig = server(workspace, router);

    let tools = rig.handle(&request("GET", "/rig/tools"), loopback());
    assert_eq!(tools.status, 200);
    assert!(
        body_of(&tools).contains("Backup, restore and export"),
        "the tools page is not the tools page"
    );

    // A path of the rig's own that is not there is refused, and it is
    // refused in a sentence.
    let missing = rig.handle(&request("GET", "/rig/nothing"), loopback());
    assert_eq!(missing.status, 404);
    assert!(
        body_of(&missing).contains("There is no rig tool at that path."),
        "the refusal is not a sentence"
    );
    Ok(())
}

/// A router that records the routes it was asked for.
///
/// It is used by the test that holds the rig to writing through the
/// router: the rig must not reach the store on its own.
#[derive(Debug, Clone, Default)]
struct RecordingRouter {
    seen: Arc<Mutex<Vec<String>>>,
}

impl Router for RecordingRouter {
    fn call(&self, request: &Request<'_>) -> Response {
        if let Ok(mut seen) = self.seen.lock() {
            seen.push(request.path.to_string());
        }
        Response::json(200, "{\"ok\":true}")
    }
}

#[test]
fn the_rig_asks_the_router_for_the_facts_it_states() -> Result<(), Box<dyn std::error::Error>> {
    let directory = scratch("facts")?;
    let (workspace, _keys, _router) = rig(&directory)?;
    let recorder = RecordingRouter::default();
    let seen = Arc::clone(&recorder.seen);
    let rig = server(
        Arc::clone(&workspace),
        Arc::new(recorder) as Arc<dyn Router>,
    );

    let _ = rig.handle(&request("GET", "/rig"), loopback());
    let asked = seen.lock().map(|seen| seen.clone()).unwrap_or_default();
    for route in [
        "/health",
        "/athlete/races",
        "/insights/threshold",
        "/plan/adopt",
        "/sync/status",
    ] {
        assert!(
            asked.iter().any(|path| path == route),
            "the rig did not ask the router for {route}: {asked:?}"
        );
    }
    Ok(())
}
