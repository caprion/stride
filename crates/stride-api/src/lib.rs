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
//! | [`state`] | the shared state: the store, the Garmin key, the configuration, the clock |
//! | [`error`] | the typed failures and the envelope they travel in |
//! | [`ffi`] | the FFI surface: JSON in, JSON out, and the panic boundary |
//! | [`routes`] | the handlers, and the helpers they share |
//!
//! ## The rules this crate keeps
//!
//! * **The core renders the sentences and the numbers.** A direction word,
//!   the arithmetic beside it, the direction of travel, the "Rest" line and
//!   a pattern sentence are written by `stride-core` and travel to the UI
//!   already written. This crate never rounds a number on its own — every
//!   rounding rule is the core's — and it never writes a domain sentence of
//!   its own.
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
//!   `not_found`, `bad_request` — each one is a sentence the app can show.
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
pub mod routes;
pub mod router;
pub mod state;

pub use error::{ApiError, ApiResult, ErrorEnvelope, ErrorKind};
pub use router::{Method, Request, Response, Route, Router, ROUTES};
pub use state::{Api, Clock, Config, FixedClock, GarminKey, SystemClock};
