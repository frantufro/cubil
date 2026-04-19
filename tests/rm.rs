use assert_cmd::Command;
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
