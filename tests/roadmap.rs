use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

fn cubil() -> Command {
    Command::cargo_bin("cubil").expect("binary built")
}

fn init_cubil(dir: &Path) {
    cubil().arg("init").current_dir(dir).assert().success();
}

fn make_task(dir: &Path, status: &str, slug: &str, title: &str) {
    let path = dir.join(".cubil").join(status).join(format!("{slug}.md"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, format!("# {title}\n")).unwrap();
}

fn read_roadmap(dir: &Path, slug: &str) -> String {
    std::fs::read_to_string(dir.join(".cubil/roadmaps").join(format!("{slug}.md")))
        .expect("roadmap readable")
}

// ─── roadmap new ────────────────────────────────────────────────────────────

#[test]
fn roadmap_new_writes_file_and_prints_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .args(["roadmap", "new", "Migrate to Postgres", "-m", "Optional narrative."])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("migrate-to-postgres\n");

    let contents = read_roadmap(dir.path(), "migrate-to-postgres");
    assert_eq!(
        contents,
        "# Migrate to Postgres\n\nOptional narrative.\n",
        "roadmap file should be plain markdown with no frontmatter"
    );
}

#[test]
fn roadmap_new_without_body_writes_just_title() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .args(["roadmap", "new", "Bare"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("bare\n");

    assert_eq!(read_roadmap(dir.path(), "bare"), "# Bare\n");
}

#[test]
fn roadmap_new_with_stdin_body() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .args(["roadmap", "new", "From Stdin", "-F", "-"])
        .current_dir(dir.path())
        .write_stdin("piped narrative")
        .assert()
        .success()
        .stdout("from-stdin\n");

    assert_eq!(
        read_roadmap(dir.path(), "from-stdin"),
        "# From Stdin\n\npiped narrative\n"
    );
}

#[test]
fn roadmap_new_errors_on_duplicate_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil()
        .args(["roadmap", "new", "Foo"])
        .current_dir(dir.path())
        .assert()
        .success();

    cubil()
        .args(["roadmap", "new", "Foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn roadmap_and_task_slugs_share_no_namespace() {
    // Same slug `foo` for both a task and a roadmap is allowed.
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    make_task(dir.path(), "backlog", "foo", "Foo Task");
    cubil()
        .args(["roadmap", "new", "Foo"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("foo\n");
}

// ─── roadmap list ───────────────────────────────────────────────────────────

#[test]
fn roadmap_list_shows_slug_and_title_sorted() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil().args(["roadmap", "new", "Zebra"]).current_dir(dir.path()).assert().success();
    cubil().args(["roadmap", "new", "Alpha"]).current_dir(dir.path()).assert().success();

    let out = cubil()
        .args(["roadmap", "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "header + 2 rows");
    assert!(lines[0].starts_with("slug"));
    assert!(lines[1].starts_with("alpha"));
    assert!(lines[1].contains("Alpha"));
    assert!(lines[2].starts_with("zebra"));
    assert!(lines[2].contains("Zebra"));
}

#[test]
fn roadmap_list_json_format() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil().args(["roadmap", "new", "Foo"]).current_dir(dir.path()).assert().success();

    cubil()
        .args(["roadmap", "list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("[{\"slug\":\"foo\",\"title\":\"Foo\"}]\n");
}

#[test]
fn roadmap_list_handles_missing_title() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    // Hand-write a roadmap file with no `# ` heading.
    std::fs::write(dir.path().join(".cubil/roadmaps/bare.md"), "just narrative\n").unwrap();

    let out = cubil()
        .args(["roadmap", "list", "--json"])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(out).unwrap(), "[{\"slug\":\"bare\",\"title\":null}]\n");
}

#[test]
fn cubil_list_does_not_surface_roadmaps_as_tasks() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    cubil().args(["roadmap", "new", "My Roadmap"]).current_dir(dir.path()).assert().success();
    make_task(dir.path(), "backlog", "real-task", "Real Task");

    let out = cubil()
        .args(["list", "--all"])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();
    assert!(stdout.contains("real-task"), "task should appear in list");
    assert!(!stdout.contains("my-roadmap"), "roadmap must not appear in `cubil list`");
    assert!(!stdout.contains("roadmaps"), "roadmap directory must not appear as a status");
}

// ─── roadmap show ───────────────────────────────────────────────────────────

#[test]
fn roadmap_show_resolves_all_four_status_markers() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    make_task(dir.path(), "done", "schema", "Schema Migration");
    make_task(dir.path(), "doing", "data-copy", "Data Copy");
    make_task(dir.path(), "backlog", "cutover", "Cutover");

    cubil().args(["roadmap", "new", "Migrate"]).current_dir(dir.path()).assert().success();
    let path = dir.path().join(".cubil/roadmaps/migrate.md");
    std::fs::write(
        &path,
        "\
# Migrate

## Milestone: M
- [ ] schema
- [ ] data-copy
- [ ] cutover
- [ ] missing-task
",
    )
    .unwrap();

    let out = cubil()
        .args(["roadmap", "show", "migrate"])
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();
    assert!(stdout.contains("- [\u{2713}] schema \u{2014} Schema Migration"));
    assert!(stdout.contains("- [~] data-copy \u{2014} Data Copy"));
    assert!(stdout.contains("- [ ] cutover \u{2014} Cutover"));
    assert!(stdout.contains("- [?] missing-task \u{2014} (missing)"));
}

#[test]
fn roadmap_show_rewrites_file_on_disk() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    make_task(dir.path(), "done", "foo", "Foo Title");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "# R\n\n- [ ] foo\n").unwrap();

    cubil()
        .args(["roadmap", "show", "r"])
        .current_dir(dir.path())
        .assert()
        .success();

    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(on_disk, "# R\n\n- [\u{2713}] foo \u{2014} Foo Title\n");
}

