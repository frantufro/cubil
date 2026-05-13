mod common;

use std::fs;
use std::process::Command;

use common::{MockServer, Route, TEST_TARGET_OVERRIDE, TEST_TRIPLE, bytes, json, make_tarball};

const CUBIL_VERSION: &str = env!("CARGO_PKG_VERSION");
const CUBIL_BIN: &str = env!("CARGO_BIN_EXE_cubil");

fn copy_cubil_to(dir: &std::path::Path) -> std::path::PathBuf {
    let dst = dir.join("cubil");
    fs::copy(CUBIL_BIN, &dst).expect("copy cubil binary");
    dst
}

fn download_path(version: &str, triple: &str) -> String {
    format!("/frantufro/cubil/releases/download/v{version}/cubil-{triple}.tar.gz")
}

#[test]
fn update_replaces_binary_with_latest_release() {
    let cache_dir = tempfile::tempdir().unwrap();
    let exe_dir = tempfile::tempdir().unwrap();
    let cubil_copy = copy_cubil_to(exe_dir.path());

    let tarball = make_tarball("cubil", b"NEW_BINARY_BYTES");
    let routes = vec![
        json(
            "/repos/frantufro/cubil/releases/latest",
            r#"{"tag_name":"v9.99.99"}"#,
        ),
        bytes(&download_path("9.99.99", TEST_TRIPLE), tarball),
    ];
    let server = MockServer::start(routes);

    let output = Command::new(&cubil_copy)
        .arg("update")
        .env("CUBIL_GITHUB_API_BASE", &server.url)
        .env("CUBIL_DOWNLOAD_BASE", &server.url)
        .env("CUBIL_CACHE_DIR", cache_dir.path())
        .env("CUBIL_TARGET_OVERRIDE", TEST_TARGET_OVERRIDE)
        .output()
        .expect("run cubil update");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Updated cubil to 9.99.99"),
        "stdout: {stdout}"
    );

    let after = fs::read(&cubil_copy).expect("read replaced binary");
    assert_eq!(after, b"NEW_BINARY_BYTES");
}

#[test]
fn update_already_latest_skips_download() {
    let cache_dir = tempfile::tempdir().unwrap();
    let exe_dir = tempfile::tempdir().unwrap();
    let cubil_copy = copy_cubil_to(exe_dir.path());
    let original_bytes = fs::read(&cubil_copy).unwrap();

    let server = MockServer::start(vec![json(
        "/repos/frantufro/cubil/releases/latest",
        &format!(r#"{{"tag_name":"v{CUBIL_VERSION}"}}"#),
    )]);

    let output = Command::new(&cubil_copy)
        .arg("update")
        .env("CUBIL_GITHUB_API_BASE", &server.url)
        .env("CUBIL_DOWNLOAD_BASE", &server.url)
        .env("CUBIL_CACHE_DIR", cache_dir.path())
        .env("CUBIL_TARGET_OVERRIDE", TEST_TARGET_OVERRIDE)
        .output()
        .expect("run cubil update");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("already up to date"),
        "stdout: {stdout}"
    );

    // Binary unchanged.
    assert_eq!(fs::read(&cubil_copy).unwrap(), original_bytes);

    // No download attempted.
    let reqs = server.requests();
    assert!(
        !reqs.iter().any(|p| p.contains("/releases/download/")),
        "unexpected download request: {reqs:?}"
    );
}

#[test]
fn update_errors_on_unsupported_target() {
    let cache_dir = tempfile::tempdir().unwrap();
    let server = MockServer::start(Vec::<Route>::new());

    let output = Command::new(CUBIL_BIN)
        .arg("update")
        .env("CUBIL_GITHUB_API_BASE", &server.url)
        .env("CUBIL_DOWNLOAD_BASE", &server.url)
        .env("CUBIL_CACHE_DIR", cache_dir.path())
        .env("CUBIL_TARGET_OVERRIDE", "x86_64:macos")
        .env("CUBIL_NO_UPDATE_CHECK", "1")
        .output()
        .expect("run cubil update");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("x86_64 macOS is not supported"),
        "stderr: {stderr}"
    );
}
