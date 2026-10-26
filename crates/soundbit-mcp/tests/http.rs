//! The localhost HTTP endpoint accepts native MCP clients and refuses browsers and foreign
//! hosts.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use soundbit_app::Session;
use soundbit_app::model::Song;
use soundbit_mcp::{HttpEndpoint, SoundBitServer};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#;

fn endpoint() -> (tempfile::TempDir, HttpEndpoint) {
    let dir = tempfile::tempdir().expect("temp dir");
    let session = Arc::new(Mutex::new(Session::new(Song::new("Shared"))));
    let server = SoundBitServer::shared(session, dir.path()).expect("valid workspace");
    let endpoint = HttpEndpoint::start(server, 0).expect("free port");
    (dir, endpoint)
}

/// Sends a raw HTTP/1.1 POST and returns the status line and body.
fn post(endpoint: &HttpEndpoint, host: &str, extra_headers: &str) -> (String, String) {
    let mut stream = TcpStream::connect(endpoint.address()).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let request = format!(
        "POST /mcp HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         Accept: application/json, text/event-stream\r\nContent-Length: {}\r\n\
         Connection: close\r\n{extra_headers}\r\n{INITIALIZE}",
        INITIALIZE.len()
    );
    stream.write_all(request.as_bytes()).expect("sends");
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    let status = response.lines().next().unwrap_or_default().to_owned();
    (status, response)
}

#[test]
fn native_clients_can_initialize() {
    let (_dir, endpoint) = endpoint();
    assert!(endpoint.url().starts_with("http://127.0.0.1:"));
    let host = endpoint.address().to_string();
    let (status, body) = post(&endpoint, &host, "");
    assert!(status.contains("200"), "{status}");
    assert!(body.contains("\"name\":\"soundbit\""), "{body}");
}

#[test]
fn browser_origins_are_refused() {
    let (_dir, endpoint) = endpoint();
    let host = endpoint.address().to_string();
    let (status, _) = post(&endpoint, &host, "Origin: https://evil.example\r\n");
    assert!(status.contains("403"), "{status}");
}

#[test]
fn foreign_hosts_are_refused() {
    let (_dir, endpoint) = endpoint();
    let (status, _) = post(&endpoint, "evil.example", "");
    assert!(!status.contains("200"), "{status}");
}
