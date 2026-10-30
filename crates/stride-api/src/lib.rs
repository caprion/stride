//! `stride-api` — the one router, and the C ABI the phone calls.
//!
//! Everything the app exposes goes through [`Api::call`]. The route table is
//! in [`router`], the response shapes are in [`shapes`], and the boundary to
//! the rest of the system is the [`Backend`] trait.
//!
//! ```
//! use std::sync::Arc;
//! use stride_api::{Api, MemBackend};
//!
//! let api = Api::new(Arc::new(MemBackend::new()));
//! let health = api.call("GET", "/api/v1/health", None).unwrap();
//! assert_eq!(health["ok"], true);
//! ```
//!
//! ## How the pieces stack
//!
//! The router reads from a [`Backend`], and the backends stack:
//!
//! ```text
//! Api
//!  └─ CoreBackend      the domain: fingerprint, readiness, plan verdicts, comparison
//!      └─ NetworkBackend  the machine's exit node, read from Tailscale
//!          └─ StateBackend  the credentials, the pairing code, the sync settings
//!              └─ stride_store::Store   the rows
//! ```
//!
//! Each layer is optional: a host that has no Tailscale leaves the network
//! layer out, and a host that keeps its state elsewhere leaves the state
//! layer out. `stride_api::store::backend` builds the bottom layer,
//! `stride_api::core::CoreBackend` and `stride_api::network::NetworkBackend`
//! and `stride_api::state::StateBackend` the ones above it.
//!
//! **The response shapes do not change.** The React app consumes them and it
//! is not being rewritten, so every key, every `null` and every rounding is
//! a port of the Python implementation. `docs/rust-core-spec.md` and
//! `docs/core-contract.md` are the contracts; the golden fixtures decide.

pub mod analytics;
pub mod backend;
pub mod compress;
pub mod core;
pub mod error;
pub mod ffi;
pub mod insights;
pub mod mem;
pub mod network;
pub mod num;
pub mod plan_import;
pub mod query;
pub mod router;
pub mod routes;
pub mod shapes;
pub mod state;
pub mod store;
pub mod sync;
pub mod tiles;
pub mod util;

use std::sync::Arc;

use serde_json::Value;

pub use crate::backend::{Backend, Caller};
use crate::error::ApiError;
use crate::query::Query;
use crate::routes::Ctx;

/// What a route answered.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// A JSON body.
    Json(Value),
    /// A body that is not JSON — the map tiles.
    Binary {
        content_type: String,
        bytes: Vec<u8>,
    },
    /// No body at all: the pairing route answers 204.
    NoContent,
}

/// A response ready to be written to the wire.
///
/// The router answers with JSON; the host writes bytes. This is the bytes,
/// with the headers that describe them.
#[derive(Debug, Clone, PartialEq)]
pub struct Encoded {
    /// The HTTP status.
    pub status: u16,
    /// The content type, e.g. `application/json`.
    pub content_type: String,
    /// The body, compressed when the header says so.
    pub body: Vec<u8>,
    /// `Some("gzip")` when the body is a gzip stream.
    pub content_encoding: Option<&'static str>,
}

