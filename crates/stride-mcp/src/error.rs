//! The errors the tools produce.
//!
//! An MCP tool that fails has to fail *legibly*: the harness shows the
//! message to the athlete (or to the model), so every error says which tool
//! failed and what was wrong with the request. Nothing here ever carries a
//! credential, a file path or a stack of internal detail — the disclosure
//! rules apply to errors as much as to data.

use serde_json::Value;
use thiserror::Error;
use stride_api::error::ApiError;

/// Something a tool could not do.
#[derive(Debug, Error)]
pub enum McpError {
    /// The harness asked for a tool that does not exist.
    #[error("unknown tool: {name}")]
    UnknownTool {
        /// The name the harness asked for.
        name: String,
    },

    /// The arguments were wrong.
    #[error("{tool}: {field}: {message}")]
    InvalidParams {
        /// The tool that was called.
        tool: String,
        /// The argument that was wrong.
        field: String,
        /// What was wrong with it.
        message: String,
    },

    /// The Stride API answered with an error.
    #[error("{tool}: the Stride API answered {status}: {detail}")]
    Api {
        /// The tool that was called.
        tool: String,
        /// The HTTP status the router returned.
        status: u16,
        /// The router's own detail.
        detail: String,
    },

    /// The data the tool needs is not in the database.
    #[error("{tool}: {reason}")]
    Unavailable {
        /// The tool that was called.
        tool: String,
        /// Why the data is not there.
        reason: String,
    },

    /// The answer could not be rendered.
    #[error("{tool}: could not render the answer: {reason}")]
    Render {
        /// The tool that was called.
        tool: String,
        /// What went wrong.
        reason: String,
    },
}

impl McpError {
    /// An invalid argument.
    pub fn invalid(tool: &str, field: &str, message: impl Into<String>) -> Self {
        Self::InvalidParams {
            tool: tool.to_string(),
            field: field.to_string(),
            message: message.into(),
        }
    }

    /// The data the tool needs is missing.
    pub fn unavailable(tool: &str, reason: impl Into<String>) -> Self {
        Self::Unavailable {
            tool: tool.to_string(),
            reason: reason.into(),
        }
    }

    /// The router answered with an error.
    pub fn api(tool: &str, error: &ApiError) -> Self {
        Self::Api {
            tool: tool.to_string(),
            status: error.status,
            detail: detail_of(&error.detail),
        }
    }

    /// The tool that failed.
    pub fn tool(&self) -> &str {
        match self {
            Self::UnknownTool { .. } => "",
            Self::InvalidParams { tool, .. }
            | Self::Api { tool, .. }
            | Self::Unavailable { tool, .. }
            | Self::Render { tool, .. } => tool,
        }
    }

    /// A machine-readable code, so a harness can branch on the failure.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownTool { .. } => "unknown_tool",
            Self::InvalidParams { .. } => "invalid_params",
            Self::Api { .. } => "api_error",
            Self::Unavailable { .. } => "unavailable",
            Self::Render { .. } => "render_error",
        }
    }

    /// The answer the harness sees.
    ///
    /// The shape is the same for every failure, so a harness can render it
    /// without knowing which error it got.
    pub fn to_value(&self) -> Value {
        serde_json::json!({
            "error": {
                "code": self.code(),
                "tool": self.tool(),
                "message": self.to_string(),
            }
        })
    }
}

/// The router's detail, as one line of text.
fn detail_of(detail: &Value) -> String {
    match detail {
        Value::String(text) => text.clone(),
        Value::Null => "no detail".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_error_carries_the_tool_and_the_field() {
        let error = McpError::invalid("search_activities", "limit", "must be 1..100");
        assert_eq!(error.tool(), "search_activities");
        assert_eq!(error.code(), "invalid_params");
        assert!(error.to_string().contains("limit"));
    }

    #[test]
    fn the_value_shape_is_the_same_for_every_failure() {
        let error = McpError::unavailable("fingerprint_trends", "no fingerprints");
        let value = error.to_value();
        assert_eq!(value["error"]["code"], "unavailable");
        assert_eq!(value["error"]["tool"], "fingerprint_trends");
    }
}
