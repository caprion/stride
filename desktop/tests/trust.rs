//! The trust model.
//!
//! Three lines, and each of them is checked here:
//!
//! 1. a request from the loopback is answered;
//! 2. a request from an address that is not trusted is refused, with a
//!    message that names the variable;
//! 3. a request from an address inside a trusted network is answered.
//!
//! The refused request is the one that matters: a rig that answers a network
//! it was not asked to answer would put the Garmin key on that network.

mod common;

use stride_desktop::trust::{Decision, Trust, TRUSTED_NETWORKS_VARIABLE};

use common::{elsewhere, inside_the_trusted_network, loopback, request, server};

#[test]
fn a_request_from_the_loopback_is_answered() {
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/api/v1/activities"), loopback());
    assert_eq!(response.status, 200, "the loopback is always answered");
}

#[test]
fn a_request_from_an_untrusted_address_is_refused() {
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/api/v1/activities"), elsewhere());
    assert_eq!(response.status, 403);
}

#[test]
fn the_refusal_names_the_variable() {
    let server = server(Trust::loopback_only());
    let response = server.handle(&request("GET", "/api/v1/activities"), elsewhere());
    let body = common::body_of(&response);
    assert!(
        body.contains(TRUSTED_NETWORKS_VARIABLE),
        "the refusal has to say what to set: {body}"
    );
    assert!(
        body.contains("192.168.1.5"),
        "the refusal has to say where the request came from: {body}"
    );
}

#[test]
fn a_request_from_a_trusted_network_is_answered() {
    let trust = Trust::from_list("10.0.0.0/8").unwrap_or_else(|_| Trust::loopback_only());
    let server = server(trust);
    let response = server.handle(
        &request("GET", "/api/v1/activities"),
        inside_the_trusted_network(),
    );
    assert_eq!(response.status, 200, "a trusted network is answered");
}

#[test]
fn a_network_outside_the_trusted_one_is_still_refused() {
    let trust = Trust::from_list("10.0.0.0/8").unwrap_or_else(|_| Trust::loopback_only());
    let server = server(trust);
    let response = server.handle(&request("GET", "/api/v1/activities"), elsewhere());
    assert_eq!(response.status, 403);
}

#[test]
fn the_decision_is_made_before_anything_else() {
    // The trust model itself, without the server around it.
    let trust = Trust::loopback_only();
    assert_eq!(trust.decide(loopback()), Decision::Allow);
    assert!(matches!(trust.decide(elsewhere()), Decision::Refuse(_)));

    let trust = Trust::from_list("192.168.1.0/24").unwrap_or_else(|_| Trust::loopback_only());
    assert_eq!(trust.decide(elsewhere()), Decision::Allow);
    assert!(matches!(
        trust.decide(inside_the_trusted_network()),
        Decision::Refuse(_)
    ));
}

#[test]
fn a_list_that_cannot_be_read_is_refused() {
    assert!(Trust::from_list("not a network").is_err());
    assert!(Trust::from_list("192.168.1.0/40").is_err());
    assert!(Trust::from_list("")
        .unwrap_or_else(|_| Trust::loopback_only())
        .networks()
        .is_empty());
}

#[test]
fn the_loopback_is_allowed_in_every_shape() {
    // A peer that arrives as an IPv4-mapped IPv6 address is on the loopback,
    // whatever the socket says about it.
    let trust = Trust::loopback_only();
    let mapped: std::net::IpAddr = "::ffff:127.0.0.1".parse().unwrap_or(loopback());
    assert_eq!(trust.decide(mapped), Decision::Allow);
}
