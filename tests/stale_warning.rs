mod common;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use common::{MockServer, closed_port, json};

const CUBIL_VERSION: &str = env!("CARGO_PKG_VERSION");
const CUBIL_BIN: &str = env!("CARGO_BIN_EXE_cubil");

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn write_cache(dir: &Path, latest: &str, age_secs: u64) {
    let entry = serde_json::json!({
        "checked_at": now_secs().saturating_sub(age_secs),
        "latest": latest,
    });
    fs::write(dir.join("latest.json"), entry.to_string()).unwrap();
}

/// Run `cubil init` in a tempdir so the command actually runs (and the stale
/// check has a chance to fire on stderr beforehand).
fn run_cubil_init(env_overrides: &[(&str, &str)]) -> std::process::Output {
    let work = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(CUBIL_BIN);
    cmd.arg("init").current_dir(work.path());
    for (k, v) in env_overrides {
        cmd.env(k, v);
    }
    cmd.output().expect("run cubil init")
}

#[test]
fn stale_warning_prints_when_cache_says_newer() {
    let cache_dir = tempfile::tempdir().unwrap();
    write_cache(cache_dir.path(), "9.99.99", 60);

    let output = run_cubil_init(&[
        ("CUBIL_CACHE_DIR", cache_dir.path().to_str().unwrap()),
    ]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "warning: cubil {CUBIL_VERSION} is out of date (latest: 9.99.99)"
        )),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("Run `cubil update` to upgrade."),
        "stderr: {stderr}"
    );
}

#[test]
fn no_warning_when_cache_matches_current() {
    let cache_dir = tempfile::tempdir().unwrap();
    write_cache(cache_dir.path(), CUBIL_VERSION, 60);

    let output = run_cubil_init(&[
        ("CUBIL_CACHE_DIR", cache_dir.path().to_str().unwrap()),
    ]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("warning: cubil"),
        "unexpected warning: {stderr}"
    );
}

#[test]
fn cubil_no_update_check_disables_warning() {
    let cache_dir = tempfile::tempdir().unwrap();
    write_cache(cache_dir.path(), "9.99.99", 60);

    let output = run_cubil_init(&[
        ("CUBIL_CACHE_DIR", cache_dir.path().to_str().unwrap()),
        ("CUBIL_NO_UPDATE_CHECK", "1"),
    ]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("warning: cubil"),
        "warning should be suppressed: {stderr}"
    );
}

#[test]
fn cache_ttl_respected_within_24h() {
    let cache_dir = tempfile::tempdir().unwrap();
    // Fresh cache (1h old), pointing at a newer version.
    write_cache(cache_dir.path(), "9.99.99", 60 * 60);

    // Server should NOT be hit. We point CUBIL_GITHUB_API_BASE at a closed
    // port so any fetch attempt would fail loudly (timeout/refused). With a
    // fresh cache the warning still prints from cache.
    let port = closed_port();
    let url = format!("http://127.0.0.1:{port}");

    let output = run_cubil_init(&[
        ("CUBIL_CACHE_DIR", cache_dir.path().to_str().unwrap()),
        ("CUBIL_GITHUB_API_BASE", &url),
    ]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("9.99.99"),
        "should print warning from fresh cache: {stderr}"
    );

    // Cache file's checked_at should be unchanged (no rewrite).
    let raw = fs::read_to_string(cache_dir.path().join("latest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let checked_at = parsed["checked_at"].as_u64().unwrap();
    let one_hour_ago = now_secs() - (60 * 60);
    let drift = checked_at.abs_diff(one_hour_ago);
    assert!(
        drift < 5,
        "cache was rewritten unexpectedly: checked_at={checked_at} expected≈{one_hour_ago}"
    );
}

#[test]
fn expired_cache_refetches_and_writes_new_entry() {
    let cache_dir = tempfile::tempdir().unwrap();
    // Stale cache (older than 24h).
    write_cache(cache_dir.path(), "0.0.1", 25 * 60 * 60);

    let server = MockServer::start(vec![json(
        "/repos/frantufro/cubil/releases/latest",
        r#"{"tag_name":"v9.99.99"}"#,
    )]);

    let output = run_cubil_init(&[
        ("CUBIL_CACHE_DIR", cache_dir.path().to_str().unwrap()),
        ("CUBIL_GITHUB_API_BASE", &server.url),
    ]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("9.99.99"),
        "should fetch newer version after TTL: {stderr}"
    );

    // Cache should have been rewritten to fresh.
    let raw = fs::read_to_string(cache_dir.path().join("latest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let checked_at = parsed["checked_at"].as_u64().unwrap();
    assert!(
        now_secs().saturating_sub(checked_at) < 60,
        "cache should be freshly written"
    );
    assert_eq!(parsed["latest"].as_str().unwrap(), "9.99.99");
}

#[test]
fn network_failure_silently_skips_warning() {
    let cache_dir = tempfile::tempdir().unwrap();
    // No cache → forces a fetch attempt.

    let port = closed_port();
    let url = format!("http://127.0.0.1:{port}");

    let output = run_cubil_init(&[
        ("CUBIL_CACHE_DIR", cache_dir.path().to_str().unwrap()),
        ("CUBIL_GITHUB_API_BASE", &url),
    ]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("warning: cubil"),
        "no warning expected on network failure: {stderr}"
    );
    // No error message either — failure is silent.
    assert!(
        stderr.is_empty() || !stderr.contains("error"),
        "unexpected error on stderr: {stderr}"
    );

    // Cache should not have been written (fetch failed).
    assert!(!cache_dir.path().join("latest.json").exists());
}

#[test]
fn warning_does_not_block_when_no_route_available() {
    // Test that a non-update command still produces its normal output even
    // when the stale check fails. Uses an unreachable URL → fetch fails →
    // command runs normally.
    let cache_dir = tempfile::tempdir().unwrap();
    let port = closed_port();
    let url = format!("http://127.0.0.1:{port}");

    let work = tempfile::tempdir().unwrap();
    let output = Command::new(CUBIL_BIN)
        .arg("init")
        .current_dir(work.path())
        .env("CUBIL_CACHE_DIR", cache_dir.path())
        .env("CUBIL_GITHUB_API_BASE", &url)
        .output()
        .expect("run cubil init");

    assert!(output.status.success());
    // init prints the canonical .cubil path on stdout.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(".cubil"),
        "init should still print path: {stdout}"
    );
    assert!(work.path().join(".cubil").is_dir());
}

