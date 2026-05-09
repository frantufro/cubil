//! Test helpers for the update / stale-version-warning suite.
//!
//! Provides a tiny `TcpListener`-based mock server and a tar.gz builder so
//! tests can simulate the GitHub release surface without a network.

#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

pub fn json(path: &str, body: &str) -> Route {
    Route {
        path: path.to_string(),
        status: 200,
        content_type: "application/json",
        body: body.as_bytes().to_vec(),
    }
}

pub fn bytes(path: &str, body: Vec<u8>) -> Route {
    Route {
        path: path.to_string(),
        status: 200,
        content_type: "application/octet-stream",
        body,
    }
}

pub struct MockServer {
    pub url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
}

impl MockServer {
    pub fn start(routes: Vec<Route>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener
            .set_nonblocking(true)
            .expect("set_nonblocking");
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}");
        let requests: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let req_clone = Arc::clone(&requests);
        let stop_clone = Arc::clone(&stop);
        thread::spawn(move || {
            // Poll-loop: accept incoming connections until stop flag flips.
            // Non-blocking + sleep keeps the test from hanging on shutdown.
            while !stop_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let routes = routes.clone();
                        let req_clone = Arc::clone(&req_clone);
                        thread::spawn(move || {
                            let _ = handle_conn(stream, req_clone, routes);
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            url,
            requests,
            stop,
        }
    }

    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn handle_conn(
    stream: TcpStream,
    requests: Arc<Mutex<Vec<String>>>,
    routes: Vec<Route>,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .to_string();
    requests.lock().unwrap().push(path.clone());

    // Drain remaining headers
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }

    let mut writer = stream;
    if let Some(route) = routes.iter().find(|r| r.path == path) {
        let header = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            route.status,
            route.content_type,
            route.body.len()
        );
        writer.write_all(header.as_bytes())?;
        writer.write_all(&route.body)?;
    } else {
        let body = b"not found";
        let header = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        writer.write_all(header.as_bytes())?;
        writer.write_all(body)?;
    }
    writer.flush()?;
    Ok(())
}

/// Build an in-memory tar.gz containing a single file `name` with `contents`.
pub fn make_tarball(name: &str, contents: &[u8]) -> Vec<u8> {
    let buf = Vec::new();
    let mut gz = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
    {
        let mut tar = tar::Builder::new(&mut gz);
        let mut header = tar::Header::new_gnu();
        header.set_path(name).unwrap();
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append(&header, contents).unwrap();
        tar.finish().unwrap();
    }
    gz.finish().unwrap()
}

/// Find a TCP port that is currently closed (no listener) by binding then
/// dropping. Useful for tests that want connect-refused, fast.
pub fn closed_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

/// Stable triple used in tests when CUBIL_TARGET_OVERRIDE is set.
pub const TEST_TRIPLE: &str = "x86_64-unknown-linux-gnu";

/// Override env value matching TEST_TRIPLE.
pub const TEST_TARGET_OVERRIDE: &str = "x86_64:linux";
