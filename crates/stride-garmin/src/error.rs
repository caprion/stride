//! The error type for the Garmin Personal API client.

/// Everything that can go wrong while talking to Garmin.
///
/// The variants carry the context needed to act on them: which endpoint, which
/// HTTP status, whether the athlete has to reconnect. The Personal API key
/// never appears in any of them — see [`crate::GarminClient`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GarminError {
    /// Garmin refused the Personal API key at the token exchange.
    ///
    /// The key is wrong, expired or revoked. `status` is the HTTP status
    /// Garmin returned (`401` or `403`).
    #[error("Garmin rejected the Personal API key (HTTP {status})")]
    InvalidKey {
        /// The HTTP status Garmin returned for the token request.
        status: u16,
    },

    /// The token exchange failed for a reason other than a rejected key.
    #[error("the Garmin token request failed (HTTP {status})")]
    TokenRequestFailed {
        /// The HTTP status Garmin returned for the token request.
        status: u16,
    },

    /// Garmin answered `403`: the key does not carry the scope this endpoint
    /// needs.
    ///
    /// A scope failure is not a token problem, so the client never re-exchanges
    /// the key for it and never retries.
    #[error("Garmin refused the request (HTTP 403): the key lacks the scope for this endpoint")]
    Forbidden,

    /// Garmin answered `401` twice: once with the cached token and once after
    /// a single re-exchange.
    ///
    /// The Personal API has no refresh tokens, so the only way forward is a new
    /// Personal API key. The caller should ask the athlete to reconnect.
    #[error("the Garmin session expired; the athlete must reconnect with a new Personal API key")]
    NeedsReconnect,

    /// Garmin answered `429`.
    #[error("Garmin is rate limiting the request (HTTP 429)")]
    RateLimited {
        /// The `Retry-After` header, in seconds, when Garmin sent one.
        retry_after_secs: Option<u64>,
    },

    /// The request never reached Garmin (DNS, TLS, timeout, reset).
    #[error("could not reach Garmin: {0}")]
    Transport(String),

    /// Garmin answered with an error status.
    #[error("Garmin returned HTTP {status} for {path}: {body}")]
    Http {
        /// The HTTP status Garmin returned.
        status: u16,
        /// The endpoint path, without the query string.
        path: String,
        /// The first 200 characters of the response body, with credentials
        /// redacted.
        body: String,
    },

    /// The response was larger than the 5 MiB ceiling this client enforces.
    #[error("the Garmin response for {path} exceeds the 5 MiB limit")]
    ResponseTooLarge {
        /// The endpoint path, without the query string.
        path: String,
    },

    /// Garmin sent a redirect without a usable `Location`.
    #[error("Garmin returned a redirect without a location for {path}")]
    Redirect {
        /// The endpoint path, without the query string.
        path: String,
    },

    /// Too many redirects.
    #[error("too many redirects while requesting {path}")]
    TooManyRedirects {
        /// The endpoint path, without the query string.
        path: String,
    },

    /// Garmin's response could not be understood.
    #[error("could not read Garmin's response for {path}: {reason}")]
    Decode {
        /// The endpoint path, without the query string.
        path: String,
        /// Why the response could not be read.
        reason: String,
    },

    /// The activity id is not a Garmin activity id.
    #[error("{0:?} is not a valid Garmin activity id")]
    InvalidActivityId(String),

    /// A date argument is not an ISO `YYYY-MM-DD` date.
    #[error("{0:?} is not a valid date; expected YYYY-MM-DD")]
    InvalidDate(String),

    /// The client refuses to send Garmin credentials to a host that is not
    /// Garmin.
    #[error("refusing to send Garmin credentials to {host}")]
    UntrustedHost {
        /// The host that was refused.
        host: String,
    },

    /// The download did not contain a usable FIT file.
    #[error("no usable FIT file in the download for activity {activity_id}")]
    FitUnavailable {
        /// The Garmin activity id whose download was unusable.
        activity_id: String,
    },

    /// The token endpoint answered, but with no token in it.
    #[error("Garmin returned an empty access token")]
    EmptyToken,
}
