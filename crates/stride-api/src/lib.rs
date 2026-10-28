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
//! **The response shapes do not change.** The React app consumes them and it
//! is not being rewritten, so every key, every `null` and every rounding is
//! a port of the Python implementation. `docs/rust-core-spec.md` and
//! `docs/core-contract.md` are the contracts; the golden fixtures decide.

pub mod analytics;
pub mod backend;
pub mod core;
pub mod error;
pub mod ffi;
pub mod mem;
pub mod num;
pub mod query;
pub mod router;
pub mod routes;
pub mod shapes;
pub mod store;
pub mod util;

use std::sync::Arc;

use serde_json::Value;

use crate::backend::Backend;
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
        let (path, query_string) = match path.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (path, None),
        };
        let query = Query::parse(query_string);
        let (id, params) = router::match_route(method, path)?;
        let ctx = Ctx {
            backend: self.backend.as_ref(),
            today: self.today,
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
