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
fn finish_moves_task_from_doing_to_done() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let src = dir.path().join(".cubil/doing/foo.md");
    std::fs::write(&src, "body").unwrap();

    cubil()
        .args(["finish", "foo"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!src.exists());
    let dest = dir.path().join(".cubil/done/foo.md");
    assert!(dest.is_file());
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "body");
}

#[test]
fn finish_errors_when_task_in_backlog() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let src = dir.path().join(".cubil/backlog/foo.md");
    std::fs::write(&src, "body").unwrap();

    cubil()
        .args(["finish", "foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "task 'foo' is in 'backlog', not 'doing'",
        ));

    assert!(src.is_file());
}

#[test]
fn finish_errors_when_task_already_in_done() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());
    let src = dir.path().join(".cubil/done/foo.md");
    std::fs::write(&src, "body").unwrap();

    cubil()
        .args(["finish", "foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "task 'foo' is in 'done', not 'doing'",
        ));

    assert!(src.is_file());
}

#[test]
fn finish_errors_when_slug_missing() {
    let dir = tempdir().unwrap();
    init_repo(dir.path());

    cubil()
        .args(["finish", "ghost"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("task not found: ghost"));
}
