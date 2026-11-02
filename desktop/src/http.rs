//! HTTP/1.1, written out.
//!
//! The rig serves one app and one router, to one browser, over loopback.
//! That is a small enough surface to write out by hand, and writing it out
//! keeps the dependency list short: the approved list has no server
//! framework on it, and a framework would be a bigger thing than the thing
//! it serves.
//!
//! What is here is the wire format and nothing else: how a request is read,
//! how a response is written, and the two limits that keep a mistake from
//! becoming a memory problem. The decision about *what* a request means
//! belongs to [`crate::server`].
//!
//! ## The limits
//!
//! * a request head is at most [`MAX_HEAD_BYTES`];
//! * an ordinary request body is at most [`MAX_BODY_BYTES`];
//! * a FIT file and an uploaded export have their own bounded limits.
//!
//! A body that is too large is refused and the connection is closed: the
//! rig does not read a body it has no use for.

use std::io::{BufRead, Write};

use crate::error::{Error, Result};

/// The largest request head the rig will read.
pub const MAX_HEAD_BYTES: usize = 32 * 1024;

/// The largest ordinary request body the rig will read.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// The largest raw FIT file the rig will read.
pub const MAX_FIT_BODY_BYTES: usize = 20 * 1024 * 1024;

/// The largest base64 FIT request the API will read.
pub const MAX_FIT_ENCODED_BODY_BYTES: usize = 28 * 1024 * 1024;

/// The largest file in an uploaded export the rig will stage.
pub const MAX_UPLOAD_BODY_BYTES: usize = 128 * 1024 * 1024;

