use assert_cmd::Command;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

fn init_repo(path: &std::path::Path) {
    for status in ["backlog", "doing", "done"] {
        std::fs::create_dir_all(path.join(".cubil").join(status)).unwrap();
    }
}

#[test]
fn mv_moves_task_between_existing_folders() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let src = dir.path().join(".cubil/backlog/foo.md");
    std::fs::write(&src, "body").unwrap();

    cubil()
        .args(["mv", "foo", "doing"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!src.exists());
    let dest = dir.path().join(".cubil/doing/foo.md");
    assert!(dest.is_file());
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "body");
}

#[test]
fn mv_to_same_status_is_silent_noop() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let src = dir.path().join(".cubil/backlog/foo.md");
    std::fs::write(&src, "body").unwrap();

    cubil()
        .args(["mv", "foo", "backlog"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(src.is_file());
    assert_eq!(std::fs::read_to_string(&src).unwrap(), "body");
}

#[test]
fn mv_errors_when_destination_status_missing() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let src = dir.path().join(".cubil/backlog/foo.md");
    std::fs::write(&src, "body").unwrap();

    cubil()
        .args(["mv", "foo", "review"])
        .current_dir(dir.path())
        .assert()
        .failure();

    assert!(src.is_file());
    assert!(!dir.path().join(".cubil/review").exists());
}

#[test]
fn mv_preserves_file_bytes_across_rename() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let body = "---\npriority: high\n---\n\nLine one.\nLine two.\n\nTrailing newline.\n";
    let src = dir.path().join(".cubil/backlog/foo.md");
    std::fs::write(&src, body).unwrap();

    cubil()
        .args(["mv", "foo", "doing"])
        .current_dir(dir.path())
        .assert()
        .success();

    let dest = dir.path().join(".cubil/doing/foo.md");
    assert_eq!(std::fs::read(&dest).unwrap(), body.as_bytes());
}

#[test]
fn mv_errors_when_slug_missing() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());

    cubil()
        .args(["mv", "ghost", "doing"])
        .current_dir(dir.path())
        .assert()
        .failure();
}
