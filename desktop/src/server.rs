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
//! 2. **the rig's own pages** — the standalone first-run page and the local
//!    export/import tools live outside the frozen API prefix;
//! 3. **the router** — a request for a route goes to `stride-api`'s router,
//!    and the rig passes the answer on **byte for byte**: the rig does not
//!    re-shape a response, add a field to one, or turn a number into a
//!    string;
//! 4. **the app** — everything else is the app's own routing, and the app's
//!    bundle answers it.
//!
//! The trust gate comes first because it is the only step that can say no.
//!
//! ## The answer
//!
//! **The answer is the body, and the status is the status** — the contract's
//! rule (`docs/contract/api-paths.md`, "The shape of an answer"). The
//! router's body goes on the wire as it is, with the router's status in the
//! status line. There is no envelope, and the rig does not add one.
//!
//! ## The path
//!
//! The app asks for `/api/v1/...`, and the rig hands the router **the path
//! the app asked for** — the prefix included, the query string aside. The
//! rig decides one thing about a path: whether it is for the router or for
//! the app. Which *route* it names is the router's business, and only the
//! router's.
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
use crate::paths;
use crate::spa::Spa;
use crate::tools;
use crate::trust::{Decision, Trust};
use crate::workspace::Workspace;

/// How long a connection may sit idle before the rig gives up on it.
pub const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// How long the rig will wait to write a response.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// The rig's HTTP surface.
pub struct Server {
    trust: Trust,
    router: Arc<dyn Router>,
    spa: Spa,
    workspace: Option<Arc<Workspace>>,
}

impl Server {
    /// The rig.
    ///
    /// This constructor is for a host that only needs the router and the
    /// app. The real rig uses [`Server::with_workspace`], which adds the
    /// machine-side first-run and data tools.
    pub fn new(trust: Trust, router: Arc<dyn Router>, spa: Spa) -> Server {
        Server {
            trust,
            router,
            spa,
            workspace: None,
        }
    }

    /// The rig, with the machine-side services it owns.
    pub fn with_workspace(
        trust: Trust,
        router: Arc<dyn Router>,
        spa: Spa,
        workspace: Arc<Workspace>,
    ) -> Server {
        Server {
            trust,
            router,
            spa,
            workspace: Some(workspace),
        }
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
        if let Some(workspace) = &self.workspace {
            if let Some(response) = self.rig_dispatch(request, workspace) {
                return response;
            }
        }

        match api::route_path(&request.path) {
            Some(path) => {
                let answer = self.answer(request, path);
                if answer.status == 200 && path == "/api/v1/auth/garmin/personal-key" {
                    if let Some(workspace) = &self.workspace {
                        if let Err(error) = workspace.remember_connected() {
                            log::log(&format!(
                                "the first-run marker could not be written: {error}"
                            ));
                        }
                    }
                }
                // The router's answer is passed on as it is. The rig is a
                // host: it does not re-shape what the router said, and it
                // does not wrap it in an envelope.
                Response::bytes(answer.status, &answer.content_type, answer.body)
            }
            None => self.spa.serve(&request.path),
        }
    }

    /// The rig's own pages and tools.
    ///
    /// These live outside the frozen API prefix. The product's route table
    /// is unchanged; this is the development rig's first-run page and its
    /// local data tools.
    fn rig_dispatch(&self, request: &Request, workspace: &Workspace) -> Option<Response> {
        let method = request.method.to_ascii_uppercase();
        match (method.as_str(), request.path.as_str()) {
            ("GET", "/") if workspace.should_show_connect() => Some(connect_page()),
            ("GET", "/connect") => {
                if workspace.keys().is_set() {
                    Some(Response::redirect(302, "/"))
                } else {
                    Some(connect_page())
                }
            }
            ("GET", "/rig") => Some(
                Response::html(200, include_str!("../pages/tools.html"))
                    .with_header("cache-control", "no-store"),
            ),
            ("POST", "/rig/first-run/not-now") => Some(match workspace.remember_skipped() {
                Ok(()) => json_response(200, serde_json::json!({"ok": true})),
                Err(error) => rig_error(500, "first_run_failed", &error.to_string()),
            }),
            ("POST", "/rig/export") => Some(self.export(workspace)),
            ("POST", "/rig/import") => Some(self.import(workspace, request)),
            ("POST", "/rig/import/start") => Some(self.start_import(workspace)),
            ("POST", "/rig/import/file") => Some(self.accept_import_file(workspace, request)),
            ("POST", "/rig/import/finish") => Some(self.finish_import(workspace, request)),
            ("POST", "/rig/fit-import") => Some(self.fit_import(workspace, request)),
            (_, path) if path.starts_with("/rig/") => {
                Some(Response::text(404, "There is no rig tool at that path."))
            }
            _ => None,
        }
    }