#[test]
fn roadmap_show_preserves_non_task_content() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "done", "foo", "Foo");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    let original = "\
# R

Some narrative paragraph.

## Milestone: First
Notes inside section.
- [ ] foo
  - sub-bullet under task
- [ ] write the docs

## Milestone: Second

End.
";
    std::fs::write(&path, original).unwrap();

    cubil()
        .args(["roadmap", "show", "r"])
        .current_dir(dir.path())
        .assert()
        .success();

    let on_disk = std::fs::read_to_string(&path).unwrap();
    // Task line rewritten, everything else preserved.
    assert!(on_disk.contains("- [\u{2713}] foo \u{2014} Foo"));
    assert!(on_disk.contains("Some narrative paragraph."));
    assert!(on_disk.contains("Notes inside section."));
    assert!(on_disk.contains("  - sub-bullet under task"));
    assert!(on_disk.contains("- [ ] write the docs"), "free-form prose checkbox preserved");
    assert!(on_disk.contains("## Milestone: Second"));
    assert!(on_disk.contains("End."));
}

#[test]
fn roadmap_show_self_heals_when_missing_task_restored() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "- [ ] foo\n").unwrap();

    // First show: foo doesn't exist.
    cubil().args(["roadmap", "show", "r"]).current_dir(dir.path()).assert().success();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "- [?] foo \u{2014} (missing)\n"
    );

    // Restore foo.
    make_task(dir.path(), "done", "foo", "Restored");
    cubil().args(["roadmap", "show", "r"]).current_dir(dir.path()).assert().success();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "- [\u{2713}] foo \u{2014} Restored\n"
    );
}

#[test]
fn roadmap_show_errors_on_missing_roadmap() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    cubil()
        .args(["roadmap", "show", "nope"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

// ─── roadmap add ────────────────────────────────────────────────────────────

#[test]
fn roadmap_add_default_appends_to_last_milestone() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "backlog", "task-x", "X");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(
        &path,
        "\
# R

## Milestone: A
- [ ] earlier

## Milestone: B
- [ ] also-earlier
",
    )
    .unwrap();
    // Make sure earlier slugs exist so they wouldn't trigger duplicate
    // detection when we add.
    make_task(dir.path(), "backlog", "earlier", "E");
    make_task(dir.path(), "backlog", "also-earlier", "AE");

    cubil()
        .args(["roadmap", "add", "r", "task-x"])
        .current_dir(dir.path())
        .assert()
        .success();

    let out = std::fs::read_to_string(&path).unwrap();
    let expected = "\
# R

## Milestone: A
- [ ] earlier

## Milestone: B
- [ ] also-earlier
- [ ] task-x
";
    assert_eq!(out, expected);
}

