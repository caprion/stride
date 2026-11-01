//! The HTTP surface.
//!
//! What the rig answers, and what it refuses.
//!
//! ## The order of things
//!
//! Every request goes through the same three steps, in this order:
//!
//! 1. **the trust gate** — the address the request came from decides
//!    whether it is answered at all;
//! 2. **the router** — a request for a route goes to `stride-api`'s router,
//!    and the rig passes the answer on **byte for byte**: the rig does not
//!    re-shape a response, add a field to one, or turn a number into a
//!    string;
//! 3. **the app** — everything else is the app's own routing, and the app's
//!    bundle answers it.
//!
//! The trust gate comes first because it is the only step that can say no.
//!
//! ## The listener
//!
//! The rig listens on the loopback by default. A listener on a network is
//! a listener that carries the Garmin key, so it is a TLS listener: the
//! certificate and the key are the athlete's, and the rig refuses to serve
//! a network in the clear.

use std::io::{BufReader, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rustls::{ServerConfig, ServerConnection, StreamOwned};

use crate::api::{self, Host, Router};
use crate::http::{self, Request, Response};
use crate::log;
use crate::spa::Spa;
use crate::trust::{Decision, Trust};

/// How long a connection may sit idle before the rig gives up on it.
pub const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// How long the rig will wait to write a response.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// The rig's HTTP surface.
pub struct Server {
    trust: Trust,
    router: Arc<dyn Router>,
    spa: Spa,
}

impl Server {
    /// The rig.
    pub fn new(trust: Trust, router: Arc<dyn Router>, spa: Spa) -> Server {
        Server { trust, router, spa }
    }

    /// The trust model the rig answers with.
    pub fn trust(&self) -> &Trust {
        &self.trust
    }

    /// Answer one request, from one peer.
    ///
    /// This is the whole of the rig's judgement, and it is a pure function
    /// of the request and the address it came from: the same request from
    /// the same address always gets the same answer.
    pub fn handle(&self, request: &Request, peer: IpAddr) -> Response {
        match self.trust.decide(peer) {
            Decision::Allow => self.dispatch(request),
            Decision::Refuse(refusal) => {
                log::log(&format!(
                    "refused a request from {}: it is not on the loopback and not in a trusted \
                     network",
                    refusal.peer()
                ));
                Response::text(403, &refusal.message())
            }
        }
    }

    /// Answer a request that the trust gate allowed.
    fn dispatch(&self, request: &Request) -> Response {
        match api::route_path(&request.path) {
            Some(path) => {
                let answer = self.router.call(&api::Request {
                    method: &request.method,
                    path: &path,
                    query: &request.query,
                    body: &request.body,
                });
                // The router's answer is passed on as it is. The rig is a
                // host: it does not re-shape what the router said.
                Response::bytes(answer.status, &answer.content_type, answer.body)
            }
            None => self.spa.serve(&request.path),
        }
    }

    /// Serve connections until the listener closes.
    pub fn serve(&self, listener: TcpListener) -> crate::error::Result<()> {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(error) => {
                    log::log(&format!("a connection could not be accepted: {error}"));
                    continue;
                }
            };
            let peer = peer_of(&stream);
            let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
            let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
            let server = self.clone_for_thread();
            std::thread::spawn(move || {
                if let Err(error) = server.serve_stream(stream, peer) {
                    log::log(&format!("a connection ended badly: {error}"));
                }
            });
        }
        Ok(())
    }

    /// Serve TLS connections until the listener closes.
    pub fn serve_tls(
        &self,
        listener: TcpListener,
        config: Arc<ServerConfig>,
    ) -> crate::error::Result<()> {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(error) => {
                    log::log(&format!("a connection could not be accepted: {error}"));
                    continue;
                }
            };
            let peer = peer_of(&stream);
            let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
            let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
            let connection = match ServerConnection::new(Arc::clone(&config)) {
                Ok(connection) => connection,
                Err(error) => {
                    log::log(&format!("a TLS connection could not be started: {error}"));
                    continue;
                }
            };
            let server = self.clone_for_thread();
            std::thread::spawn(move || {
                let stream = StreamOwned::new(connection, stream);
                if let Err(error) = server.serve_stream(stream, peer) {
                    log::log(&format!("a connection ended badly: {error}"));
                }
            });
        }
        Ok(())
    }

    /// Answer the requests on one connection.
    fn serve_stream<S: Read + Write>(&self, stream: S, peer: IpAddr) -> crate::error::Result<()> {
        let mut reader = BufReader::new(stream);
        loop {
            let request = match http::read_request(&mut reader) {
                Ok(Some(request)) => request,
                Ok(None) => return Ok(()),
                Err(error) => return Err(error),
            };
            let keep_alive = request.wants_keep_alive();
            let with_body = !request.method.eq_ignore_ascii_case("HEAD");
            self.log_request(&request);
            let response = self.handle(&request, peer);
            response
                .write_to(&mut reader.get_mut(), keep_alive, with_body)
                .map_err(|source| {
                    crate::error::Error::io(std::path::PathBuf::from("the connection"), source)
                })?;
            if !keep_alive {
                return Ok(());
            }
        }
    }

    /// Say what the rig is doing.
    ///
    /// The path is logged and the query string is not: a query string is
    /// where a secret ends up by accident, and the rig has no reason to
    /// keep one.
    fn log_request(&self, request: &Request) {
        log::log(&format!("{} {}", request.method, request.path));
    }

    /// A copy of the rig, for a thread of its own.
    fn clone_for_thread(&self) -> Server {
        Server {
            trust: self.trust.clone(),
            router: Arc::clone(&self.router),
            spa: self.spa.clone(),
        }
    }
}