impl Encoded {
    /// The body as text.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

impl Response {
    /// The body as JSON, for the callers that want one value.
    ///
    /// A binary body becomes `{"content_type": ..., "base64": ...}` — which is
    /// exactly what the phone wants, because a Kotlin string cannot carry a
    /// PNG. A body-less answer becomes `null`.
    pub fn to_value(&self) -> Value {
        match self {
            Response::Json(value) => value.clone(),
            Response::NoContent => Value::Null,
            Response::Binary {
                content_type,
                bytes,
            } => serde_json::json!({
                "content_type": content_type,
                "base64": util::base64_encode(bytes),
            }),
        }
    }
}

/// The whole stack, built in one call.
///
/// The layers are: the domain on top, then the map tiles, then the machine's
/// network route, then the application state, then the rows. A host that
/// wants them all asks for them all:
///
/// ```no_run
/// # use std::sync::Arc;
/// # use stride_api::{stack, Backend};
/// # fn run(store: Arc<dyn Backend>, cache: &std::path::Path) {
/// let api = stack(store, cache);
/// # }
/// ```
///
/// A host that wants fewer layers builds them by hand — the order is the
/// only thing that matters, and it is documented on [`Api`].
pub fn stack(backend: Arc<dyn Backend>, cache_dir: &std::path::Path) -> Api {
    let tiles = tiles::TileBackend::new(backend, cache_dir.to_path_buf());
    let network = network::NetworkBackend::new(Arc::new(tiles));
    let state = state::StateBackend::new(Arc::new(network));
    let core = core::CoreBackend::new(Arc::new(state));
    Api::new(Arc::new(core))
}

/// The router.
pub struct Api {
    backend: Arc<dyn Backend>,
    /// The day the date-dependent routes count from.
    today: chrono::NaiveDate,
}

impl Api {
    /// A router over a backend.
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self {
            backend,
            today: chrono::Utc::now().date_naive(),
        }
    }

    /// A router that counts from a fixed day.
    ///
    /// The golden test needs a fixed day, because "this week" and "this
    /// month" are relative to it.
    pub fn with_today(backend: Arc<dyn Backend>, today: chrono::NaiveDate) -> Self {
        Self { backend, today }
    }

    /// The backend the router reads from.
    pub fn backend(&self) -> &Arc<dyn Backend> {
        &self.backend
    }

    /// Call a route.
    ///
    /// `method` and `path` are the HTTP method and the path; the query
    /// string may be part of `path`. `body` is the request body, if the route
    /// takes one.
    pub fn call(&self, method: &str, path: &str, body: Option<&str>) -> Result<Value, ApiError> {
        Ok(self.call_response(method, path, body)?.to_value())
    }

    /// Call a route and write the answer out.
    ///
    /// `accept_gzip` says the client can read a gzip stream. A JSON body is
    /// compressed when the client asks for it and when compressing it makes
    /// it smaller — a 40-byte body is left alone, because a compressed
    /// 40-byte body is larger than the 40-byte body.
    ///
    /// The host uses this: the desktop writes `content_encoding` into the
    /// `Content-Encoding` header and the body into the response.
    pub fn call_encoded(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        accept_gzip: bool,
    ) -> Result<Encoded, ApiError> {
        let response = self.call_response(method, path, body)?;
        Ok(match response {
            Response::Json(value) => {
                let bytes = serde_json::to_vec(&value).map_err(|err| {
                    ApiError::internal(format!("the answer could not be written: {err}"))
                })?;
                let (bytes, encoding) = if accept_gzip {
                    compress::compress_if_worth_it(bytes)
                        .map_err(|err| ApiError::internal(err.to_string()))?
                } else {
                    (bytes, None)
                };
                Encoded {
                    status: 200,
                    content_type: "application/json".to_string(),
                    body: bytes,
                    content_encoding: encoding,
                }
            }
            Response::Binary {
                content_type,
                bytes,
            } => Encoded {
                status: 200,
                content_type,
                body: bytes,
                // A PNG is already compressed; gzipping it costs CPU and
                // buys nothing.
                content_encoding: None,
            },
            Response::NoContent => Encoded {
                status: 204,
                content_type: "application/json".to_string(),
                body: Vec::new(),
                content_encoding: None,
            },
        })
    }

    /// Call a route, keeping the kind of the answer.
    ///
    /// The HTTP layer uses this to serve a map tile as a PNG and the pairing
    /// route as a 204.
    pub fn call_response(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<Response, ApiError> {
        self.call_response_from(&Caller::default(), method, path, body)
    }

    /// Call a route on behalf of a caller.
    ///
    /// The caller is who the host says is asking: the address the
    /// connection came from and the name it used. The routes that depend on
    /// that — the pairing code — read it from here.
    pub fn call_response_from(
        &self,
        caller: &Caller,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<Response, ApiError> {
        let (path, query_string) = match path.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (path, None),
        };
        let query = Query::parse(query_string);
        let (id, params) = router::match_route(method, path)?;
        let ctx = Ctx {
            backend: self.backend.as_ref(),
            today: self.today,
            caller,
        };
        router::dispatch(&ctx, id, &params, &query, body)
    }
}

impl std::fmt::Debug for Api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Api")
            .field("today", &self.today)
            .finish_non_exhaustive()
    }
}

