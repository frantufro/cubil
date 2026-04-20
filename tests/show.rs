use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

fn init_cubil(root: &std::path::Path) {
    std::fs::create_dir_all(root.join(".cubil/backlog")).unwrap();
    std::fs::create_dir_all(root.join(".cubil/doing")).unwrap();
    std::fs::create_dir_all(root.join(".cubil/done")).unwrap();
}

#[test]
fn show_prints_file_bytes_verbatim() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    let body = b"---\ncreated: 2026-04-19\n---\n\n# Hello\n\nBody line.\n";
    let path = dir.path().join(".cubil/backlog/hello.md");
    std::fs::write(&path, body).unwrap();

    let output = cubil()
        .arg("show")
        .arg("hello")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(output, body);
}

#[test]
fn show_errors_on_missing_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .arg("show")
        .arg("nope")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("task not found"));
}

#[test]
fn show_resolves_across_status_folders() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    let body = b"task in doing\n";
    std::fs::write(dir.path().join(".cubil/doing/in-progress.md"), body).unwrap();
    let done_body = b"task in done\n";
    std::fs::write(dir.path().join(".cubil/done/finished.md"), done_body).unwrap();

    let out_doing = cubil()
        .arg("show")
        .arg("in-progress")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(out_doing, body);

    let out_done = cubil()
        .arg("show")
        .arg("finished")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(out_done, done_body);
}

#[test]
fn show_preserves_non_ascii_bytes() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    // Deliberate mix: multi-byte UTF-8 (café, 日本語, emoji), CR, LF, lone-CR,
    // no trailing newline. `io::copy` must stream bytes unchanged.
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice("café résumé — 日本語 — 🚀\r\n".as_bytes());
    body.extend_from_slice(b"line2 no-newline-at-eof");
    // Arbitrary non-UTF8 byte sequence (0xFF isn't valid UTF-8) to prove we
    // don't re-encode through a String.
    body.extend_from_slice(&[0xFF, 0xFE, 0xFD]);

    let path = dir.path().join(".cubil/backlog/unicode.md");
    std::fs::write(&path, &body).unwrap();

    let output = cubil()
        .arg("show")
        .arg("unicode")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(output, body);
}

#[test]
fn show_errors_on_ambiguous_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    std::fs::write(dir.path().join(".cubil/backlog/dup.md"), b"a").unwrap();
    std::fs::write(dir.path().join(".cubil/done/dup.md"), b"b").unwrap();

    cubil()
        .arg("show")
        .arg("dup")
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn show_walks_upward_to_find_cubil() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    std::fs::write(dir.path().join(".cubil/backlog/nested.md"), b"hi\n").unwrap();

    let nested = dir.path().join("a/b/c");
    std::fs::create_dir_all(&nested).unwrap();

    let output = cubil()
        .arg("show")
        .arg("nested")
        .current_dir(&nested)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(output, b"hi\n");
}
