//! `stride-api` — the one router, the DTOs, and the FFI surface.
//!
//! **One core, two hosts.** The desktop serves this router over loopback
//! HTTP; the phone calls it in-process through the FFI. Neither can
//! disagree about a number, because there is one route table, one set of
//! response shapes, one clock and one set of sentences.
//!
//! ## What is here
//!
//! | module | what it is |
//! |---|---|
//! | [`router`] | the dispatcher: route → handler → JSON, and the frozen route table |
//! | [`dto`] | the response shapes, one type per route |
//! | [`render`] | the rendered value: the core's words, and the number for a chart |
//! | [`state`] | the shared state: the store, the Garmin key, the configuration, the clock |
//! | [`error`] | the typed failures and the envelope they travel in |
//! | [`ffi`] | the FFI surface: JSON in, JSON out, and the panic boundary |
//! | [`routes`] | the handlers, and the helpers they share |
//! | [`plan`] | the seam to the core's plan engine: the store's rows in, the core's answers out |
//! | [`planfile`] | the plan files the athlete may import: markdown, CSV, JSON, the Garmin schedule |
//!
//! ## The rules this crate keeps
//!
//! * **The core renders the sentences and the numbers.** A direction word,
//!   the arithmetic beside it, the direction of travel, the "Rest" line, a
//!   pattern sentence and the three statements — the distance comparison,
//!   the Progress headline, the Compare basis — are written by
//!   `stride-core` and travel to the UI already written. A domain value
//!   crosses the wire as a [`RenderedValue`]: the core's words *and* the
//!   raw number a chart draws. There is no `format:` hint — a hint tells
//!   the app how to render, which is the hole the contract closes. This
//!   crate never rounds a number on its own and never writes a domain
//!   sentence of its own.
//! * **"No value" is `null`, never `0`.** A missing reading is `None`, and
//!   the fields that may be absent are `Option`.
//! * **The Garmin key never enters a response.** It arrives as an argument,
//!   it is used for the calls that need it, and it is not returned, not
//!   logged and not embedded in an error.
//! * **One retry, one owner.** The ingest crate performs one token
//!   exchange and one attempt per call; the router adds no retry of its
//!   own and surfaces `needs_reconnect` as the typed failure it is. The
//!   caller may make one more attempt (`docs/contract/core-contract.md`
//!   §7).
//! * **Every response is bounded.** The feed is paged with a cursor, the
//!   series are capped by the configuration, and the FIT file is read
//!   through the ingest crate's own ceiling.
//! * **The failures are typed.** `no_plan`, `stale_data`,
//!   `not_enough_history`, `garmin_rejected`, `needs_reconnect`,
//!   `not_found`, `bad_request`, `locked` — each one is a sentence the app
//!   can show.
//!
//! ## The answer is the body
//!
//! The router answers with the response **body** — the DTO, and nothing
//! around it (`docs/contract/api-paths.md`, "The shape of an answer"). The
//! host puts the status in the status line: the desktop's HTTP layer, or
//! the phone's bridge, which rejects the call when the router failed. The
//! FFI's own envelope is the seam's business (`android/…/stride-ffi`),
//! and the two hosts hand the app the same bytes.
//!
//! ## How a host wires this
//!
//! The router is the whole of the domain's answer, and a host is a dozen
//! lines around it:
//!
//! ```no_run
//! use std::sync::Arc;
//! use stride_api::{Api, Config, Router, SystemClock};
//! use stride_store::{DatabaseKey, Store};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let key = DatabaseKey::from_text("the key the caller holds")?;
//! let store = Arc::new(Store::open_and_migrate(std::path::Path::new("stride.db"), &key)?);
//! let api = Arc::new(Api::new(store, Config::default(), Arc::new(SystemClock))?);
//! let router = Router::new(api);
//!
//! // The desktop's HTTP layer: the body on the wire, the status in the
//! // status line.
//! let answer = router.call("GET", "/api/v1/activities", None);
//! let (status, body) = (answer.status, answer.body);
//!
//! // The phone's bridge: the same envelope, and the shell unwraps it.
//! let envelope = router.envelope("GET", "/api/v1/activities", None);
//! # Ok(())
//! # }
//! ```
//!
//! **One envelope on both sides** ([`Router::envelope`]), so the two hosts
//! hand the app the same bytes and a number cannot differ between them
//! without one of them being wrong.
//!
//! ## The route table is frozen
//!
//! `docs/contract/api-paths.md` is the list, and this crate implements
//! exactly it: no route is added and no shape is changed.
//! `tests/route_parity.rs` walks the contract's own list and holds the
//! router to it — every route in the contract answers, and no other route
//! does.
//!
//! ## An example
//!
//! ```no_run
//! use std::sync::Arc;
//! use stride_api::{Api, Config, FixedClock, Router, SystemClock};
//! use stride_store::{DatabaseKey, Store};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let key = DatabaseKey::from_text("the key the caller holds")?;
//! let store = Arc::new(Store::open_and_migrate(std::path::Path::new("stride.db"), &key)?);
//! let api = Arc::new(Api::new(store, Config::default(), Arc::new(SystemClock))?);
//! let router = Router::new(api);
//!
//! let answer = router.call("GET", "/activities", None);
//! assert!(answer.is_ok());
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

pub mod dto;
pub mod error;
pub mod ffi;
pub mod plan;
pub mod planfile;
pub mod render;
pub mod router;
pub mod routes;
pub mod state;

pub use dto::RenderedValue;
pub use error::{ApiError, ApiResult, ErrorEnvelope, ErrorKind};
pub use router::{Method, Request, Response, Route, Router, ROUTES};
pub use state::{Api, Clock, Config, FixedClock, GarminKey, SystemClock};
