//! The route table is the contract's, route for route.
//!
//! `docs/contract/api-paths.md` is frozen, and this test walks the
//! contract's own list — read out of the file, not copied into the test —
//! and holds the router to it:
//!
//! * every route the contract names answers;
//! * no route outside the contract answers;
//! * the router's table is the contract's list, exactly.
//!
//! The cut routes are checked too: the contract names them as well, and
//! they are the routes that must never come back.

mod common;

use std::collections::BTreeSet;
use std::path::Path;

/// The contract's route table.
fn contract_text() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("contract")
        .join("api-paths.md");
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => panic!("the contract could not be read: {error}"),
    }
}

/// The routes the contract names.
///
/// The table is written in the contract's own shorthand — one row with
/// `{a,b,c}` for a group — so the groups are expanded here, the same way
/// the contract means them.
fn contract_routes() -> BTreeSet<String> {
    let text = contract_text();
    // Everything after the cut list is not a route the contract wants.
    let wanted = match text.find("## Cut from the old table") {
        Some(at) => &text[..at],
        None => text.as_str(),
    };
    let mut routes = BTreeSet::new();
    for line in wanted.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        // A row may name more than one route: "/health · /health/daily".
        for cell in line.split('`') {
            let cell = cell.trim();
            if !cell.starts_with('/') {
                continue;
            }
            for route in expand(cell) {
                routes.insert(route);
            }
        }
    }
    routes
}

/// The routes the contract cut.
fn cut_routes() -> BTreeSet<String> {
    let text = contract_text();
    let mut routes = BTreeSet::new();
    // The cut list ends where the discrepancies begin: the discrepancies
    // are about the routes that were kept, not about routes that were cut.
    let cut = match text.find("## Cut from the old table") {
        Some(at) => &text[at..],
        None => return routes,
    };
    let cut = match cut.find("## Discrepancies") {
        Some(at) => &cut[..at],
        None => cut,
    };
    let mut cutting = false;
    for line in cut.lines() {
        if line.starts_with("## Cut from the old table") {
            cutting = true;
            continue;
        }
        if !cutting {
            continue;
        }
        for part in line.split('`') {
            let part = part.trim();
            if part.starts_with('/') {
                for route in expand(part) {
                    routes.insert(route);
                }
            }
            // The cut list names groups: `/auth/garmin/{login,mfa,…}`.
            if part.starts_with("/auth/garmin/{") {
                let inner = part
                    .trim_start_matches("/auth/garmin/{")
                    .trim_end_matches('}')
                    .trim_end_matches("…");
                for name in inner.split(',') {
                    let name = name.trim();
                    if !name.is_empty() {
                        routes.insert(format!("/auth/garmin/{name}"));
                    }
                }
            }
            if part.starts_with("/auth/{") {
                let inner = part
                    .trim_start_matches("/auth/{")
                    .trim_end_matches('}')
                    .trim_end_matches("…");
                for name in inner.split(',') {
                    let name = name.trim();
                    if !name.is_empty() {
                        routes.insert(format!("/auth/{name}"));
                    }
                }
            }
        }
    }
    routes
}

/// Expand the groups in a route pattern.
fn expand(pattern: &str) -> Vec<String> {
    let open = match pattern.find('{') {
        Some(at) => at,
        None => return vec![pattern.to_string()],
    };
    let close = match pattern[open..].find('}') {
        Some(at) => open + at,
        None => return vec![pattern.to_string()],
    };
    let head = &pattern[..open];
    let body = &pattern[open + 1..close];
    let tail = &pattern[close + 1..];
    let names: Vec<&str> = body.split(',').map(|name| name.trim()).collect();
    // A group of one is a placeholder, not a group: "/activities/{id}" names
    // one route, and the placeholder is part of its name.
    if names.len() == 1 {
        return vec![pattern.to_string()];
    }
    let mut out = Vec::new();
    for name in names {
        if name.is_empty() {
            continue;
        }
        for expanded in expand(&format!("{head}{name}{tail}")) {
            out.push(expanded);
        }
    }
    out
}

