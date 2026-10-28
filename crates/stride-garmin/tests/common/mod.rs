//! A tiny HTTP stub for the Garmin endpoints.
//!
//! The client is pointed at this instead of Garmin, so no test ever touches
//! the network. The stub records every request it serves and answers with
//! scripted replies, one queue per route.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// One request the stub served.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Request {
    /// The value of a header, case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// A scripted reply.
#[derive(Debug, Clone)]
pub enum Reply {
    /// A JSON body.
    Json(serde_json::Value),
    /// A text body.
    Text(String),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// An empty body with a status code.
    Empty(u16),
    /// A status code and a body.
    Status(u16, String),
}

impl Reply {
    /// A JSON reply.
    pub fn json(value: serde_json::Value) -> Self {
        Reply::Json(value)
    }

    /// A text reply.
    pub fn text(value: impl Into<String>) -> Self {
        Reply::Text(value.into())
    }

    /// A raw-bytes reply.
    pub fn bytes(value: Vec<u8>) -> Self {
        Reply::Bytes(value)
    }

    /// An empty reply with a status code.
    pub fn status(code: u16) -> Self {
        Reply::Empty(code)
    }

    /// A reply with a status code and a body.
    pub fn error(code: u16, body: impl Into<String>) -> Self {
        Reply::Status(code, body.into())
    }

    fn write(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let (code, content_type, body): (u16, &str, Vec<u8>) = match self {
            Reply::Json(value) => (
                200,
                "application/json",
                serde_json::to_vec(value).unwrap_or_default(),
            ),
            Reply::Text(text) => (200, "text/plain", text.as_bytes().to_vec()),
            Reply::Bytes(bytes) => (200, "application/octet-stream", bytes.clone()),
            Reply::Empty(code) => (*code, "text/plain", Vec::new()),
            Reply::Status(code, text) => (*code, "text/plain", text.as_bytes().to_vec()),
        };
        let head = format!(
            "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n",
            reason = reason(code),
            len = body.len(),
        );
        stream.write_all(head.as_bytes())?;
        stream.write_all(&body)?;
        stream.flush()
    }
}

fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        204 => "No Content",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Status",
    }
}

#[derive(Default)]
struct StubState {
    routes: HashMap<String, Vec<Reply>>,
    requests: Vec<Request>,
}

/// A local HTTP server that answers with scripted Garmin payloads.
pub struct StubServer {
    addr: SocketAddr,
    state: Arc<Mutex<StubState>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl StubServer {
    /// Starts the stub on a free loopback port.
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("the stub can bind a loopback port");
        listener
            .set_nonblocking(true)
            .expect("the stub can set non-blocking mode");
        let addr = listener.local_addr().expect("the stub has a local address");
        let state = Arc::new(Mutex::new(StubState::default()));
        let stop = Arc::new(AtomicBool::new(false));

        let server_state = Arc::clone(&state);
        let server_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !server_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let connection_state = Arc::clone(&server_state);
                        thread::spawn(move || {
                            let _ = serve(stream, connection_state);
                        });
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            addr,
            state,
            stop,
            handle: Some(handle),
        }
    }

    /// The base URL the client should be pointed at.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Queues a reply for a route. The queue is consumed in order.
    pub fn push(&self, method: &str, path: &str, reply: Reply) {
        let mut state = self.state.lock().expect("the stub state is not poisoned");
        state
            .routes
            .entry(route(method, path))
            .or_default()
            .push(reply);
    }

    /// Every request the stub has served.
    pub fn requests(&self) -> Vec<Request> {
        let state = self.state.lock().expect("the stub state is not poisoned");
        state.requests.clone()
    }

    /// The requests for one route.
    pub fn requests_for(&self, method: &str, path: &str) -> Vec<Request> {
        self.requests()
            .into_iter()
            .filter(|request| request.method == method && request.path == path)
            .collect()
    }

    /// How many times a route was requested.
    pub fn count(&self, method: &str, path: &str) -> usize {
        self.requests_for(method, path).len()
    }
}

impl Drop for StubServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn route(method: &str, path: &str) -> String {
    format!("{method} {path}")
}

fn serve(mut stream: TcpStream, state: Arc<Mutex<StubState>>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let target = parts.next().unwrap_or_default().to_owned();
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target.clone(), String::new()),
    };

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_owned();
            if name == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            headers.push((name, value));
        }
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let record = Request {
        method: method.clone(),
        path: path.clone(),
        query,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    };

    let reply = {
        let mut state = state.lock().expect("the stub state is not poisoned");
        state.requests.push(record);
        state
            .routes
            .get_mut(&route(&method, &path))
            .and_then(|queue| {
                if queue.is_empty() {
                    None
                } else {
                    Some(queue.remove(0))
                }
            })
            .unwrap_or(Reply::Status(
                500,
                format!("the stub has no scripted reply for {} {}", method, path),
            ))
    };

    reply.write(&mut stream)?;
    let _ = stream.shutdown(std::net::Shutdown::Both);
    Ok(())
}
