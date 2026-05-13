use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::core::error::Result;
use crate::core::roadmap::{
    TaskStatus, parse_task_line, read_task_title, resolve_roadmap,
};
use crate::core::root::find_root;
use crate::core::slug::scan_all;

/// Render the roadmap with task statuses resolved from the actual task files,
/// then write the resolved view back to disk.
///
/// Resolution rules:
/// - Done task → `- [✓] <slug> — <title>`
/// - Doing → `- [~] <slug> — <title>`
/// - Backlog or any other status folder → `- [ ] <slug> — <title>`
/// - Slug with no task file → `- [?] <slug> — (missing)`
///
/// Non-task content (headings, narrative, sub-bullets, blank lines) passes
/// through byte-for-byte. Only lines parsed as `- [<x>] <slug>` checkbox
/// items are rewritten.
pub fn run(slug: String) -> Result<()> {
    let root = find_root(None)?;
    let path = resolve_roadmap(&root, &slug)?;
    let src = std::fs::read_to_string(&path)?;

    let lookup = build_lookup(&root)?;
    let new_src = rewrite(&src, &lookup)?;

    write_atomic(&path, new_src.as_bytes())?;

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(new_src.as_bytes())?;
    Ok(())
}

struct TaskInfo {
    status: TaskStatus,
    title: Option<String>,
}

fn build_lookup(root: &Path) -> Result<HashMap<String, (String, PathBuf)>> {
    let mut map = HashMap::new();
    for entry in scan_all(root)? {
        // First-write-wins for ambiguous slugs. Roadmap rendering is robust
        // to ambiguity: it just picks one. Users discover the duplicate via
        // `cubil list` or `cubil show`.
        map.entry(entry.slug)
            .or_insert((entry.status, entry.path));
    }
    Ok(map)
}

fn resolve(slug: &str, lookup: &HashMap<String, (String, PathBuf)>) -> Result<TaskInfo> {
    match lookup.get(slug) {
        None => Ok(TaskInfo {
            status: TaskStatus::Missing,
            title: None,
        }),
        Some((status_name, path)) => {
            let status = TaskStatus::from_status_name(status_name);
            let title = read_task_title(path)?;
            Ok(TaskInfo { status, title })
        }
    }
}

fn rewrite(src: &str, lookup: &HashMap<String, (String, PathBuf)>) -> Result<String> {
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        let eol_len = if line.ends_with("\r\n") {
            2
        } else if line.ends_with('\n') {
            1
        } else {
            0
        };
        let body = &line[..line.len() - eol_len];
        let eol = &line[line.len() - eol_len..];

        match parse_task_line(body) {
            Some(parsed) => {
                let info = resolve(&parsed.slug, lookup)?;
                let new_body = render_task_line(&parsed.indent, &parsed.slug, &info);
                out.push_str(&new_body);
                out.push_str(eol);
            }
            None => {
                out.push_str(line);
            }
        }
    }
    Ok(out)
}

fn render_task_line(indent: &str, slug: &str, info: &TaskInfo) -> String {
    let checkbox = info.status.checkbox();
    let suffix = if info.status.is_missing() {
        "(missing)".to_string()
    } else {
        info.title.clone().unwrap_or_else(|| "(untitled)".to_string())
    };
    format!("{indent}- [{checkbox}] {slug} \u{2014} {suffix}")
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().expect("roadmap path has parent");
    let tmp = parent.join(format!(
        ".{}.tmp",
        path.file_name().expect("roadmap has filename").to_string_lossy()
    ));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(items: &[(&str, &str, Option<&str>)]) -> HashMap<String, (String, PathBuf)> {
        // Build a lookup whose values point to per-test temp files containing
        // the desired title. We use tempdir per test instead.
        let dir = tempfile::tempdir().unwrap();
        let mut map = HashMap::new();
        for (slug, status, title) in items {
            let dir_path = dir.path().join(status);
            std::fs::create_dir_all(&dir_path).unwrap();
            let path = dir_path.join(format!("{slug}.md"));
            let body = match title {
                Some(t) => format!("# {t}\n"),
                None => String::new(),
            };
            std::fs::write(&path, body).unwrap();
            map.insert((*slug).to_string(), ((*status).to_string(), path));
        }
        // Leak the tempdir so files survive the rest of the test.
        std::mem::forget(dir);
        map
    }

    #[test]
    fn rewrite_resolves_all_status_markers() {
        let map = lookup(&[
            ("a", "done", Some("Task A")),
            ("b", "doing", Some("Task B")),
            ("c", "backlog", Some("Task C")),
        ]);
        let src = "\
# Roadmap

## Milestone: M
- [ ] a
- [ ] b
- [ ] c
- [ ] missing-task
";
        let out = rewrite(src, &map).unwrap();
        let expected = "\
# Roadmap

## Milestone: M
- [\u{2713}] a \u{2014} Task A
- [~] b \u{2014} Task B
- [ ] c \u{2014} Task C
- [?] missing-task \u{2014} (missing)
";
        assert_eq!(out, expected);
    }

    #[test]
    fn rewrite_preserves_non_task_lines_byte_for_byte() {
        let map = lookup(&[("a", "done", Some("A"))]);
        let src = "\
# Title

Some narrative.

## Milestone: First
Notes inside section.
- [ ] a
  - sub bullet
- not a checkbox
";
        let out = rewrite(src, &map).unwrap();
        // Everything except the `- [ ] a` line is unchanged.
        assert!(out.contains("Some narrative.\n"));
        assert!(out.contains("Notes inside section.\n"));
        assert!(out.contains("  - sub bullet\n"));
        assert!(out.contains("- not a checkbox\n"));
        assert!(out.contains("- [\u{2713}] a \u{2014} A\n"));
    }

    #[test]
    fn rewrite_handles_already_resolved_lines() {
        let map = lookup(&[("a", "done", Some("New Title"))]);
        let src = "- [ ] a — Old Title\n";
        let out = rewrite(src, &map).unwrap();
        assert_eq!(out, "- [\u{2713}] a \u{2014} New Title\n");
    }

    #[test]
    fn rewrite_uses_untitled_when_task_has_no_h1() {
        let map = lookup(&[("a", "done", None)]);
        let src = "- [ ] a\n";
        let out = rewrite(src, &map).unwrap();
        assert_eq!(out, "- [\u{2713}] a \u{2014} (untitled)\n");
    }

    #[test]
    fn rewrite_treats_unknown_status_as_backlog_marker() {
        let map = lookup(&[("a", "review", Some("A"))]);
        let src = "- [✓] a — Old\n";
        let out = rewrite(src, &map).unwrap();
        assert_eq!(out, "- [ ] a \u{2014} A\n");
    }
}
