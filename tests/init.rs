use assert_cmd::Command;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

#[test]
fn init_creates_three_status_folders() {
    let dir = tempdir().unwrap();
    cubil()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join(".cubil").is_dir());
    assert!(dir.path().join(".cubil/backlog").is_dir());
    assert!(dir.path().join(".cubil/doing").is_dir());
    assert!(dir.path().join(".cubil/done").is_dir());
}

#[test]
fn init_is_idempotent() {
    let dir = tempdir().unwrap();
    for _ in 0..2 {
        cubil()
            .arg("init")
            .current_dir(dir.path())
            .assert()
            .success();
    }
    assert!(dir.path().join(".cubil/backlog").is_dir());
    assert!(dir.path().join(".cubil/doing").is_dir());
    assert!(dir.path().join(".cubil/done").is_dir());
}

#[test]
fn init_errors_when_cubil_is_a_file() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".cubil"), b"not a dir").unwrap();
    cubil()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn init_does_not_walk_upward() {
    let outer = tempdir().unwrap();
    std::fs::create_dir(outer.path().join(".cubil")).unwrap();
    let inner = outer.path().join("nested");
    std::fs::create_dir(&inner).unwrap();

    cubil().arg("init").current_dir(&inner).assert().success();
    assert!(inner.join(".cubil").is_dir());
    assert!(inner.join(".cubil/backlog").is_dir());
}
