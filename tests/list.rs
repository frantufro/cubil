use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

fn seed_fixture(root: &Path) {
    // backlog/
    std::fs::create_dir_all(root.join(".cubil/backlog")).unwrap();
    std::fs::create_dir_all(root.join(".cubil/doing")).unwrap();
    std::fs::create_dir_all(root.join(".cubil/done")).unwrap();

    std::fs::write(
        root.join(".cubil/backlog/tidy-readme.md"),
        "# no frontmatter here\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".cubil/backlog/big-refactor.md"),
        "---\ncreated: 2026-04-10\npriority: 3\n---\n# body\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".cubil/doing/implement-new.md"),
        "---\ncreated: 2026-04-19\npriority: 1\n---\n# body\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".cubil/done/ancient-bug.md"),
        "---\ncreated: 2026-03-01\npriority: 2\n---\n",
    )
    .unwrap();
}

#[test]
fn list_default_excludes_done_and_aligns_columns() {
    let dir = tempdir().unwrap();
    seed_fixture(dir.path());

    let assert = cubil()
        .arg("list")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Header + three rows (backlog has 2, doing has 1, done excluded).
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4, "expected 4 lines, got: {out:?}");
    assert!(lines[0].starts_with("slug"));
    assert!(!out.contains("ancient-bug"), "done/ should be hidden");

    // Sort: status asc (backlog, backlog, doing), priority asc nulls last,
    // slug asc. So: big-refactor (p3), tidy-readme (None), implement-new.
    assert!(lines[1].contains("big-refactor"));
    assert!(lines[2].contains("tidy-readme"));
    assert!(lines[3].contains("implement-new"));

    // Column alignment: slug column must pad to widest slug among all rows
    // (big-refactor and implement-new are both 13, tidy-readme is 11).
    // Each cell-before-last is left-padded to the max width. So the `status`
    // column should appear at the same byte column on every data line.
    let find_status = |line: &str, name: &str| line.find(name).unwrap();
    let c1 = find_status(lines[1], "backlog");
    let c2 = find_status(lines[2], "backlog");
    let c3 = find_status(lines[3], "doing");
    assert_eq!(c1, c2);
    assert_eq!(c1, c3);

    // Missing priority/created render as `-` in the table.
    assert!(lines[2].contains("-"));
}

#[test]
fn list_all_includes_done() {
    let dir = tempdir().unwrap();
    seed_fixture(dir.path());

    let assert = cubil()
        .arg("list")
        .arg("--all")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(out.contains("ancient-bug"));
    assert!(out.contains("big-refactor"));
    assert!(out.contains("tidy-readme"));
    assert!(out.contains("implement-new"));
}

#[test]
fn list_status_done_includes_done_without_all() {
    let dir = tempdir().unwrap();
    seed_fixture(dir.path());

    let assert = cubil()
        .arg("list")
        .arg("--status")
        .arg("done")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(out.contains("ancient-bug"));
    assert!(!out.contains("big-refactor"));
    assert!(!out.contains("tidy-readme"));
    assert!(!out.contains("implement-new"));
}

#[test]
fn list_status_filters_to_single_folder() {
    let dir = tempdir().unwrap();
    seed_fixture(dir.path());

    let assert = cubil()
        .arg("list")
        .arg("--status")
        .arg("doing")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(out.contains("implement-new"));
    assert!(!out.contains("big-refactor"));
    assert!(!out.contains("tidy-readme"));
    assert!(!out.contains("ancient-bug"));
}

#[test]
fn list_json_emits_parseable_array() {
    let dir = tempdir().unwrap();
    seed_fixture(dir.path());

    let assert = cubil()
        .arg("list")
        .arg("--all")
        .arg("--json")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let trimmed = out.trim();

    // Expected objects (order: status asc, priority asc nulls last, slug asc).
    // Status order alphabetically: backlog < doing < done ('i' < 'n').
    // backlog: big-refactor(3) before tidy-readme(None).
    // doing: implement-new(1). done: ancient-bug(2).
    let expected = r#"[{"slug":"big-refactor","status":"backlog","priority":3,"created":"2026-04-10"},{"slug":"tidy-readme","status":"backlog","priority":null,"created":null},{"slug":"implement-new","status":"doing","priority":1,"created":"2026-04-19"},{"slug":"ancient-bug","status":"done","priority":2,"created":"2026-03-01"}]"#;
    assert_eq!(trimmed, expected);
}

#[test]
fn list_works_with_empty_status_folder() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".cubil/backlog")).unwrap();
    std::fs::create_dir_all(dir.path().join(".cubil/doing")).unwrap();
    std::fs::create_dir_all(dir.path().join(".cubil/done")).unwrap();
    std::fs::write(
        dir.path().join(".cubil/backlog/only.md"),
        "---\npriority: 1\n---\n",
    )
    .unwrap();

    // Default (doing empty, done empty): just one row.
    let assert = cubil()
        .arg("list")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("only"));

    // --status doing (empty folder): header only, exit 0.
    cubil()
        .arg("list")
        .arg("--status")
        .arg("doing")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("slug"));

    // JSON for an empty filter is still valid: "[]".
    let assert = cubil()
        .arg("list")
        .arg("--status")
        .arg("doing")
        .arg("--json")
        .current_dir(dir.path())
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_eq!(out.trim(), "[]");
}

#[test]
fn list_errors_when_no_cubil_root() {
    let dir = tempdir().unwrap();
    cubil()
        .arg("list")
        .current_dir(dir.path())
        .assert()
        .failure();
}