pub use core::CoreBackend;
pub use error::{ApiError as Error, BackendError};
pub use mem::MemBackend;
pub use network::NetworkBackend;
pub use state::{State, StateBackend};
pub use tiles::TileBackend;

#[cfg(test)]
mod tests {
    //! The router, exercised from inside the crate.

    use std::sync::Arc;

    use serde_json::Value;

    use crate::backend::ActivityRow;
    use crate::mem::MemBackend;
    use crate::Api;

    /// A backend with enough runs for the answers to be worth compressing.
    fn busy_backend() -> Arc<MemBackend> {
        let backend = MemBackend::new();
        for index in 0..200 {
            backend.add_activity(ActivityRow {
                id: index + 1,
                source: "garmin_fit".to_string(),
                external_id: format!("{index}"),
                name: format!("run {index}"),
                sport_type: "running".to_string(),
                local_date: "2026-03-02".to_string(),
                start_time: Some("2026-03-02T06:15:00+00:00".to_string()),
                distance_m: 10_000.0,
                moving_time_s: 3_000.0,
                elapsed_time_s: 3_100.0,
                geo_points: Some(
                    (0..90)
                        .map(|point| [18.52 + point as f64 * 0.001, 73.85 + point as f64 * 0.001])
                        .collect(),
                ),
                ..ActivityRow::default()
            });
        }
        Arc::new(backend)
    }

    /// A big answer is compressed when the client asks for it.
    #[test]
    fn a_big_answer_is_compressed() {
        let api = Api::new(busy_backend());
        let encoded = api
            .call_encoded("GET", "/api/v1/activities?per_page=200", None, true)
            .expect("the list answers");
        assert_eq!(encoded.content_encoding, Some("gzip"));
        assert_eq!(&encoded.body[..2], &[0x1f, 0x8b]);

        let mut decoded = Vec::new();
        {
            let mut decoder = flate2::read::GzDecoder::new(&encoded.body[..]);
            std::io::Read::read_to_end(&mut decoder, &mut decoded).expect("it decompresses");
        }
        let plain = api
            .call_encoded("GET", "/api/v1/activities?per_page=200", None, false)
            .expect("the list answers");
        assert_eq!(decoded, plain.body);
        assert_eq!(plain.content_encoding, None);
        // The compressed answer is much smaller than the plain one.
        assert!(
            encoded.body.len() < plain.body.len() / 4,
            "the plain answer was {} bytes and the compressed one {}",
            plain.body.len(),
            encoded.body.len()
        );
    }

    /// A small answer is left alone: a compressed 40-byte body is larger.
    #[test]
    fn a_small_answer_is_not_compressed() {
        let api = Api::new(Arc::new(MemBackend::new()));
        let encoded = api
            .call_encoded("GET", "/api/v1/health", None, true)
            .expect("the health answers");
        assert_eq!(encoded.content_encoding, None);
        let body: Value = serde_json::from_slice(&encoded.body).expect("it is JSON");
        assert_eq!(body["ok"], true);
    }

    /// A client that does not ask for gzip never gets it.
    #[test]
    fn gzip_is_only_for_the_clients_that_ask() {
        let api = Api::new(busy_backend());
        let encoded = api
            .call_encoded("GET", "/api/v1/activities?per_page=200", None, false)
            .expect("the list answers");
        assert_eq!(encoded.content_encoding, None);
        assert!(
            encoded.body.len() > 100_000,
            "the plain answer was {} bytes",
            encoded.body.len()
        );
    }
}
