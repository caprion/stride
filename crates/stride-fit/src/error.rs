//! Errors raised by the FIT importer.
//!
//! Every variant carries the context needed to explain what went wrong with a
//! file: which message, at which byte, which limit. A malformed FIT file is a
//! `Result`, never a crash — a phone that dies on a bad file is a phone that
//! dies.

use thiserror::Error;

/// The failure modes of [`crate::parse_fit`].
#[derive(Debug, Error)]
pub enum FitError {
    /// The bytes do not start with a FIT file header.
    #[error("not a FIT file: the '.FIT' header magic is missing")]
    NotFit,

    /// The file ended before a structure it declared was complete.
    #[error(
        "truncated FIT file: needed {needed} bytes at offset {offset}, only {available} available"
    )]
    Truncated {
        /// Byte offset where the read started.
        offset: usize,
        /// Number of bytes the structure needed.
        needed: usize,
        /// Number of bytes actually available.
        available: usize,
    },

    /// The header is structurally wrong.
    #[error("invalid FIT header: {reason}")]
    InvalidHeader {
        /// What is wrong with the header.
        reason: String,
    },

    /// A FIT version this importer does not know how to read.
    #[error("unsupported FIT version: {version}")]
    UnsupportedVersion {
        /// The protocol version found in the header.
        version: u16,
    },

    /// A message could not be decoded.
    #[error("malformed FIT message at byte {offset}: {reason}")]
    Malformed {
        /// Byte offset of the message.
        offset: usize,
        /// What is wrong with the message.
        reason: String,
    },

    /// A data message referred to a definition that was never sent.
    #[error(
        "malformed FIT file at byte {offset}: data message for unknown local message type {local}"
    )]
    UnknownLocalMessage {
        /// Byte offset of the message.
        offset: usize,
        /// The local message type that has no definition.
        local: u8,
    },

    /// The file carries more messages of one kind than the importer accepts.
    #[error("the FIT file has more than {limit} {kind} messages")]
    TooManyMessages {
        /// The kind of message that overflowed.
        kind: &'static str,
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The input is larger than the importer will accept.
    #[error("the FIT file is {size} bytes, over the {limit} byte limit")]
    TooLarge {
        /// The size of the input.
        size: usize,
        /// The limit that was exceeded.
        limit: usize,
    },
}