    /// Write an export and describe where it is.
    fn export(&self, workspace: &Workspace) -> Response {
        match tools::export(workspace) {
            Ok(summary) => json_response(
                200,
                serde_json::json!({
                    "path": summary.path,
                    "rows": summary.rows,
                    "format_version": summary.format_version,
                    "schema_version": summary.schema_version,
                    "tables": summary.tables,
                    "contains": [
                        "runs",
                        "wellness days",
                        "plan blocks",
                        "planned sessions",
                        "notes",
                        "sync cursors"
                    ],
                    "message": "The export is private. Keep the folder off public storage and copy the whole folder over a private channel."
                }),
            ),
            Err(error) => rig_error(500, "export_failed", &error.to_string()),
        }
    }

    /// Read an export from a path.
    fn import(&self, workspace: &Workspace, request: &Request) -> Response {
        let Some(path) = body_path(request) else {
            return rig_error(400, "bad_request", "there is no export path in the request");
        };
        match tools::import(workspace, std::path::Path::new(&path)) {
            Ok(summary) => json_response(200, import_json(summary)),
            Err(error) => rig_error(400, "import_refused", &error.to_string()),
        }
    }

    /// Start an upload for an export directory.
    fn start_import(&self, workspace: &Workspace) -> Response {
        let root = paths::imports_directory(workspace.data_dir());
        match workspace.uploads().start(&root) {
            Ok(upload) => json_response(200, serde_json::json!({"upload_id": upload})),
            Err(error) => rig_error(500, "import_failed", &error.to_string()),
        }
    }

    /// Put one file into an upload.
    fn accept_import_file(&self, workspace: &Workspace, request: &Request) -> Response {
        let upload = query_value(&request.query, "upload").unwrap_or_default();
        let name = query_value(&request.query, "name").unwrap_or_default();
        match workspace.uploads().accept(&upload, &name, &request.body) {
            Ok(()) => json_response(200, serde_json::json!({"ok": true})),
            Err(error) => rig_error(400, "import_refused", &error.to_string()),
        }
    }

    /// Finish an upload and read the export it assembled.
    fn finish_import(&self, workspace: &Workspace, request: &Request) -> Response {
        let upload = query_value(&request.query, "upload").unwrap_or_default();
        let directory = match workspace.uploads().take(&upload) {
            Ok(directory) => directory,
            Err(error) => return rig_error(400, "import_refused", &error.to_string()),
        };
        let answer = tools::import(workspace, &directory);
        let _ = std::fs::remove_dir_all(&directory);
        match answer {
            Ok(summary) => json_response(200, import_json(summary)),
            Err(error) => rig_error(400, "import_refused", &error.to_string()),
        }
    }

    /// Send a FIT file through the API's import route.
    ///
    /// The rig does not decode a FIT file and does not own the dedupe rule.
    /// It calls the route, observes whether the route added a row, and asks
    /// the API for the run's own date and distance so the answer is the
    /// core's rendering.
    fn fit_import(&self, workspace: &Workspace, request: &Request) -> Response {
        if request.body.is_empty() {
            return rig_error(400, "bad_request", "there is no FIT file in the request");
        }
        let before = match workspace.store().count_activities() {
            Ok(count) => count,
            Err(error) => return rig_error(500, "import_failed", &error.to_string()),
        };
        let encoded = base64_encode(&request.body);
        let body = serde_json::to_vec(&serde_json::json!({"fit_base64": encoded}))
            .unwrap_or_else(|_| b"{}".to_vec());
        let call = api::Request {
            method: "POST",
            path: "/api/v1/auth/garmin/fit-import",
            query: "",
            body: &body,
        };
        let answer = self.router.call(&call);
        if answer.status != 200 {
            return Response::bytes(answer.status, &answer.content_type, answer.body);
        }
        let value: serde_json::Value = serde_json::from_slice(&answer.body).unwrap_or_default();
        let imported = value
            .get("imported")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let activity_id = value
            .get("activity_id")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default();
        let after = workspace.store().count_activities().unwrap_or(before);
        let duplicate = !imported || after <= before;
        let (mut date, mut distance) = self.activity_details(activity_id);
        if let Some(direct_date) = value.get("date").and_then(serde_json::Value::as_str) {
            date = direct_date.to_string();
        }
        if let Some(direct_distance) = value.get("distance") {
            distance = distance_text(direct_distance);
        }
        json_response(
            200,
            serde_json::json!({
                "imported": !duplicate,
                "duplicate": duplicate,
                "activity_id": activity_id,
                "date": date,
                "distance": distance,
            }),
        )
    }

