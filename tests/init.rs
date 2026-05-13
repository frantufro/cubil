use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cubil() -> Command {
    let mut cmd = Command::cargo_bin("cubil").expect("binary built");
    cmd.env("CUBIL_NO_UPDATE_CHECK", "1");
    cmd
}

#[test]
fn init_creates_three_status_folders() {
    let dir = tempdir().unwrap();
    let expected = dir.path().canonicalize().unwrap().join(".cubil");

    cubil()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected.display().to_string()));

    assert!(dir.path().join(".cubil").is_dir());
    assert!(dir.path().join(".cubil/backlog").is_dir());
    assert!(dir.path().join(".cubil/doing").is_dir());
    assert!(dir.path().join(".cubil/done").is_dir());
    assert!(dir.path().join(".cubil/backlog/.gitkeep").is_file());
    assert!(dir.path().join(".cubil/doing/.gitkeep").is_file());
    assert!(dir.path().join(".cubil/done/.gitkeep").is_file());
}

#[test]
fn init_creates_roadmaps_dir_with_gitkeep() {
    let dir = tempdir().unwrap();
    cubil().arg("init").current_dir(dir.path()).assert().success();

    let roadmaps = dir.path().join(".cubil/roadmaps");
    assert!(roadmaps.is_dir(), "roadmaps/ directory should exist");
    let gitkeep = roadmaps.join(".gitkeep");
    assert!(gitkeep.is_file(), ".gitkeep should exist so empty folder survives `git add`");
    assert_eq!(
        std::fs::read_to_string(&gitkeep).unwrap(),
        "",
        ".gitkeep should be empty"
    );
}

#[test]
fn init_idempotent_does_not_clobber_gitkeep() {
    let dir = tempdir().unwrap();
    cubil().arg("init").current_dir(dir.path()).assert().success();

    let gitkeep = dir.path().join(".cubil/roadmaps/.gitkeep");
    // Mutate the gitkeep so we can detect a clobber.
    std::fs::write(&gitkeep, b"sentinel").unwrap();

    cubil().arg("init").current_dir(dir.path()).assert().success();

    assert_eq!(
        std::fs::read_to_string(&gitkeep).unwrap(),
        "sentinel",
        "second init must not overwrite an existing .gitkeep"
    );
}

#[test]
fn init_is_idempotent() {
    let dir = tempdir().unwrap();
    let expected = dir.path().canonicalize().unwrap().join(".cubil");

    for _ in 0..2 {
        cubil()
            .arg("init")
            .current_dir(dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(expected.display().to_string()));
    }
    assert!(dir.path().join(".cubil/backlog").is_dir());
    assert!(dir.path().join(".cubil/doing").is_dir());
    assert!(dir.path().join(".cubil/done").is_dir());
    assert!(dir.path().join(".cubil/backlog/.gitkeep").is_file());
    assert!(dir.path().join(".cubil/doing/.gitkeep").is_file());
    assert!(dir.path().join(".cubil/done/.gitkeep").is_file());
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
