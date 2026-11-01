//! The rig's errors.
//!
//! One type, so the command line can print a sentence and stop. Every
//! variant carries a path or a reason — **never a key**. The Garmin key and
//! the database key are the two secrets the rig holds, and neither of them
//! can reach an error message: the key types have no `Display`, no accessor
//! and a fixed `Debug`, so there is nothing to print.

use std::path::PathBuf;

use thiserror::Error;

/// What went wrong.
#[derive(Debug, Error)]
pub enum Error {
    /// A file could not be read or written.
    #[error("could not use {path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// Why.
        #[source]
        source: std::io::Error,
    },

    /// The database said no.
    #[error(transparent)]
    Store(#[from] stride_store::StoreError),

    /// The rig was asked to do something it cannot, or was given something
    /// it cannot read.
    #[error("{0}")]
    Config(String),

    /// A key was refused: it is empty, it is in the wrong place, or it is in
    /// a file other people can read.
    #[error("{0}")]
    Key(String),

    /// A request could not be read.
    #[error("{0}")]
    Http(String),

    /// The TLS material could not be read, or the listener could not be
    /// opened.
    #[error("{0}")]
    Tls(String),
}

impl Error {
    /// An I/O error with the path it happened on.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
        Error::Io {
            path: path.into(),
            source,
        }
    }
    /// A configuration problem, in a sentence.
    pub fn config(message: impl Into<String>) -> Error {
        Error::Config(message.into())
    }

    /// A key problem, in a sentence.
    pub fn key(message: impl Into<String>) -> Error {
        Error::Key(message.into())
    }

    /// An HTTP problem, in a sentence.
    pub fn http(message: impl Into<String>) -> Error {
        Error::Http(message.into())
    }

    /// A TLS problem, in a sentence.
    pub fn tls(message: impl Into<String>) -> Error {
        Error::Tls(message.into())
    }
}

/// The rig's result.
pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    /// An I/O error whose path is not known here.
    ///
    /// The places that do know the path say so, with [`Error::io`]. This is
    /// for the places where the path would be noise.
    fn from(source: std::io::Error) -> Error {
        Error::Io {
            path: PathBuf::from("the file"),
            source,
        }
    }
}