    /// The date and distance of a run, as the API renders them.
    fn activity_details(&self, activity_id: i64) -> (String, String) {
        if activity_id <= 0 {
            return (String::new(), String::new());
        }
        let path = format!("/api/v1/activities/{activity_id}");
        let call = api::Request {
            method: "GET",
            path: &path,
            query: "",
            body: &[],
        };
        let answer = self.router.call(&call);
        if answer.status != 200 {
            return (String::new(), String::new());
        }
        let value: serde_json::Value = serde_json::from_slice(&answer.body).unwrap_or_default();
        let date = value
            .get("local_date")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let distance = value.get("distance").map(distance_text).unwrap_or_default();
        (date, distance)
    }

    /// Ask the router, and pass on what it says.
    ///
    /// **The rig owns the retry on a pull.** The client makes one token
    /// exchange and one attempt, and the router adds no retry of its own;
    /// the rig may make one more attempt, and only one more. See
    /// [`crate::sync`].
    fn answer(&self, request: &Request, path: &str) -> api::Response {
        let call = api::Request {
            method: &request.method,
            path,
            query: &request.query,
            body: &request.body,
        };
        if api::is_pull(&request.method, path) {
            crate::sync::pull(self.router.as_ref(), &call)
        } else {
            self.router.call(&call)
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
            workspace: self.workspace.clone(),
        }
    }
}

/// The first-run page.
fn connect_page() -> Response {
    Response::html(200, include_str!("../pages/connect.html"))
        .with_header("cache-control", "no-store")
}

/// A JSON answer.
fn json_response(status: u16, value: serde_json::Value) -> Response {
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
    Response::json(status, &body)
}

/// A JSON failure, without a stack trace.
fn rig_error(status: u16, code: &str, detail: &str) -> Response {
    json_response(status, serde_json::json!({"code": code, "detail": detail}))
}

/// The import report as JSON.
fn import_json(summary: tools::ImportSummary) -> serde_json::Value {
    serde_json::json!({
        "path": summary.path,
        "rows": summary.rows,
        "format_version": summary.format_version,
        "schema_version": summary.schema_version,
        "activities": summary.activities,
        "wellness": summary.wellness,
        "plan_blocks": summary.plan_blocks,
        "plan_sessions": summary.plan_sessions,
        "notes": summary.notes,
        "refused": summary.refused,
    })
}

/// A path in a JSON request body.
fn body_path(request: &Request) -> Option<String> {
    let text = std::str::from_utf8(&request.body).ok()?;
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let path = value.get("path")?.as_str()?.trim();
    if path.is_empty() {
        return None;
    }
    Some(path.to_string())
}

/// One value in a query string.
fn query_value(query: &str, name: &str) -> Option<String> {
    for piece in query.split('&') {
        let Some((key, value)) = piece.split_once('=') else {
            continue;
        };
        if key == name {
            return Some(value.to_string());
        }
    }
    None
}

/// The text of a distance, whether the API sent a string or a rendered
/// value.
fn distance_text(value: &serde_json::Value) -> String {
    value
        .as_str()
        .or_else(|| value.get("text").and_then(serde_json::Value::as_str))
        .unwrap_or_default()
        .to_string()
}

/// Base64-encode bytes, without a dependency.
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let packed = (u32::from(first) << 16) | (u32::from(second) << 8) | u32::from(third);
        out.push(ALPHABET[((packed >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((packed >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((packed >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(packed & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
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
    keys: std::sync::Arc<crate::key::KeyStore>,
}

impl DesktopHost {
    /// The host.
    pub fn new(
        store: std::sync::Arc<stride_store::Store>,
        keys: std::sync::Arc<crate::key::KeyStore>,
    ) -> DesktopHost {
        DesktopHost { store, keys }
    }
}

impl Host for DesktopHost {
    fn store(&self) -> std::sync::Arc<stride_store::Store> {
        Arc::clone(&self.store)
    }

    fn keys(&self) -> std::sync::Arc<crate::key::KeyStore> {
        Arc::clone(&self.keys)
    }
}
