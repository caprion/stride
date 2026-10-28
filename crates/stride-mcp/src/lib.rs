//! `stride-mcp` — the harness tools.
//!
//! The harness — Claude Code, Codex, Gemini CLI — asks an athlete's
//! questions about the athlete's own runs, and this crate answers them.
//! Everything it answers comes from the Stride router, so the desktop and
//! the phone give the same answer to the same question, and the numbers are
//! the numbers the app shows.
//!
//! **The tools are written once, over the router.** The nine tools are
//! question-shaped aggregates, not one tool per endpoint: the first call of a
//! session should not have to make six calls and join the answers itself.
//!
//! **Read-only by default.** Nothing here writes. The design's two write
//! tools need an audit trail and an undo token, and the router has no audit
//! surface to write that trail to — so they are not here.
//!
//! **Every answer is bounded.** The athlete's history is large and a
//! harness's context is small. Every list is capped, every series is
//! downsampled, and the whole answer is kept inside
//! [`output::MAX_OUTPUT_BYTES`].
//!
//! **Every answer is a disclosure.** The output of a tool ends up in a
//! model's context. Only the fields an athlete would want there are passed
//! on: no Garmin identifiers, no device names, no notes, no credentials, no
//! file paths — and the route, which is the athlete's location, only when it
//! is asked for by name.
//!
//! ```
//! use std::sync::Arc;
//! use stride_mcp::{StrideApi, Tools};
//!
//! let api: Arc<dyn StrideApi> = Arc::new(stride_api::Api::new(Arc::new(
//!     stride_api::mem::MemBackend::new(),
//! )));
//! let tools = Tools::new(api);
//! assert_eq!(tools.list().len(), 9);
//! ```
//!
//! **How a host wires it.** The desktop and the phone both hold a
//! `stride_api::Api` over their own store, and both hand it to this crate:
//!
//! ```no_run
//! use std::sync::Arc;
//! use stride_mcp::Tools;
//!
//! # fn run(store: Arc<dyn stride_api::backend::Backend>) -> std::io::Result<()> {
//! let api = stride_api::Api::new(store);
//! let tools = Tools::new(Arc::new(api));
//! let stdin = std::io::stdin();
//! let mut stdout = std::io::stdout();
//! stride_mcp::serve(tools, stdin.lock(), &mut stdout)
//! # }
//! ```
//!
//! The process, the pairing and the revocation belong to the host: this
//! crate has no binary of its own, because the sidecar the design document
//! describes talks to a running desktop over HTTP, and this crate is the
//! layer that works the same on both hosts.

pub mod api;
pub mod args;
pub mod error;
pub mod output;
pub mod schema;
pub mod server;
pub mod tools;

pub use api::StrideApi;
pub use error::McpError;
pub use output::{Envelope, MAX_ITEMS, MAX_OUTPUT_BYTES, MAX_SAMPLES};
pub use server::{handle, serve, PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION};
pub use tools::{Context, Tool, Tools};
