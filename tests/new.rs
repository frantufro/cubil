use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

fn init_cubil(dir: &Path) {
    cubil().arg("init").current_dir(dir).assert().success();
}

fn read_task(dir: &Path, status: &str, slug: &str) -> String {
    std::fs::read_to_string(dir.join(".cubil").join(status).join(format!("{slug}.md")))
        .expect("task file readable")
}

fn assert_iso_date(s: &str) {
    assert_eq!(s.len(), 10, "date `{s}` not 10 chars");
    assert_eq!(&s[4..5], "-");
    assert_eq!(&s[7..8], "-");
    assert!(s[0..4].chars().all(|c| c.is_ascii_digit()));
    assert!(s[5..7].chars().all(|c| c.is_ascii_digit()));
    assert!(s[8..10].chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn new_with_message_writes_frontmatter_title_and_body() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    let output = cubil()
        .args(["new", "Fix Widget", "-m", "The widget is broken."])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "fix-widget\n",
        "stdout should be exactly `<slug>\\n`"
    );

    let contents = read_task(dir.path(), "backlog", "fix-widget");
    let lines: Vec<&str> = contents.split('\n').collect();
    assert_eq!(lines[0], "---");
    assert!(lines[1].starts_with("created: "));
    assert_iso_date(&lines[1]["created: ".len()..]);
    assert_eq!(lines[2], "---");
    assert_eq!(lines[3], "");
    assert_eq!(lines[4], "# Fix Widget");
    assert_eq!(lines[5], "");
    assert_eq!(lines[6], "The widget is broken.");
    assert_eq!(lines[7], "");
    assert_eq!(lines.len(), 8, "expected trailing newline and nothing more");
    assert!(
        !contents.contains("priority:"),
        "priority key should be omitted when unset"
    );
}

#[test]
fn new_with_file_reads_body_from_disk() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    let body_path = dir.path().join("body.md");
    std::fs::write(&body_path, "Body from file.\nSecond line.\n").unwrap();

    cubil()
        .args(["new", "From File"])
        .arg("-F")
        .arg(&body_path)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("from-file\n");

    let contents = read_task(dir.path(), "backlog", "from-file");
    assert!(
        contents.ends_with("# From File\n\nBody from file.\nSecond line.\n"),
        "file body not appended verbatim: {contents:?}"
    );
}

#[test]
fn new_with_stdin_reads_body_from_stdin() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .args(["new", "From Stdin", "-F", "-"])
        .current_dir(dir.path())
        .write_stdin("stdin body")
        .assert()
        .success()
        .stdout("from-stdin\n");

    let contents = read_task(dir.path(), "backlog", "from-stdin");
    let lines: Vec<&str> = contents.split('\n').collect();
    assert_eq!(lines[0], "---");
    assert!(lines[1].starts_with("created: "));
    assert_iso_date(&lines[1]["created: ".len()..]);
    assert_eq!(lines[2], "---");
    assert_eq!(lines[3], "");
    assert_eq!(lines[4], "# From Stdin");
    assert_eq!(lines[5], "");
    assert_eq!(lines[6], "stdin body");
    assert_eq!(lines[7], "");
    assert_eq!(lines.len(), 8);
}

#[test]
fn new_without_body_produces_no_trailing_blank_line() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .args(["new", "Empty Body"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("empty-body\n");

    let contents = read_task(dir.path(), "backlog", "empty-body");
    assert!(
        contents.ends_with("# Empty Body\n"),
        "expected file to end at title line; got: {contents:?}"
    );
    assert!(!contents.ends_with("\n\n"));
}

#[test]
fn new_errors_on_slug_collision() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    // Pre-seed a collision in a different status folder. `init` already
    // creates `doing/`, but create it defensively so this test doesn't
    // depend on implementation details of another command.
    let doing = dir.path().join(".cubil").join("doing");
    std::fs::create_dir_all(&doing).unwrap();
    std::fs::write(doing.join("dup.md"), "placeholder").unwrap();

    cubil()
        .args(["new", "Dup", "-m", "body"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);

    // Collision must prevent writing to backlog.
    assert!(!dir.path().join(".cubil/backlog/dup.md").exists());
}

#[test]
fn new_errors_when_no_cubil_root() {
    let dir = tempdir().unwrap();
    cubil()
        .args(["new", "Nope", "-m", "x"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn new_errors_on_empty_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    cubil()
        .args(["new", "!!!", "-m", "x"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}