/// A request, as the rig read it.
#[derive(Debug, Clone, Default)]
pub struct Request {
    /// The method: `GET`, `POST`, and so on.
    pub method: String,
    /// The target, as it arrived.
    pub target: String,
    /// The path, decoded, without the query string.
    pub path: String,
    /// The query string, without the `?`.
    pub query: String,
    /// The headers, in the order they arrived.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

impl Request {
    /// The value of a header, matched without regard to case.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Does the client want the connection kept open?
    pub fn wants_keep_alive(&self) -> bool {
        match self.header("connection") {
            Some(value) => !value.eq_ignore_ascii_case("close"),
            None => true,
        }
    }
}

/// A response, as the rig will write it.
#[derive(Debug, Clone, Default)]
pub struct Response {
    /// The status code.
    pub status: u16,
    /// The headers.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

impl Response {
    /// A response with a status and nothing else.
    pub fn with_status(status: u16) -> Response {
        Response {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// A response with a header on it.
    pub fn with_header(mut self, name: &str, value: &str) -> Response {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    /// A response with a body of text.
    pub fn text(status: u16, body: &str) -> Response {
        Response {
            status,
            headers: vec![(
                "content-type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )],
            body: body.as_bytes().to_vec(),
        }
    }

    /// A response with a body of HTML.
    pub fn html(status: u16, body: &str) -> Response {
        Response {
            status,
            headers: vec![(
                "content-type".to_string(),
                "text/html; charset=utf-8".to_string(),
            )],
            body: body.as_bytes().to_vec(),
        }
    }

    /// A response with a body of JSON.
    pub fn json(status: u16, body: &str) -> Response {
        Response {
            status,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: body.as_bytes().to_vec(),
        }
    }

    /// A redirect to another path on this rig.
    pub fn redirect(status: u16, location: &str) -> Response {
        Response {
            status,
            headers: vec![("location".to_string(), location.to_string())],
            body: Vec::new(),
        }
    }

    /// A response with a body of bytes.
    pub fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Response {
        Response {
            status,
            headers: vec![("content-type".to_string(), content_type.to_string())],
            body,
        }
    }

    /// Write the response out.
    ///
    /// `with_body` is false for a `HEAD` request: the head is the same, and
    /// the body is left out.
    pub fn write_to(
        &self,
        out: &mut impl Write,
        keep_alive: bool,
        with_body: bool,
    ) -> std::io::Result<()> {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status, reason(self.status));
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str(&format!("content-length: {}\r\n", self.body.len()));
        head.push_str(if keep_alive {
            "connection: keep-alive\r\n"
        } else {
            "connection: close\r\n"
        });
        head.push_str("\r\n");
        out.write_all(head.as_bytes())?;
        if with_body {
            out.write_all(&self.body)?;
        }
        out.flush()
    }
}

/// The reason phrase for a status.
fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        304 => "Not Modified",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        411 => "Length Required",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}

/// Read one request from a connection.
///
/// `Ok(None)` means the connection was closed before anything arrived: the
/// browser did that, and it is not an error.
pub fn read_request(reader: &mut impl BufRead) -> Result<Option<Request>> {
    let line = match read_line(reader, MAX_HEAD_BYTES)? {
        Some(line) => line,
        None => return Ok(None),
    };
    if line.trim().is_empty() {
        // A blank first line: some clients send one before a request. Skip
        // it and read the next line.
        return match read_line(reader, MAX_HEAD_BYTES)? {
            Some(line) if !line.trim().is_empty() => read_request_after_line(reader, line),
            _ => Ok(None),
        };
    }
    read_request_after_line(reader, line)
}

/// Read a request whose first line has been read.
fn read_request_after_line(reader: &mut impl BufRead, line: String) -> Result<Option<Request>> {
    let mut pieces = line.split_whitespace();
    let method = pieces.next().unwrap_or_default().to_string();
    let target = pieces.next().unwrap_or_default().to_string();
    if method.is_empty() || target.is_empty() {
        return Err(Error::http("the request line is not a request line"));
    }

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut head_bytes = line.len();
    loop {
        let header_line = match read_line(reader, MAX_HEAD_BYTES)? {
            Some(header_line) => header_line,
            None => break,
        };
        head_bytes += header_line.len();
        if head_bytes > MAX_HEAD_BYTES {
            return Err(Error::http("the request head is too long"));
        }
        let trimmed = header_line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        let (name, value) = match trimmed.split_once(':') {
            Some((name, value)) => (name.trim().to_string(), value.trim().to_string()),
            None => return Err(Error::http("a header line has no colon in it")),
        };
        headers.push((name, value));
    }

    let (path, query) = split_target(&target)?;
    let body = read_body(reader, &headers, body_limit_for(&path))?;

    Ok(Some(Request {
        method,
        target,
        path,
        query,
        headers,
        body,
    }))
}

/// The path and the query of a request target.
fn split_target(target: &str) -> Result<(String, String)> {
    if !target.starts_with('/') {
        return Err(Error::http("the request target is not a path"));
    }
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path, query),
        None => (target, ""),
    };
    let decoded = percent_decode(path);
    if decoded.contains('\0') {
        return Err(Error::http("the request path holds a null byte"));
    }
    Ok((decoded, percent_decode(query)))
}

/// Decode the `%xx` escapes in a path.
pub fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = (bytes[index + 1] as char).to_digit(16);
            let low = (bytes[index + 2] as char).to_digit(16);
            if let (Some(high), Some(low)) = (high, low) {
                out.push(((high << 4) | low) as u8);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The body limit for one path.
///
/// Ordinary API requests stay small. A FIT import and an uploaded export
/// have their own, bounded limits; no other path can ask for more than a
/// megabyte.
pub fn body_limit_for(path: &str) -> usize {
    match path {
        "/rig/fit-import" => MAX_FIT_BODY_BYTES,
        "/api/v1/auth/garmin/fit-import" => MAX_FIT_ENCODED_BODY_BYTES,
        "/rig/import/file" => MAX_UPLOAD_BODY_BYTES,
        _ => MAX_BODY_BYTES,
    }
}

/// Read the body of a request.
fn read_body(
    reader: &mut impl BufRead,
    headers: &[(String, String)],
    limit: usize,
) -> Result<Vec<u8>> {
    let transfer = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("transfer-encoding"))
        .map(|(_, value)| value.to_ascii_lowercase());
    if let Some(encoding) = transfer {
        if encoding.contains("chunked") {
            return Err(Error::http(
                "the rig does not read chunked request bodies; send the length instead",
            ));
        }
    }

    let length = match headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
    {
        Some((_, value)) => value
            .trim()
            .parse::<usize>()
            .map_err(|_| Error::http("the content length is not a number"))?,
        None => 0,
    };
    if length > limit {
        return Err(Error::http("the request body is too large"));
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|source| Error::io(std::path::PathBuf::from("the request body"), source))?;
    }
    Ok(body)
}

/// Read one line, up to a limit.
fn read_line(reader: &mut impl BufRead, limit: usize) -> Result<Option<String>> {
    let mut line = Vec::new();
    let read = reader
        .read_until(b'\n', &mut line)
        .map_err(|source| Error::io(std::path::PathBuf::from("the request"), source))?;
    if read == 0 {
        return Ok(None);
    }
    if line.len() > limit {
        return Err(Error::http("the request head is too long"));
    }
    Ok(Some(String::from_utf8_lossy(&line).into_owned()))
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    fn read(text: &str) -> Result<Option<Request>> {
        let mut reader = BufReader::new(text.as_bytes());
        read_request(&mut reader)
    }

    #[test]
    fn a_get_is_read() -> Result<()> {
        let request = read("GET /runs?limit=10 HTTP/1.1\r\nHost: localhost\r\n\r\n")?;
        let request = match request {
            Some(request) => request,
            None => return Err(Error::http("no request")),
        };
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/runs");
        assert_eq!(request.query, "limit=10");
        Ok(())
    }

    #[test]
    fn a_body_is_read() -> Result<()> {
        let request = read(
            "POST /api/v1/auth/garmin/personal-key HTTP/1.1\r\ncontent-length: 9\r\n\r\n{\"key\":1}",
        )?;
        let request = match request {
            Some(request) => request,
            None => return Err(Error::http("no request")),
        };
        assert_eq!(request.body.len(), 9);
        Ok(())
    }

    #[test]
    fn a_closed_connection_is_not_an_error() -> Result<()> {
        assert!(read("")?.is_none());
        Ok(())
    }

    #[test]
    fn an_escaped_path_is_decoded() {
        assert_eq!(percent_decode("/a%20b"), "/a b");
        assert_eq!(percent_decode("/a%2Fb"), "/a/b");
    }
}