#[test]
fn roadmap_add_with_milestone_targets_named_section() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "backlog", "task-x", "X");
    make_task(dir.path(), "backlog", "earlier", "E");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(
        &path,
        "\
## Milestone: A
- [ ] earlier

## Milestone: B
",
    )
    .unwrap();

    cubil()
        .args(["roadmap", "add", "r", "task-x", "--milestone", "A"])
        .current_dir(dir.path())
        .assert()
        .success();

    let out = std::fs::read_to_string(&path).unwrap();
    let expected = "\
## Milestone: A
- [ ] earlier
- [ ] task-x

## Milestone: B
";
    assert_eq!(out, expected);
}

#[test]
fn roadmap_add_errors_on_missing_milestone() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "backlog", "task-x", "X");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "## Milestone: A\n").unwrap();

    cubil()
        .args(["roadmap", "add", "r", "task-x", "--milestone", "Z"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn roadmap_add_errors_on_missing_task_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    cubil().args(["roadmap", "new", "R"]).current_dir(dir.path()).assert().success();

    cubil()
        .args(["roadmap", "add", "r", "ghost"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn roadmap_add_errors_on_duplicate_task() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "backlog", "task-x", "X");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "## Milestone: A\n- [ ] task-x\n").unwrap();

    cubil()
        .args(["roadmap", "add", "r", "task-x"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn roadmap_add_to_milestone_less_file_appends_to_eof() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "backlog", "task-x", "X");

    cubil().args(["roadmap", "new", "R"]).current_dir(dir.path()).assert().success();

    cubil()
        .args(["roadmap", "add", "r", "task-x"])
        .current_dir(dir.path())
        .assert()
        .success();

    let on_disk = read_roadmap(dir.path(), "r");
    assert_eq!(on_disk, "# R\n- [ ] task-x\n");
}

// ─── roadmap next ───────────────────────────────────────────────────────────

#[test]
fn roadmap_next_returns_first_non_done_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "done", "a", "A");
    make_task(dir.path(), "doing", "b", "B");
    make_task(dir.path(), "backlog", "c", "C");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "- [ ] a\n- [ ] b\n- [ ] c\n").unwrap();

    cubil()
        .args(["roadmap", "next", "r"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("b\n");
}

#[test]
fn roadmap_next_empty_when_all_done() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "done", "a", "A");
    make_task(dir.path(), "done", "b", "B");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "- [ ] a\n- [ ] b\n").unwrap();

    cubil()
        .args(["roadmap", "next", "r"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("");
}

#[test]
fn roadmap_next_returns_doing_task_before_later_backlog() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    make_task(dir.path(), "done", "a", "A");
    make_task(dir.path(), "doing", "b", "B");
    make_task(dir.path(), "backlog", "c", "C");

    let path = dir.path().join(".cubil/roadmaps/r.md");
    std::fs::write(&path, "- [ ] a\n- [ ] b\n- [ ] c\n").unwrap();

    cubil()
        .args(["roadmap", "next", "r"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("b\n");
}

// ─── roadmap rm ─────────────────────────────────────────────────────────────

#[test]
fn roadmap_rm_deletes_file_silently() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    cubil().args(["roadmap", "new", "Foo"]).current_dir(dir.path()).assert().success();

    cubil()
        .args(["roadmap", "rm", "foo"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("");

    assert!(!dir.path().join(".cubil/roadmaps/foo.md").exists());
}

#[test]
fn roadmap_rm_errors_on_missing_slug() {
    let dir = tempdir().unwrap();
    init_cubil(dir.path());
    cubil()
        .args(["roadmap", "rm", "nope"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}
