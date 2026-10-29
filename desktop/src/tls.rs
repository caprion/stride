//! The HTTPS listener, for a phone's browser.
//!
//! A phone's browser needs HTTPS for one concrete reason: a service worker
//! only registers in a **secure context**, so on plain HTTP the browser
//! would refuse to install the app and the athlete would be left with a
//! browser tab instead of the app on the home screen.
//!
//! ## Nothing is detected
//!
//! The listener binds `STRIDE_TLS_HOST` and `STRIDE_TLS_PORT` — the
//! operator's configuration — and nothing else. No address is discovered at
//! startup, no command is run to work out where to listen, and no networking
//! product is named: the repository configures a network, it does not detect
//! one.
//!
//! The certificate is the operator's business too: point `STRIDE_TLS_DIR` at
//! the directory that holds `stride.crt` and `stride.key`. With no
//! certificate there is no listener, and the desktop says so.

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::{ServerConfig, ServerConnection};
use rustls::StreamOwned;

/// The certificate's file name.
pub const CERT_NAME: &str = "stride.crt";

/// The private key's file name.
pub const KEY_NAME: &str = "stride.key";

/// What can go wrong with the certificate.
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// A file could not be read.
    #[error("could not read {path}: {source}")]
    Read {
        /// The file.
        path: PathBuf,
        /// Why.
        #[source]
        source: std::io::Error,
    },
    /// The certificate holds nothing.
    #[error("{path} holds no certificate")]
    NoCertificate {
        /// The file.
        path: PathBuf,
    },
    /// The key holds nothing.
    #[error("{path} holds no private key")]
    NoPrivateKey {
        /// The file.
        path: PathBuf,
    },
    /// The PEM could not be read.
    #[error("{path} could not be read: {reason}")]
    Parse {
        /// The file.
        path: PathBuf,
        /// Why.
        reason: String,
    },
    /// The certificate and the key do not make a server.
    #[error("the certificate and the key do not make a server: {0}")]
    Config(String),
}

/// The server configuration, if there is a certificate to make one from.
///
/// `Ok(None)` means there is no certificate: the desktop then serves plain
/// HTTP only, and says so. The reference behaves the same way.
pub fn load(directory: &Path) -> Result<Option<ServerConfig>, TlsError> {
    let cert_path = directory.join(CERT_NAME);
    let key_path = directory.join(KEY_NAME);
    if !cert_path.is_file() || !key_path.is_file() {
        return Ok(None);
    }
    let certs = read_certs(&cert_path)?;
    let key = read_key(&key_path)?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| TlsError::Config(err.to_string()))?;
    Ok(Some(config))
}

/// The certificates in a PEM file.
fn read_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, TlsError> {
    let file = std::fs::File::open(path).map_err(|source| TlsError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = std::io::BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| TlsError::Parse {
            path: path.to_path_buf(),
            reason: err.to_string(),
        })?;
    if certs.is_empty() {
        return Err(TlsError::NoCertificate {
            path: path.to_path_buf(),
        });
    }
    Ok(certs)
}

/// The private key in a PEM file.
fn read_key(path: &Path) -> Result<PrivateKeyDer<'static>, TlsError> {
    let file = std::fs::File::open(path).map_err(|source| TlsError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = std::io::BufReader::new(file);
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|err| TlsError::Parse {
            path: path.to_path_buf(),
            reason: err.to_string(),
        })?
        .ok_or_else(|| TlsError::NoPrivateKey {
            path: path.to_path_buf(),
        })?;
    Ok(key)
}

/// The handshake, and the stream it leaves behind.
///
/// The handshake reads and writes the socket, so it goes through one handle on
/// it. The socket's read timeout applies, so a client that says nothing
/// during the handshake is dropped rather than holding the thread for ever.
pub fn accept(
    config: Arc<ServerConfig>,
    stream: TcpStream,
) -> Option<StreamOwned<ServerConnection, TcpStream>> {
    let mut connection = match ServerConnection::new(Arc::clone(&config)) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("stride: the TLS connection could not be made: {err}");
            return None;
        }
    };
    let mut stream = stream;
    while connection.is_handshaking() {
        if let Err(err) = connection.complete_io(&mut stream) {
            eprintln!("stride: the TLS handshake failed: {err}");
            return None;
        }
    }
    Some(StreamOwned::new(connection, stream))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory with no certificate is no listener.
    #[test]
    fn a_directory_with_no_certificate_is_no_listener() {
        let directory = std::env::temp_dir().join(format!("stride-tls-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("the directory");
        assert!(load(&directory).expect("the answer").is_none());
    }

    /// A certificate that is not a certificate is an error, not a listener.
    #[test]
    fn a_certificate_that_is_not_a_certificate_is_an_error() {
        let directory = std::env::temp_dir().join(format!("stride-tls-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("the directory");
        std::fs::write(directory.join(CERT_NAME), b"not a certificate").expect("the certificate");
        std::fs::write(directory.join(KEY_NAME), b"not a key").expect("the key");
        assert!(load(&directory).is_err());
    }
}
