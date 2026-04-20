use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

#[test]
fn rm_deletes_file_silently() {
    let dir = tempdir().unwrap();
    let root = dir.path().join(".cubil");
    std::fs::create_dir_all(root.join("backlog")).unwrap();
    let task = root.join("backlog").join("foo.md");
    std::fs::write(&task, "# foo\n").unwrap();

    cubil()
        .args(["rm", "foo"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!task.exists());
}

#[test]
fn rm_errors_when_slug_missing() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".cubil").join("backlog")).unwrap();

    cubil()
        .args(["rm", "nope"])
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn rm_works_in_non_default_status_folder() {
    let dir = tempdir().unwrap();
    let root = dir.path().join(".cubil");
    std::fs::create_dir_all(root.join("custom")).unwrap();
    let task = root.join("custom").join("bar.md");
    std::fs::write(&task, "# bar\n").unwrap();

    cubil()
        .args(["rm", "bar"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!task.exists());
    assert!(root.join("custom").is_dir());
}

#[test]
fn rm_errors_when_slug_is_ambiguous() {
    let dir = tempdir().unwrap();
    let root = dir.path().join(".cubil");
    std::fs::create_dir_all(root.join("backlog")).unwrap();
    std::fs::create_dir_all(root.join("done")).unwrap();
    let a = root.join("backlog").join("dup.md");
    let b = root.join("done").join("dup.md");
    std::fs::write(&a, "# dup\n").unwrap();
    std::fs::write(&b, "# dup\n").unwrap();

    cubil()
        .args(["rm", "dup"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("multiple statuses"));

    assert!(a.exists());
    assert!(b.exists());
}
