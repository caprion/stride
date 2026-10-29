//! The HTTP layer of the desktop.
//!
//! One listener, on the loopback interface, for one browser. The server is
//! deliberately small: it speaks HTTP/1.1 well enough for a browser and a
//! `fetch`, and it speaks nothing else.
//!
//! ## Why it is hand-written
//!
//! The approved dependency set has no HTTP server in it, and adding one
//! would add a crate — and a TLS stack, or an async runtime, or both — to a
//! binary that is meant to be an app rather than a service. The desktop
//! serves exactly three kinds of request: a `GET` for a file, a `GET` or
//! `POST` for the API, and nothing else. That is a few hundred lines.
//!
//! ## What it guarantees
//!
//! * A request head is bounded ([`MAX_HEAD_BYTES`]) and a body is bounded
//!   ([`MAX_BODY_BYTES`]), so a malformed or hostile request cannot make the
//!   process allocate without limit.
//! * Every connection is read and written with a timeout, so a browser that
//!   opens a socket and says nothing does not hold a thread for ever.
//! * The number of live connections is bounded, so a loopback flood cannot
//!   spawn threads without limit.
//! * The answer always carries a `Content-Length`, so the browser knows when
//!   the body ends without guessing.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// The largest request head (the request line and the headers) the server reads.
pub const MAX_HEAD_BYTES: usize = 64 * 1024;

/// The largest request body the server reads. The FIT import is the biggest
/// thing a browser posts, and a FIT file is a few megabytes.
pub const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

/// The most connections the server serves at once.
pub const MAX_CONNECTIONS: usize = 64;

/// How long a connection may stay silent before it is dropped.
pub const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// The headers of a request.
#[derive(Debug, Clone, Default)]
pub struct Headers {
    entries: Vec<(String, String)>,
}

impl Headers {
    /// An empty set of headers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a header.
    pub fn push(&mut self, name: &str, value: &str) {
        self.entries.push((name.to_string(), value.to_string()));
    }

    /// The first value of a header, matched without regard to case.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// One request.
#[derive(Debug, Clone)]
pub struct Request {
    /// The method, upper-cased.
    pub method: String,
    /// The path, percent-decoded — the way Starlette hands it to a route.
    pub path: String,
    /// The query string, exactly as it arrived. The router decodes it.
    pub query: String,
    /// The headers.
    pub headers: Headers,
    /// The body.
    pub body: Vec<u8>,
    /// The address the request came from.
    pub peer: IpAddr,
}

impl Request {
    /// The `Host` header.
    pub fn host(&self) -> &str {
        self.headers.get("host").unwrap_or("")
    }

    /// The value of one cookie, if the request carries it.
    pub fn cookie(&self, name: &str) -> Option<&str> {
        let header = self.headers.get("cookie")?;
        for part in header.split(';') {
            let part = part.trim();
            let Some((key, value)) = part.split_once('=') else {
                continue;
            };
            if key.trim() == name {
                return Some(value.trim());
            }
        }
        None
    }

    /// The body as a string, if it is one.
    pub fn body_text(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}

/// One answer.
#[derive(Debug, Clone)]
pub struct Response {
    /// The status code.
    pub status: u16,
    /// The headers, in the order they are written.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

impl Response {
    /// An answer with a body and a content type.
    pub fn with_body(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".to_string(), content_type.to_string())],
            body,
        }
    }

