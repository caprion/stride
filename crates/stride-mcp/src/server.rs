//! The MCP server.
//!
//! The harness speaks JSON-RPC 2.0 over stdio, one message per line. This
//! module answers three methods — `initialize`, `tools/list` and `tools/call`
//! — and nothing else. There is no method that writes, no method that reads
//! a file, no method that reaches a URL the caller names, and no method that
//! returns the athlete's data without a tool call.
//!
//! The server is deliberately thin. It knows how to speak MCP; it does not
//! know anything about Stride. Everything it answers comes from the tools.

use std::io::{BufRead, Write};

use serde_json::{json, Value};

use crate::error::McpError;
use crate::tools::Tools;

/// The protocol version this server speaks.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// The name the server answers with.
pub const SERVER_NAME: &str = "stride";

/// The version of the server.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Answer one JSON-RPC request.
///
/// `None` means the message was a notification: there is nothing to answer.
pub fn handle(tools: &Tools, request: &Value) -> Option<Value> {
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let id = request.get("id").cloned();
    let params = request.get("params").cloned().unwrap_or(Value::Null);

    let Some(id) = id else {
        // A notification: the client is told, not asked.
        return None;
    };

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
            "instructions": "Stride answers questions about the athlete's own runs, plan and \
        physiology. Every tool is read-only. A tool's answer is a disclosure: it says which activity or \
        date range it came from, and what it cannot be used for.",
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": tools.list().iter().map(|tool| tool.to_value()).collect::<Vec<Value>>(),
        })),
        "tools/call" => match call_tool(tools, &params) {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        },
        other => Err(McpError::invalid(
            "server",
            "method",
            format!("{other:?} is not a method this server answers"),
        )),
    };

    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code_for(&error),
                "message": error.to_string(),
            }
        }),
    })
}

/// Call a tool and wrap its answer the way MCP expects.
fn call_tool(tools: &Tools, params: &Value) -> Result<Value, McpError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| McpError::invalid("tools/call", "name", "is required"))?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match tools.call(name, &arguments) {
        Ok(value) => {
            let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string());
            Ok(json!({
                "content": [{ "type": "text", "text": text }],
                "structuredContent": value,
                "isError": false,
            }))
        }
        Err(McpError::UnknownTool { name }) => Err(McpError::UnknownTool { name }),
        Err(error) => {
            // A tool that fails is a result, not a protocol error: the
            // harness shows the message to the athlete.
            let text = serde_json::to_string_pretty(&error.to_value())
                .unwrap_or_else(|_| "{}".to_string());
            Ok(json!({
                "content": [{ "type": "text", "text": text }],
                "structuredContent": error.to_value(),
                "isError": true,
            }))
        }
    }
}

/// The JSON-RPC code of a failure.
fn code_for(error: &McpError) -> i64 {
    match error {
        McpError::UnknownTool { .. } => -32602,
        McpError::InvalidParams { .. } => -32602,
        _ => -32603,
    }
}

/// Serve MCP over stdio until the input ends.
pub fn serve(tools: Tools, input: impl BufRead, output: &mut impl Write) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("parse error: {error}") },
                });
                writeln!(output, "{response}")?;
                output.flush()?;
                continue;
            }
        };
        if let Some(response) = handle(&tools, &request) {
            writeln!(output, "{response}")?;
            output.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn tools() -> Tools {
        let api: Arc<dyn crate::api::StrideApi> = Arc::new(stride_api::Api::new(Arc::new(
            stride_api::mem::MemBackend::new(),
        )));
        Tools::with_today(
            api,
            chrono::NaiveDate::from_ymd_opt(2026, 10, 28).expect("date"),
        )
    }

    #[test]
    fn initialize_answers_with_the_protocol() {
        let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" });
        let response = handle(&tools(), &request).expect("a response");
        assert_eq!(
            response["result"]["protocolVersion"],
            json!(PROTOCOL_VERSION)
        );
        assert_eq!(response["id"], json!(1));
    }

    #[test]
    fn a_notification_is_not_answered() {
        let request = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(handle(&tools(), &request).is_none());
    }

    #[test]
    fn an_unknown_method_is_an_error() {
        let request = json!({ "jsonrpc": "2.0", "id": 7, "method": "shell" });
        let response = handle(&tools(), &request).expect("a response");
        assert_eq!(response["error"]["code"], json!(-32602));
    }

    #[test]
    fn the_tool_list_holds_the_nine_read_only_tools() {
        let request = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });
        let response = handle(&tools(), &request).expect("a response");
        let tools = response["result"]["tools"].as_array().expect("tools");
        assert_eq!(tools.len(), 9);
        for tool in tools {
            assert_eq!(tool["annotations"]["readOnlyHint"], json!(true));
        }
    }
}
