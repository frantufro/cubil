use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

fn init_root_with_task(dir: &Path, slug: &str) -> PathBuf {
    let cubil_dir = dir.join(".cubil");
    let backlog = cubil_dir.join("backlog");
    fs::create_dir_all(&backlog).unwrap();
    fs::create_dir_all(cubil_dir.join("doing")).unwrap();
    fs::create_dir_all(cubil_dir.join("done")).unwrap();
    let task_path = backlog.join(format!("{slug}.md"));
    fs::write(&task_path, "# task\n").unwrap();
    task_path.canonicalize().unwrap()
}

/// Write a shell stub that records the args it was invoked with into `marker`.
fn write_stub(dir: &Path, name: &str, marker: &Path) -> PathBuf {
    let path = dir.join(name);
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
        marker.display()
    );
    fs::write(&path, script).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

#[test]
fn edit_invokes_editor_with_resolved_task_path() {
    let dir = tempdir().unwrap();
    let task_path = init_root_with_task(dir.path(), "do-thing");
    let marker = dir.path().join("editor-was-here");
    let stub = write_stub(dir.path(), "stub-editor", &marker);

    cubil()
        .arg("edit")
        .arg("do-thing")
        .current_dir(dir.path())
        .env("EDITOR", &stub)
        .assert()
        .success();

    let argv = fs::read_to_string(&marker).expect("stub did not run");
    let lines: Vec<&str> = argv.lines().collect();
    assert_eq!(lines.last().copied(), Some(task_path.to_str().unwrap()));
}

#[test]
fn edit_passes_extra_args_from_editor_var() {
    let dir = tempdir().unwrap();
    let task_path = init_root_with_task(dir.path(), "do-thing");
    let marker = dir.path().join("editor-was-here");
    let stub = write_stub(dir.path(), "stub-editor", &marker);

    let editor = format!("{} --wait --flag", stub.display());
    cubil()
        .arg("edit")
        .arg("do-thing")
        .current_dir(dir.path())
        .env("EDITOR", editor)
        .assert()
        .success();

    let argv = fs::read_to_string(&marker).expect("stub did not run");
    let lines: Vec<&str> = argv.lines().collect();
    assert_eq!(lines.len(), 3, "argv = {argv:?}");
    assert_eq!(lines[0], "--wait");
    assert_eq!(lines[1], "--flag");
    assert_eq!(lines[2], task_path.to_str().unwrap());
}

#[test]
fn edit_falls_back_to_vi_when_editor_unset() {
    // Prove the fallback by shadowing `vi` on PATH with a stub.
    let dir = tempdir().unwrap();
    let task_path = init_root_with_task(dir.path(), "do-thing");
    let bin_dir = dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let marker = dir.path().join("vi-was-here");
    write_stub(&bin_dir, "vi", &marker);

    cubil()
        .arg("edit")
        .arg("do-thing")
        .current_dir(dir.path())
        .env_remove("EDITOR")
        .env("PATH", &bin_dir)
        .assert()
        .success();

    let argv = fs::read_to_string(&marker).expect("vi stub did not run");
    let lines: Vec<&str> = argv.lines().collect();
    assert_eq!(lines.last().copied(), Some(task_path.to_str().unwrap()));
}

#[test]
fn edit_falls_back_to_vi_when_editor_empty() {
    let dir = tempdir().unwrap();
    let task_path = init_root_with_task(dir.path(), "do-thing");
    let bin_dir = dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let marker = dir.path().join("vi-was-here");
    write_stub(&bin_dir, "vi", &marker);

    cubil()
        .arg("edit")
        .arg("do-thing")
        .current_dir(dir.path())
        .env("EDITOR", "")
        .env("PATH", &bin_dir)
        .assert()
        .success();

    let argv = fs::read_to_string(&marker).expect("vi stub did not run");
    let lines: Vec<&str> = argv.lines().collect();
    assert_eq!(lines.last().copied(), Some(task_path.to_str().unwrap()));
}

#[test]
fn edit_errors_on_missing_slug() {
    let dir = tempdir().unwrap();
    init_root_with_task(dir.path(), "exists");
    // Stub editor that should never be invoked — if it is, the test will see the marker.
    let marker = dir.path().join("editor-was-here");
    let stub = write_stub(dir.path(), "stub-editor", &marker);

    cubil()
        .arg("edit")
        .arg("nope")
        .current_dir(dir.path())
        .env("EDITOR", &stub)
        .assert()
        .failure();

    assert!(
        !marker.exists(),
        "editor must not be spawned for missing slug"
    );
}

#[test]
fn edit_errors_when_outside_a_cubil_root() {
    let dir = tempdir().unwrap();
    let marker = dir.path().join("editor-was-here");
    let stub = write_stub(dir.path(), "stub-editor", &marker);

    cubil()
        .arg("edit")
        .arg("any")
        .current_dir(dir.path())
        .env("EDITOR", &stub)
        .assert()
        .failure();

    assert!(!marker.exists());
}
