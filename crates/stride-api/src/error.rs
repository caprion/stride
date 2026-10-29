//! The error shape the Python app returns, and the errors the router produces.
//!
//! The Python app raises `HTTPException(status_code, detail)`, and FastAPI
//! renders it as `{"detail": "..."}` with the status code. A request that
//! fails validation renders as `{"detail": [ {...}, ... ]}` with 422. The
//! frontend reads `detail`, so the shape is part of the contract and does not
//! change.

use serde_json::{json, Value};
use thiserror::Error;

/// An error the router returns: an HTTP status and the `detail` payload.
///
/// `detail` is a string for an `HTTPException` and an array of objects for a
/// FastAPI validation error, because that is what the Python app produces and
/// the frontend can tell them apart.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    pub status: u16,
    pub detail: Value,
}

impl ApiError {
    /// A `HTTPException` with a string detail — the common case.
    pub fn new(status: u16, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: Value::String(detail.into()),
        }
    }

    /// 400 — the request was understood but is wrong.
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, detail)
    }

    /// 403 — the caller may not do this.
    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::new(403, detail)
    }

    /// 404 — no such thing.
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, detail)
    }

    /// 422 — the request failed validation.
    pub fn unprocessable(detail: Value) -> Self {
        Self {
            status: 422,
            detail,
        }
    }

    /// 500 — the server broke.
    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(500, detail)
    }

    /// 501 — the operation exists in the contract but this build cannot do it.
    pub fn not_implemented(detail: impl Into<String>) -> Self {
        Self::new(501, detail)
    }

    /// The body FastAPI would render for this error.
    pub fn to_body(&self) -> Value {
        json!({ "detail": self.detail })
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.detail)
    }
}

impl std::error::Error for ApiError {}

/// Something the storage layer or the domain layer could not do.
///
/// The router turns this into a 500 (or a 501 when the operation is not
/// implemented by this build) — never into a panic.
#[derive(Debug, Error)]
pub enum BackendError {
    /// The store could not be read or written.
    #[error("store error: {0}")]
    Store(String),

    /// The domain layer refused the request.
    #[error("domain error: {0}")]
    Domain(String),

    /// The caller may not do this.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// The caller did not prove who they are.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// Something upstream could not be reached.
    ///
    /// 502: the request was fine and the thing it asked for is not there.
    #[error("unavailable: {0}")]
    Unavailable(String),

    /// The operation is part of the contract but this build does not implement
    /// it — the phone build has no Garmin client, for instance.
    #[error("not implemented: {0}")]
    Unsupported(String),

    /// A row could not be decoded into the shape the router expects.
    #[error("malformed row: {0}")]
    Malformed(String),
}

impl BackendError {
    /// The message the API shows.
    ///
    /// The `detail` of the answer is the message alone — the same text the
    /// Python's `HTTPException` carries — so the prefix thiserror adds for
    /// the logs never reaches the browser.
    pub fn detail(&self) -> &str {
        match self {
            BackendError::Store(message)
            | BackendError::Domain(message)
            | BackendError::Forbidden(message)
            | BackendError::Unauthorized(message)
            | BackendError::Unavailable(message)
            | BackendError::Unsupported(message)
            | BackendError::Malformed(message) => message,
        }
    }

    /// The status this failure maps to.
    pub fn status(&self) -> u16 {
        match self {
            BackendError::Unsupported(_) => 501,
            BackendError::Domain(_) => 400,
            BackendError::Forbidden(_) => 403,
            BackendError::Unauthorized(_) => 401,
            BackendError::Unavailable(_) => 502,
            _ => 500,
        }
    }
}

impl From<BackendError> for ApiError {
    fn from(err: BackendError) -> Self {
        ApiError::new(err.status(), err.detail().to_string())
    }
}

/// The FastAPI validation error body, for the cases FastAPI itself rejects.
///
/// The shape is FastAPI's: a list of objects, each naming the location, the
/// message, the kind of error, the value that was sent and — for a bound —
/// the bound it broke. The frontend only looks at `detail`, but a client that
/// wants to show *why* a request was rejected reads the rest.
pub fn validation_error(loc: Vec<&str>, msg: &str, kind: &str) -> ApiError {
    validation_error_with(loc, msg, kind, Value::Null, Value::Null)
}

/// The validation error body, with the value that was sent and the bound.
pub fn validation_error_with(
    loc: Vec<&str>,
    msg: &str,
    kind: &str,
    input: Value,
    ctx: Value,
) -> ApiError {
    let mut error = serde_json::Map::new();
    error.insert("type".into(), json!(kind));
    error.insert("loc".into(), json!(loc));
    error.insert("msg".into(), json!(msg));
    error.insert("input".into(), input);
    if !ctx.is_null() {
        error.insert("ctx".into(), ctx);
    }
    ApiError::unprocessable(Value::Array(vec![Value::Object(error)]))
}
