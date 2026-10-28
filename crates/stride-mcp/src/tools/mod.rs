//! The tools.
//!
//! Nine read-only tools, one per question an athlete actually asks. They are
//! written once, over the router, so the desktop and the phone answer the
//! same question the same way.
//!
//! **Read-only by default.** Nothing here writes. The design's two write
//! tools (`update_activity_notes` and `edit_plan`) are deliberately absent:
//! they need an audit trail and an undo token, and the router has no audit
//! surface to write that trail to. A tool that writes without a trail is
//! worse than no tool at all, so the registry has no way to turn one on.
//!
//! **Every answer is attributable.** Each tool says which activity or which
//! date range it answered from, in its `coverage`.
//!
//! **Every answer is bounded.** A tool returns a digest, a page, or a
//! downsampled series — never the athlete's whole history.

pub mod activity;
pub mod analysis;
pub mod compare;
pub mod common;
pub mod digest;
pub mod fingerprint;
pub mod physiology;
pub mod plan;
pub mod search;
pub mod statistics;

use std::sync::Arc;

use chrono::NaiveDate;
use serde_json::{json, Value};

use crate::api::StrideApi;
use crate::error::McpError;

/// What a tool needs to know beyond its arguments.
#[derive(Debug, Clone, Copy)]
pub struct Context {
    /// The day the tools count from — "this week" means the week of this
    /// day.
    ///
    /// It is the same day the router counts from; a host that fixed the
    /// router's day fixes this one too.
    pub today: NaiveDate,
}

impl Context {
    /// A context for a day.
    pub fn new(today: NaiveDate) -> Self {
        Self { today }
    }

    /// Today, as `YYYY-MM-DD`.
    pub fn today_text(&self) -> String {
        common::date_text(self.today)
    }
}

/// One tool, as the harness sees it.
#[derive(Debug, Clone)]
pub struct Tool {
    /// The name the harness calls.
    pub name: &'static str,
    /// What the tool does.
    pub description: &'static str,
    /// The JSON Schema of its arguments.
    pub input_schema: Value,
}

impl Tool {
    /// The tool, as MCP describes it.
    pub fn to_value(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false,
            },
        })
    }
}

/// The tools, and the router they read from.
pub struct Tools {
    api: Arc<dyn StrideApi>,
    today: NaiveDate,
}

impl Tools {
    /// The tools over a router.
    pub fn new(api: Arc<dyn StrideApi>) -> Self {
        Self {
            api,
            today: chrono::Utc::now().date_naive(),
        }
    }

    /// The tools over a router, counting from a fixed day.
    pub fn with_today(api: Arc<dyn StrideApi>, today: NaiveDate) -> Self {
        Self { api, today }
    }

    /// The context the tools run in.
    pub fn context(&self) -> Context {
        Context::new(self.today)
    }

    /// Every tool, in the order the harness should see them.
    pub fn list(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: digest::NAME,
                description: digest::DESCRIPTION,
                input_schema: digest::schema(),
            },
            Tool {
                name: search::NAME,
                description: search::DESCRIPTION,
                input_schema: search::schema(),
            },
            Tool {
                name: activity::NAME,
                description: activity::DESCRIPTION,
                input_schema: activity::schema(),
            },
            Tool {
                name: analysis::NAME,
                description: analysis::DESCRIPTION,
                input_schema: analysis::schema(),
            },
            Tool {
                name: statistics::NAME,
                description: statistics::DESCRIPTION,
                input_schema: statistics::schema(),
            },
            Tool {
                name: plan::NAME,
                description: plan::DESCRIPTION,
                input_schema: plan::schema(),
            },
            Tool {
                name: physiology::NAME,
                description: physiology::DESCRIPTION,
                input_schema: physiology::schema(),
            },
            Tool {
                name: fingerprint::NAME,
                description: fingerprint::DESCRIPTION,
                input_schema: fingerprint::schema(),
            },
            Tool {
                name: compare::NAME,
                description: compare::DESCRIPTION,
                input_schema: compare::schema(),
            },
        ]
    }

    /// Call a tool.
    ///
    /// The answer is the envelope: `data`, `coverage` and `limitations`.
    pub fn call(&self, name: &str, args: &Value) -> Result<Value, McpError> {
        let api = self.api.as_ref();
        let ctx = self.context();
        match name {
            digest::NAME => digest::run(api, &ctx, args),
            search::NAME => search::run(api, &ctx, args),
            activity::NAME => activity::run(api, &ctx, args),
            analysis::NAME => analysis::run(api, &ctx, args),
            statistics::NAME => statistics::run(api, &ctx, args),
            plan::NAME => plan::run(api, &ctx, args),
            physiology::NAME => physiology::run(api, &ctx, args),
            fingerprint::NAME => fingerprint::run(api, &ctx, args),
            compare::NAME => compare::run(api, &ctx, args),
            other => Err(McpError::UnknownTool {
                name: other.to_string(),
            }),
        }
    }
}

impl std::fmt::Debug for Tools {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tools").field("today", &self.today).finish()
    }
}
