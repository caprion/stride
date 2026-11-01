//! The typed errors, and the shape they travel in.
//!
//! A failure is **typed**: the UI renders each kind as its own sentence, so
//! a generic error is a bug rather than a fallback. The kinds are the ones
//! `docs/plan.html` §6.3 names — `no_plan`, `stale_data`,
//! `not_enough_history`, `garmin_rejected` — plus the two the transport
//! and the router need: `needs_reconnect` (the Garmin session ended) and
//! `bad_request` (the caller asked for something that cannot be read).
//!
//! ```text
//! {"ok": false, "status": 404, "error": {"code": "not_found", "detail": "…"}}
//! ```
//!
//! ## The detail is a sentence the athlete can read
//!
//! Every detail is a `status` string in the voice of `docs/contract/voice.md`:
//! no imperative, no verdict, no conclusion. `tests/voice.rs` runs every
//! detail the router can produce through the core's own lint, so a detail
//! that breaks the voice fails the build.
//!
//! ## Nothing here carries a credential
//!
//! An error never carries the Garmin key, the database key or a file path.
//! The Garmin client already redacts its response bodies, and the router
//! only ever puts its own sentences in a detail.

use serde::Serialize;

/// The kinds of failure the router reports.
///
/// The set is closed. A new kind is a contract change: the UI has a
/// sentence for each one, and a kind with no sentence is a generic error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// There is no plan to answer with.
    NoPlan,
    /// What is on this device is older than the window the caller asked for.
    StaleData,
    /// There are not enough comparable days behind the reading.
    NotEnoughHistory,
    /// Garmin refused the Personal API key.
    GarminRejected,
    /// The Garmin session ended; the caller may exchange the key once more.
    NeedsReconnect,
    /// There is no such thing.
    NotFound,
    /// The request could not be read.
    BadRequest,
    /// The phone is locked, and the data is behind the lock.
    ///
    /// The shell refuses the data routes on a locked screen before the
    /// core is asked for anything; this is the same refusal, seen from
    /// here: the database key is not available.
    Locked,
    /// The router itself could not answer.
    ///
    /// This one is a bug, not a business failure: it is the only kind that
    /// is not a fact about the athlete's data, and it exists so that a
    /// failing store cannot be reported as something it is not.
    Internal,
}

impl ErrorKind {
    /// The code the failure travels under.
    ///
    /// The codes are the contract's names, spelled in lower case with an
    /// underscore: `no_plan`, `stale_data`, `not_enough_history`,
    /// `garmin_rejected`, `needs_reconnect`, `not_found`, `bad_request`.
    pub fn code(self) -> &'static str {
        match self {
            ErrorKind::NoPlan => "no_plan",
            ErrorKind::StaleData => "stale_data",
            ErrorKind::NotEnoughHistory => "not_enough_history",
            ErrorKind::GarminRejected => "garmin_rejected",
            ErrorKind::NeedsReconnect => "needs_reconnect",
            ErrorKind::NotFound => "not_found",
            ErrorKind::BadRequest => "bad_request",
            ErrorKind::Locked => "locked",
            ErrorKind::Internal => "internal",
        }
    }

    /// The HTTP status the failure travels under.
    pub fn status(self) -> u16 {
        match self {
            ErrorKind::NoPlan => 404,
            ErrorKind::StaleData => 409,
            ErrorKind::NotEnoughHistory => 409,
            ErrorKind::GarminRejected => 401,
            ErrorKind::NeedsReconnect => 401,
            ErrorKind::NotFound => 404,
            ErrorKind::BadRequest => 400,
            ErrorKind::Locked => 403,
            ErrorKind::Internal => 500,
        }
    }
}

/// One failure, with the sentence the UI shows.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{detail}")]
pub struct ApiError {
    kind: ErrorKind,
    detail: String,
}

impl ApiError {
    /// A failure of a kind, with its own sentence.
    pub fn new(kind: ErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    /// There is no plan to answer with.
    pub fn no_plan(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::NoPlan, detail)
    }

    /// What is here is older than the window.
    pub fn stale_data(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::StaleData, detail)
    }

    /// There are not enough comparable days.
    pub fn not_enough_history(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotEnoughHistory, detail)
    }

    /// Garmin refused the key.
    pub fn garmin_rejected(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::GarminRejected, detail)
    }

    /// The Garmin session ended.
    pub fn needs_reconnect(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::NeedsReconnect, detail)
    }

    /// There is no such thing.
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, detail)
    }

    /// The request could not be read.
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::BadRequest, detail)
    }

    /// The data is behind a lock.
    pub fn locked(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::Locked, detail)
    }

    /// The router could not answer.
    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, detail)
    }

    /// The kind of the failure.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The code the failure travels under.
    pub fn code(&self) -> &'static str {
        self.kind.code()
    }

    /// The HTTP status the failure travels under.
    pub fn status(&self) -> u16 {
        self.kind.status()
    }

    /// The sentence the UI shows.
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// The failure as it travels in the envelope.
    pub fn envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope {
            code: self.code(),
            detail: self.detail.clone(),
        }
    }
}

/// The failure as it travels in the envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ErrorEnvelope {
    /// The kind of the failure, as the contract names it.
    pub code: &'static str,
    /// The sentence the UI shows.
    pub detail: String,
}

/// The result of a handler.
pub type ApiResult<T> = Result<T, ApiError>;