    /// An answer with no body at all.
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Add a header.
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    /// The value of a header, if it is set.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// The reason phrase for a status code.
pub fn reason(status: u16) -> &'static str {
    match status {
        100 => "Continue",
        200 => "OK",
        204 => "No Content",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        411 => "Length Required",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

/// Decode a percent-encoded string.
///
/// `+` is left alone: this decodes a *path*, and a plus sign in a path is a
/// plus sign. The router decodes the query string, where `+` does mean a
/// space.
pub fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = &raw[index + 1..index + 3];
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// What the handler is: a function from a request to an answer.
pub type Handler = Arc<dyn Fn(&Request) -> Response + Send + Sync>;

/// Serve until the listener fails.
///
/// Every connection gets a thread. The desktop serves one browser, so the
/// number of connections is small and bounded.
pub fn serve(listener: TcpListener, handler: Handler) -> std::io::Result<()> {
    serve_with(listener, handler, |stream: TcpStream| Some(stream))
}

/// Serve a listener whose connections are made by `accept`.
///
/// The TLS listener uses this: `accept` performs the handshake and hands back
/// the encrypted stream. The handshake happens on the connection's own
/// thread, so a client that says nothing cannot hold up the listener.
pub fn serve_with<S, A>(listener: TcpListener, handler: Handler, accept: A) -> std::io::Result<()>
where
    S: Read + Write + Send + 'static,
    A: Fn(TcpStream) -> Option<S> + Send + Sync + 'static,
{
    let live = Arc::new(AtomicUsize::new(0));
    let accept = Arc::new(accept);
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("stride: could not accept a connection: {error}");
                continue;
            }
        };
        let peer = match stream.peer_addr() {
            Ok(peer) => peer.ip(),
            Err(_) => continue,
        };
        let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
        let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
        let _ = stream.set_nodelay(true);
        if live.load(Ordering::Relaxed) >= MAX_CONNECTIONS {
            // Too many connections: answer the browser and let it retry.
            let mut stream = stream;
            let _ = write_response(
                &mut stream,
                &Response::with_body(
                    503,
                    "application/json",
                    br#"{"detail":"Too many connections; retry shortly."}"#.to_vec(),
                ),
                false,
            );
            continue;
        }
        live.fetch_add(1, Ordering::Relaxed);
        let handler = Arc::clone(&handler);
        let accept = Arc::clone(&accept);
        let counter = Arc::clone(&live);
        let spawned = thread::Builder::new()
            .name("stride-http".to_string())
            .spawn(move || {
                if let Some(stream) = accept(stream) {
                    serve_connection(stream, peer, handler);
                }
                counter.fetch_sub(1, Ordering::Relaxed);
            });
        if spawned.is_err() {
            live.fetch_sub(1, Ordering::Relaxed);
        }
    }
    Ok(())
}

/// Serve one connection.
///
/// The stream is generic because the same code serves the plain listener and
/// the TLS one: a TLS stream is a stream.
fn serve_connection<S: Read + Write>(stream: S, peer: IpAddr, handler: Handler) {
    let mut reader = BufReader::new(stream);
    let request = match read_request(&mut reader, peer) {
        Ok(Some(request)) => request,
        Ok(None) => return,
        Err(response) => {
            let _ = write_response(reader.get_mut(), &response, false);
            return;
        }
    };

    let response = handler(&request);
    // A `HEAD` answer carries the headers of the `GET` answer and no body.
    let head_only = request.method == "HEAD";
    // The request has been read; whatever is left in the buffer is of no use
    // to anybody, because the connection closes after this answer.
    let mut stream = reader.into_inner();
    let _ = write_response(&mut stream, &response, head_only);
}

/// Read one request. `Ok(None)` means the connection ended cleanly.
fn read_request<R: Read + Write>(
    reader: &mut BufReader<R>,
    peer: IpAddr,
) -> Result<Option<Request>, Response> {
    let mut head = Vec::new();
    loop {
        let mut line = Vec::new();
        match read_line(reader, &mut line) {
            Ok(0) => {
                if head.is_empty() {
                    return Ok(None);
                }
                return Err(bad_request("the request ended in the middle of a head"));
            }
            Ok(_) => {}
            Err(error) => return Err(bad_request(&format!("could not read the request: {error}"))),
        }
        let blank = line == b"\r\n" || line == b"\n";
        head.extend_from_slice(&line);
        if head.len() > MAX_HEAD_BYTES {
            return Err(bad_request("the request head is too large"));
        }
        if blank {
            break;
        }
    }

    let text = String::from_utf8_lossy(&head).into_owned();
    let mut lines = text.split("\r\n").flat_map(|part| part.split('\n'));
    let request_line = lines.next().unwrap_or_default().trim_end();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_uppercase();
    let target = parts.next().unwrap_or_default().to_string();
    if method.is_empty() || target.is_empty() {
        return Err(bad_request("the request line is not a request line"));
    }

    let mut headers = Headers::new();
    for line in lines {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(bad_request("a header line has no colon"));
        };
        headers.push(name.trim(), value.trim());
    }

