//! The MCP surface itself.
//!
//! The harness speaks JSON-RPC over stdio. These tests are that
//! conversation: the handshake, the list of tools, a call, and the answers
//! to the two questions that matter — what happens when the harness asks for
//! a tool that does not exist, and what happens when a tool fails.

mod common;

use serde_json::{json, Value};
use stride_mcp::{handle, server};

fn request(id: i64, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

#[test]
fn the_handshake_says_what_the_server_speaks() {
    let tools = common::tools();
    let response = handle(&tools, &request(1, "initialize", json!({}))).expect("a response");
    assert_eq!(response["jsonrpc"], json!("2.0"));
    assert_eq!(
        response["result"]["protocolVersion"],
        json!(server::PROTOCOL_VERSION)
    );
    assert_eq!(response["result"]["serverInfo"]["name"], json!("stride"));
    assert!(response["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn the_tool_list_is_strict_and_read_only() {
    let tools = common::tools();
    let response = handle(&tools, &request(2, "tools/list", json!({}))).expect("a response");
    let listed = response["result"]["tools"].as_array().expect("tools");
    assert_eq!(listed.len(), 9);

    let names: Vec<&str> = listed
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "training_digest",
            "search_activities",
            "get_activity",
            "activity_analysis",
            "training_statistics",
            "plan_context",
            "physiology_context",
            "fingerprint_trends",
            "compare_runs",
        ]
    );

    for tool in listed {
        assert_eq!(tool["inputSchema"]["type"], json!("object"));
        assert_eq!(tool["inputSchema"]["additionalProperties"], json!(false));
        assert_eq!(tool["annotations"]["readOnlyHint"], json!(true));
        assert_eq!(tool["annotations"]["destructiveHint"], json!(false));
        assert_eq!(tool["annotations"]["openWorldHint"], json!(false));
    }
}

#[test]
fn a_call_answers_with_the_structured_content() {
    let tools = common::tools();
    let response = handle(
        &tools,
        &request(
            3,
            "tools/call",
            json!({
                "name": "search_activities",
                "arguments": { "from": "2026-10-01", "to": "2026-10-28" },
            }),
        ),
    )
    .expect("a response");
    let result = &response["result"];
    assert_eq!(result["isError"], json!(false));
    assert_eq!(
        result["structuredContent"]["tool"],
        json!("search_activities")
    );
    assert_eq!(result["content"][0]["type"], json!("text"));
    // The text is the same answer, rendered.
    let text = result["content"][0]["text"].as_str().expect("text");
    let parsed: Value = serde_json::from_str(text).expect("json");
    assert_eq!(parsed["tool"], json!("search_activities"));
}

#[test]
fn an_unknown_tool_is_a_protocol_error() {
    let tools = common::tools();
    let response = handle(
        &tools,
        &request(4, "tools/call", json!({ "name": "drop_the_database" })),
    )
    .expect("a response");
    assert_eq!(response["error"]["code"], json!(-32602));
}

#[test]
fn a_tool_that_fails_answers_with_the_message_and_not_a_crash() {
    let tools = common::tools();
    let response = handle(
        &tools,
        &request(
            5,
            "tools/call",
            json!({ "name": "compare_runs", "arguments": { "activity_ids": [1] } }),
        ),
    )
    .expect("a response");
    let result = &response["result"];
    assert_eq!(result["isError"], json!(true));
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        json!("invalid_params")
    );
    assert!(result["structuredContent"]["error"]["message"]
        .as_str()
        .expect("message")
        .contains("two to five"));
}

#[test]
fn the_server_answers_over_stdio_and_says_nothing_to_notifications() {
    let tools = common::tools();
    let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n\
                 {\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n\
                 {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n";
    let mut output: Vec<u8> = Vec::new();
    server::serve(tools, input.as_bytes(), &mut output).expect("the server");
    let text = String::from_utf8(output).expect("utf-8");
    let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), 2);
    let first: Value = serde_json::from_str(lines[0]).expect("json");
    assert_eq!(first["id"], json!(1));
    let second: Value = serde_json::from_str(lines[1]).expect("json");
    assert_eq!(second["id"], json!(2));
}

#[test]
fn a_broken_line_is_answered_with_a_parse_error_and_the_server_goes_on() {
    let tools = common::tools();
    let input = "not json\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
    let mut output: Vec<u8> = Vec::new();
    server::serve(tools, input.as_bytes(), &mut output).expect("the server");
    let text = String::from_utf8(output).expect("utf-8");
    let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), 2);
    let first: Value = serde_json::from_str(lines[0]).expect("json");
    assert_eq!(first["error"]["code"], json!(-32700));
    let second: Value = serde_json::from_str(lines[1]).expect("json");
    assert_eq!(second["id"], json!(1));
}