/// A path that names a route: the placeholders are filled in.
fn concrete(route: &str) -> String {
    route.replace("{id}", "1")
}

#[test]
fn the_contract_names_the_routes_the_router_has() {
    let contract = contract_routes();
    assert!(
        !contract.is_empty(),
        "the contract's route table could not be read out of the file"
    );

    let router = common::Harness::empty("route-parity");
    let mut missing: Vec<String> = Vec::new();
    for route in contract.iter() {
        if router.router.route_at(&concrete(route)).is_none() {
            missing.push(route.clone());
        }
    }
    assert!(
        missing.is_empty(),
        "the contract names routes the router does not have: {missing:?}"
    );
}

#[test]
fn the_router_has_no_route_the_contract_does_not_name() {
    let contract = contract_routes();
    let mut extra: Vec<String> = Vec::new();
    for route in stride_api::ROUTES.iter() {
        if !contract.contains(route.pattern) {
            extra.push(route.pattern.to_string());
        }
    }
    assert!(
        extra.is_empty(),
        "the router has routes the contract does not name: {extra:?}"
    );
}

#[test]
fn every_route_in_the_contract_answers() {
    let contract = contract_routes();
    let harness = common::Harness::empty("route-answers");
    for route in contract.iter() {
        let path = concrete(route);
        let answer = harness.router.call("GET", &path, None);
        let detail = answer.body["detail"].as_str().unwrap_or_default();
        assert!(
            !detail.starts_with("there is no route at"),
            "the route {path} did not answer: {}",
            answer.body
        );
    }
}

#[test]
fn the_cut_routes_never_come_back() {
    let cut = cut_routes();
    assert!(
        !cut.is_empty(),
        "the contract's cut list could not be read out of the file"
    );
    let harness = common::Harness::empty("cut-routes");
    for route in cut.iter() {
        assert!(
            harness.router.route_at(route).is_none(),
            "the cut route {route} is in the router again"
        );
        let answer = harness.router.call("GET", route, None);
        assert_eq!(answer.status, 404, "the cut route {route} answered");
        assert_eq!(
            answer.body["code"], "not_found",
            "the cut route {route} answered with something else"
        );
    }
}

#[test]
fn a_path_that_is_not_a_route_is_not_answered() {
    let harness = common::Harness::empty("not-a-route");
    for path in [
        "/",
        "/activities/1/laps",
        "/activities/1/geo",
        "/physiology/readiness",
        "/stats/monthly",
        "/stats/hr-zones",
        "/network/status",
        "/tiles/policy",
        "/gear",
        "/auth/pair-code",
        "/auth/garmin/developer-key",
        "/insights",
        "/plan",
    ] {
        assert!(
            harness.router.route_at(path).is_none(),
            "the path {path} names a route"
        );
        let answer = harness.router.call("GET", path, None);
        assert_eq!(answer.status, 404, "the path {path} answered");
        assert_eq!(answer.body["code"], "not_found");
    }
}

#[test]
fn the_query_string_is_not_part_of_the_route() {
    let harness = common::Harness::empty("query");
    let plain = harness.router.call("GET", "/activities", None);
    let with_query = harness.router.call("GET", "/activities?limit=5&since=2026-01-01", None);
    assert_eq!(plain.status, with_query.status);
    assert!(harness.router.route_at("/activities?x=1").is_some());
}

#[test]
fn the_desktops_prefix_is_cut_off_before_the_route_is_matched() {
    let harness = common::Harness::empty("prefix");
    let plain = harness.router.call("GET", "/activities", None);
    let prefixed = harness.router.call("GET", "/api/v1/activities", None);
    assert_eq!(plain.status, prefixed.status);
    assert_eq!(plain.body, prefixed.body);
}

#[test]
fn every_route_is_reachable_with_its_own_method() {
    let harness = common::Harness::empty("methods");
    for route in stride_api::ROUTES.iter() {
        let path = concrete(route.pattern);
        let method = route.method.as_str();
        let answer = harness.router.call(method, &path, Some("{}"));
        assert!(
            answer.status != 405,
            "the route {path} did not answer a {method}: {}",
            answer.body
        );
    }
}