    // A browser that expects a `100 Continue` waits for the interim answer
    // before it sends the body.
    if headers
        .get("expect")
        .is_some_and(|value| value.eq_ignore_ascii_case("100-continue"))
    {
        let stream = reader.get_mut();
        let _ = stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
        let _ = stream.flush();
    }

    if headers
        .get("transfer-encoding")
        .is_some_and(|value| value.to_ascii_lowercase().contains("chunked"))
    {
        return Err(bad_request(
            "chunked request bodies are not supported; send a Content-Length",
        ));
    }

    let length: usize = match headers.get("content-length") {
        Some(raw) => match raw.trim().parse() {
            Ok(length) => length,
            Err(_) => return Err(bad_request("the Content-Length is not a number")),
        },
        None => 0,
    };
    if length > MAX_BODY_BYTES {
        return Err(Response::with_body(
            413,
            "application/json",
            br#"{"detail":"The request body is too large."}"#.to_vec(),
        ));
    }
    let mut body = vec![0u8; length];
    if length > 0 && reader.read_exact(&mut body).is_err() {
        return Err(bad_request("the request body ended early"));
    }

    let (raw_path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), query.to_string()),
        None => (target.clone(), String::new()),
    };
    let path = percent_decode(&raw_path);

    Ok(Some(Request {
        method,
        path,
        query,
        headers,
        body,
        peer,
    }))
}

/// Read one line, with the length it took.
fn read_line<R: Read>(reader: &mut BufReader<R>, out: &mut Vec<u8>) -> std::io::Result<usize> {
    let mut total = 0;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(total);
        }
        match available.iter().position(|byte| *byte == b'\n') {
            Some(index) => {
                out.extend_from_slice(&available[..=index]);
                reader.consume(index + 1);
                total += index + 1;
                return Ok(total);
            }
            None => {
                let length = available.len();
                out.extend_from_slice(available);
                reader.consume(length);
                total += length;
                if total > MAX_HEAD_BYTES {
                    return Ok(total);
                }
            }
        }
    }
}

/// The answer to a request the server could not read.
fn bad_request(detail: &str) -> Response {
    let body = serde_json::json!({ "detail": detail }).to_string();
    Response::with_body(400, "application/json", body.into_bytes())
}

/// Write an answer.
fn write_response<T: Write>(
    stream: &mut T,
    response: &Response,
    head_only: bool,
) -> std::io::Result<()> {
    let mut head = String::with_capacity(256);
    head.push_str(&format!(
        "HTTP/1.1 {} {}\r\n",
        response.status,
        reason(response.status)
    ));
    for (name, value) in &response.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if response.status != 204 {
        head.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    }
    head.push_str("Connection: close\r\n\r\n");
    stream.write_all(head.as_bytes())?;
    if response.status != 204 && !head_only && !response.body.is_empty() {
        stream.write_all(&response.body)?;
    }
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_decoded_but_a_plus_is_a_plus() {
        assert_eq!(percent_decode("/assets/a%20b.js"), "/assets/a b.js");
        assert_eq!(percent_decode("/a+b"), "/a+b");
        assert_eq!(percent_decode("/%2e%2e/secret"), "/../secret");
    }

    #[test]
    fn a_cookie_is_found() {
        let mut headers = Headers::new();
        headers.push("Cookie", "stride_capability=abc123; other=1");
        let request = Request {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: String::new(),
            headers,
            body: Vec::new(),
            peer: "127.0.0.1".parse().unwrap_or(IpAddr::from([127, 0, 0, 1])),
        };
        assert_eq!(request.cookie("stride_capability"), Some("abc123"));
        assert_eq!(request.cookie("missing"), None);
    }
}
