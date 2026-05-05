use std::collections::HashMap;

use crate::core::error::Result;
use crate::core::roadmap::{TaskStatus, parse_task_line, resolve_roadmap};
use crate::core::root::find_root;
use crate::core::slug::scan_all;

/// Print the slug of the next not-done task in the roadmap.
///
/// Walks task lines in file order and prints the first one whose status is
/// not `done` (and which has a corresponding task file — missing slugs are
/// skipped, since `next` exists to drive `cubil start <slug>`). Empty stdout
/// + zero exit if all tasks are done. Designed for shell pipelines.
pub fn run(slug: String) -> Result<()> {
    let root = find_root(None)?;
    let path = resolve_roadmap(&root, &slug)?;
    let src = std::fs::read_to_string(&path)?;

    let lookup = build_status_lookup(&root)?;
    if let Some(next) = first_not_done(&src, &lookup) {
        println!("{next}");
    }
    Ok(())
}

fn build_status_lookup(root: &std::path::Path) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for entry in scan_all(root)? {
        map.entry(entry.slug).or_insert(entry.status);
    }
    Ok(map)
}

fn first_not_done(src: &str, lookup: &HashMap<String, String>) -> Option<String> {
    for line in src.lines() {
        let Some(parsed) = parse_task_line(line) else {
            continue;
        };
        let status = match lookup.get(&parsed.slug) {
            Some(s) => TaskStatus::from_status_name(s),
            None => continue, // missing task — not a candidate
        };
        if !status.is_done() {
            return Some(parsed.slug);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(items: &[(&str, &str)]) -> HashMap<String, String> {
        items
            .iter()
            .map(|(s, t)| ((*s).to_string(), (*t).to_string()))
            .collect()
    }

    #[test]
    fn returns_first_backlog_task() {
        let map = lookup(&[("a", "done"), ("b", "backlog"), ("c", "backlog")]);
        let src = "- [ ] a\n- [ ] b\n- [ ] c\n";
        assert_eq!(first_not_done(src, &map).as_deref(), Some("b"));
    }

    #[test]
    fn returns_first_doing_task_over_later_backlog() {
        let map = lookup(&[("a", "done"), ("b", "doing"), ("c", "backlog")]);
        let src = "- [ ] a\n- [ ] b\n- [ ] c\n";
        assert_eq!(first_not_done(src, &map).as_deref(), Some("b"));
    }

    #[test]
    fn returns_none_when_all_done() {
        let map = lookup(&[("a", "done"), ("b", "done")]);
        let src = "- [ ] a\n- [ ] b\n";
        assert_eq!(first_not_done(src, &map), None);
    }

    #[test]
    fn returns_none_for_empty_roadmap() {
        let map = lookup(&[]);
        assert_eq!(first_not_done("# Just a title\n", &map), None);
    }

    #[test]
    fn skips_missing_tasks() {
        let map = lookup(&[("a", "done"), ("c", "backlog")]);
        let src = "- [ ] a\n- [ ] missing\n- [ ] c\n";
        assert_eq!(first_not_done(src, &map).as_deref(), Some("c"));
    }

    #[test]
    fn ignores_milestone_headings_and_narrative() {
        let map = lookup(&[("first-task", "backlog")]);
        let src = "\
# Roadmap

Narrative.

## Milestone: M
- [ ] first-task
";
        assert_eq!(first_not_done(src, &map).as_deref(), Some("first-task"));
    }
}
