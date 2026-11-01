//! The router has no side effects of its own.
//!
//! A route that fails leaves the store exactly as it was. The router is a
//! reader: it opens no connection of its own, it writes no SQL, and the
//! only routes that write anything write it through the store — one
//! transaction, all of it or none of it.
//!
//! This test takes the store's own counts before and after a set of
//! failing calls and holds them to the same numbers.

mod common;

use stride_store::Store;

/// The counts of what is in the store.
fn counts(store: &Store) -> (i64, i64, i64, i64) {
    let activities = store.count_activities().unwrap_or(-1);
    let wellness = store.count_wellness_daily().unwrap_or(-1);
    let sessions = store.count_plan_sessions().unwrap_or(-1);
    let notes = store.count_notes().unwrap_or(-1);
    (activities, wellness, sessions, notes)
}

#[test]
fn a_failing_route_leaves_the_store_as_it_was() {
    let harness = common::Harness::seeded("purity");
    let before = counts(&harness.store);

    // A run that is not there.
    let answer = harness.router.call("GET", "/activities/9999", None);
    assert!(!answer.is_ok());

    // A cursor that is not a cursor.
    let answer = harness.router.call("GET", "/activities?cursor=not-a-cursor", None);
    assert!(!answer.is_ok());

    // A limit that is not a number.
    let answer = harness.router.call("GET", "/activities?limit=many", None);
    assert!(!answer.is_ok());

    // A body that is not JSON.
    let answer = harness.router.call("POST", "/plan/import", Some("{not json"));
    assert!(!answer.is_ok());

    // An import with no plan in it.
    let answer = harness.router.call("POST", "/plan/import", Some("{}"));
    assert!(!answer.is_ok());

    // A key that is not there.
    let answer = harness.router.call("POST", "/auth/garmin/personal-key", Some("{}"));
    assert!(!answer.is_ok());

    // A path that is not a route.
    let answer = harness.router.call("GET", "/nothing/here", None);
    assert!(!answer.is_ok());

    // The wrong method for a route.
    let answer = harness.router.call("POST", "/activities", Some("{}"));
    assert!(!answer.is_ok());

    let after = counts(&harness.store);
    assert_eq!(before, after, "a failing route changed the store");
}

#[test]
fn a_reading_route_leaves_the_store_as_it_was() {
    let harness = common::Harness::seeded("purity-read");
    let before = counts(&harness.store);
    for path in [
        "/activities",
        "/physiology/daily",
        "/physiology/sleep",
        "/physiology/hrv",
        "/physiology/vitals",
        "/physiology/latest",
        "/physiology/sources",
        "/fitness/fingerprint",
        "/fitness/trends",
        "/fitness/zones",
        "/stats/summary",
        "/stats/weekly",
        "/stats/pace-trends",
        "/stats/elevation",
        "/stats/personal-records",
        "/plan/current",
        "/plan/today",
        "/plan/week",
        "/plan/blocks",
        "/plan/sources",
        "/plan/adherence",
        "/plan/results",
        "/insights/today",
        "/insights/progress",
        "/compare/candidates",
        "/health",
        "/health/daily",
        "/sync/status",
        "/sync/runs",
        "/sync/history",
        "/sync/settings",
        "/auth/status",
        "/calendar",
    ] {
        let answer = harness.router.call("GET", path, None);
        assert!(answer.is_ok(), "the route {path} did not answer");
    }
    let after = counts(&harness.store);
    assert_eq!(before, after, "a reading route changed the store");
}

#[test]
fn the_same_read_twice_answers_the_same_way() {
    let harness = common::Harness::seeded("purity-twice");
    for path in ["/activities", "/physiology/vitals", "/insights/today", "/stats/summary"] {
        let first = harness.router.call("GET", path, None);
        let second = harness.router.call("GET", path, None);
        assert_eq!(first.body, second.body, "the route {path} answered differently");
    }
}
