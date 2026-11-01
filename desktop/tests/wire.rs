//! The rig over a socket.
//!
//! The trust model, the guard and the router are tested through the handler
//! — they are the rig's judgement, and the handler is where it happens.
//! This test is the other half: the wire format, end to end, over a real
//! connection, because a rig that cannot answer a request is a rig that
//! cannot be used.

mod common;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use stride_desktop::error::Result;
use stride_desktop::trust::Trust;

use common::server;

/// Ask the rig something, over a socket, and read the answer.
fn ask(address: std::net::SocketAddr, request: &str) -> Result<String> {
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(request.as_bytes())?;
    stream.flush()?;
    let mut answer = String::new();
    stream.read_to_string(&mut answer)?;
    Ok(answer)
}

#[test]
fn the_rig_answers_a_request_over_a_socket() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let rig = server(Trust::loopback_only());
    std::thread::spawn(move || {
        let _ = rig.serve(listener);
    });

    let answer = ask(
        address,
        "GET /api/v1/activities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )?;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.contains("the fake router"), "{answer}");
    Ok(())
}

#[test]
fn the_rig_answers_the_app_over_a_socket() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let rig = server(Trust::loopback_only());
    std::thread::spawn(move || {
        let _ = rig.serve(listener);
    });

    let answer = ask(
        address,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )?;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(
        answer
            .to_ascii_lowercase()
            .contains("content-type: text/html"),
        "{answer}"
    );
    Ok(())
}

#[test]
fn a_path_that_walks_upwards_is_refused_over_a_socket() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let rig = server(Trust::loopback_only());
    std::thread::spawn(move || {
        let _ = rig.serve(listener);
    });

    let answer = ask(
        address,
        "GET /../../etc/passwd HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )?;
    assert!(answer.starts_with("HTTP/1.1 400"), "{answer}");
    assert!(!answer.contains("root:"), "{answer}");
    Ok(())
}

#[test]
fn a_post_with_a_body_is_answered() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let rig = server(Trust::loopback_only());
    std::thread::spawn(move || {
        let _ = rig.serve(listener);
    });

    let answer = ask(
        address,
        "POST /api/v1/auth/garmin/personal-key HTTP/1.1\r\nHost: localhost\r\ncontent-length: 2\r\n\
         connection: close\r\n\r\n{}",
    )?;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    Ok(())
}