/// The address a connection came from.
fn peer_of(stream: &TcpStream) -> IpAddr {
    match stream.peer_addr() {
        Ok(address) => address.ip(),
        Err(_) => IpAddr::from([0, 0, 0, 0]),
    }
}

/// The TLS material, read from a certificate and a key.
///
/// Both files are the athlete's: the rig does not make a certificate, and it
/// does not ask for one. A self-signed certificate the rig made for itself
/// would be a certificate nobody has to trust, and a certificate the athlete
/// made is one the phone can be told to trust.
pub fn tls_config(certificate: &Path, key: &Path) -> crate::error::Result<Arc<ServerConfig>> {
    use std::fs::File;
    use std::io::BufReader as IoBufReader;

    let certificates: Vec<rustls::pki_types::CertificateDer<'static>> =
        rustls_pemfile::certs(&mut IoBufReader::new(
            File::open(certificate)
                .map_err(|source| crate::error::Error::io(certificate, source))?,
        ))
        .collect::<std::result::Result<Vec<_>, std::io::Error>>()
        .map_err(|source| crate::error::Error::io(certificate, source))?;

    if certificates.is_empty() {
        return Err(crate::error::Error::tls(format!(
            "the certificate at {} holds no certificate",
            certificate.display()
        )));
    }

    let key_material = rustls_pemfile::private_key(&mut IoBufReader::new(
        File::open(key).map_err(|source| crate::error::Error::io(key, source))?,
    ))
    .map_err(|source| crate::error::Error::io(key, source))?
    .ok_or_else(|| {
        crate::error::Error::tls(format!("the key at {} holds no private key", key.display()))
    })?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, key_material)
        .map_err(|error| crate::error::Error::tls(error.to_string()))?;

    Ok(Arc::new(config))
}

/// Install the cryptography provider this build uses.
///
/// The rig uses `ring`, which is the provider the rest of the tree already
/// builds. It is installed once, and a second call is not an error.
pub fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// The host services the rig offers the router.
///
/// The database and the Garmin key: the two things the machine has and the
/// router may use. Nothing else crosses this line.
pub struct DesktopHost {
    store: std::sync::Arc<stride_store::Store>,
    keys: crate::key::KeyStore,
}

impl DesktopHost {
    /// The host.
    pub fn new(
        store: std::sync::Arc<stride_store::Store>,
        keys: crate::key::KeyStore,
    ) -> DesktopHost {
        DesktopHost { store, keys }
    }
}

impl Host for DesktopHost {
    fn store(&self) -> &stride_store::Store {
        &self.store
    }

    fn keys(&self) -> &crate::key::KeyStore {
        &self.keys
    }
}
