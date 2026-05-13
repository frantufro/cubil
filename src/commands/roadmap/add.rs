use std::path::Path;

use crate::core::error::{CubilError, Result};
use crate::core::roadmap::{
    parse_h2_heading, parse_milestone_heading, parse_task_line, resolve_roadmap,
};
use crate::core::root::find_root;
use crate::core::slug::scan_all;

/// Append `- [ ] <task_slug>` to a roadmap.
///
/// With `milestone == None`, appends to the end of the last milestone in the
/// file. If the file has no milestones, appends to end of file. With
/// `milestone == Some(name)`, appends to the end of that named milestone's
/// section (errors if missing or ambiguous).
///
/// Validates that `task_slug` exists in some status folder (backlog, doing,
/// done, or any custom one). No forward references. Errors if the task is
/// already listed anywhere in this roadmap.
pub fn run(roadmap_slug: String, task_slug: String, milestone: Option<String>) -> Result<()> {
    let root = find_root(None)?;
    let path = resolve_roadmap(&root, &roadmap_slug)?;

    // Validate task exists somewhere.
    let mut found = false;
    for entry in scan_all(&root)? {
        if entry.slug == task_slug {
            found = true;
            break;
        }
    }
    if !found {
        return Err(CubilError::SlugNotFound(task_slug));
    }

    let src = std::fs::read_to_string(&path)?;

    // Reject duplicate task in the same roadmap.
    if find_existing_task(&src, &task_slug) {
        return Err(CubilError::TaskAlreadyInRoadmap {
            roadmap: roadmap_slug,
            task: task_slug,
        });
    }

    let new_src = insert_task_line(&src, milestone.as_deref(), &roadmap_slug, &task_slug)?;
    write_atomic(&path, new_src.as_bytes())?;
    Ok(())
}

fn find_existing_task(src: &str, task_slug: &str) -> bool {
    for line in src.lines() {
        if let Some(parsed) = parse_task_line(line)
            && parsed.slug == task_slug
        {
            return true;
        }
    }
    false
}

fn insert_task_line(
    src: &str,
    milestone: Option<&str>,
    roadmap_slug: &str,
    task_slug: &str,
) -> Result<String> {
    let lines: Vec<&str> = src.split_inclusive('\n').collect();

    let section = match milestone {
        Some(name) => Some(find_named_milestone(&lines, name, roadmap_slug)?),
        None => find_last_milestone(&lines),
    };

    let insert_line_idx = match section {
        Some((start, end)) => insert_position_in_section(&lines, start, end),
        None => lines.len(),
    };

    let byte_offset: usize = lines[..insert_line_idx].iter().map(|l| l.len()).sum();

    let needs_leading_newline = byte_offset > 0 && src.as_bytes()[byte_offset - 1] != b'\n';

    let mut out = String::with_capacity(src.len() + task_slug.len() + 8);
    out.push_str(&src[..byte_offset]);
    if needs_leading_newline {
        out.push('\n');
    }
    out.push_str("- [ ] ");
    out.push_str(task_slug);
    out.push('\n');
    out.push_str(&src[byte_offset..]);
    Ok(out)
}

/// Find the line indices `(start, end)` of the section beginning at the named
/// milestone heading. Errors if missing or ambiguous.
fn find_named_milestone(
    lines: &[&str],
    target: &str,
    roadmap_slug: &str,
) -> Result<(usize, usize)> {
    let mut hits: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let stripped = strip_eol(line);
        if let Some(name) = parse_milestone_heading(stripped)
            && name == target
        {
            hits.push(i);
        }
    }
    match hits.len() {
        0 => Err(CubilError::MilestoneNotFound {
            roadmap: roadmap_slug.to_string(),
            milestone: target.to_string(),
            available: collect_milestone_names(lines),
        }),
        1 => {
            let start = hits[0];
            Ok((start, section_end(lines, start)))
        }
        _ => Err(CubilError::MilestoneAmbiguous {
            roadmap: roadmap_slug.to_string(),
            milestone: target.to_string(),
        }),
    }
}

fn find_last_milestone(lines: &[&str]) -> Option<(usize, usize)> {
    let mut last: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if parse_milestone_heading(strip_eol(line)).is_some() {
            last = Some(i);
        }
    }
    last.map(|start| (start, section_end(lines, start)))
}

/// A section runs from its `## ` heading line (inclusive) to the next `## `
/// heading line (exclusive), or end of file.
fn section_end(lines: &[&str], start: usize) -> usize {
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        if parse_h2_heading(strip_eol(line)).is_some() {
            return i;
        }
    }
    lines.len()
}

/// Insertion line index: just after the last non-blank line in the section
/// (excluding the heading itself). If the section has no body content, insert
/// right after the heading.
fn insert_position_in_section(lines: &[&str], start: usize, end: usize) -> usize {
    let mut last_non_blank = start;
    for (i, line) in lines.iter().enumerate().take(end).skip(start + 1) {
        if !strip_eol(line).trim().is_empty() {
            last_non_blank = i;
        }
    }
    last_non_blank + 1
}

fn collect_milestone_names(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|l| parse_milestone_heading(strip_eol(l)).map(str::to_string))
        .collect()
}

fn strip_eol(s: &str) -> &str {
    s.trim_end_matches('\n').trim_end_matches('\r')
}

/// Atomic-ish write: write to a tempfile in the same directory and rename.
/// Survives crashes mid-write — readers never see a half-written roadmap.
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

    #[test]
    fn insert_into_file_with_no_milestones_appends_at_eof() {
        let src = "# Roadmap\n\nNarrative.\n";
        let out = insert_task_line(src, None, "r", "task-1").unwrap();
        assert_eq!(out, "# Roadmap\n\nNarrative.\n- [ ] task-1\n");
    }

    #[test]
    fn insert_into_file_with_no_trailing_newline_adds_one() {
        let src = "# Roadmap";
        let out = insert_task_line(src, None, "r", "task-1").unwrap();
        assert_eq!(out, "# Roadmap\n- [ ] task-1\n");
    }

    #[test]
    fn default_appends_to_last_milestone() {
        let src = "\
# Roadmap

## Milestone: A
- [ ] task-1

## Milestone: B
- [ ] task-3
";
        let out = insert_task_line(src, None, "r", "task-4").unwrap();
        let expected = "\
# Roadmap

## Milestone: A
- [ ] task-1

## Milestone: B
- [ ] task-3
- [ ] task-4
";
        assert_eq!(out, expected);
    }

    #[test]
    fn named_milestone_inserts_before_trailing_blank() {
        let src = "\
# Roadmap

## Milestone: A
- [ ] task-1
- [ ] task-2

## Milestone: B
- [ ] task-3
";
        let out = insert_task_line(src, Some("A"), "r", "task-x").unwrap();
        let expected = "\
# Roadmap

## Milestone: A
- [ ] task-1
- [ ] task-2
- [ ] task-x

## Milestone: B
- [ ] task-3
";
        assert_eq!(out, expected);
    }

    #[test]
    fn named_milestone_with_no_body_inserts_right_after_heading() {
        let src = "\
## Milestone: A
## Milestone: B
- [ ] task-3
";
        let out = insert_task_line(src, Some("A"), "r", "task-x").unwrap();
        let expected = "\
## Milestone: A
- [ ] task-x
## Milestone: B
- [ ] task-3
";
        assert_eq!(out, expected);
    }

    #[test]
    fn named_milestone_missing_lists_available() {
        let src = "## Milestone: A\n- [ ] x\n## Milestone: B\n";
        let err = insert_task_line(src, Some("Z"), "rmap", "task-x").unwrap_err();
        match err {
            CubilError::MilestoneNotFound {
                roadmap,
                milestone,
                available,
            } => {
                assert_eq!(roadmap, "rmap");
                assert_eq!(milestone, "Z");
                assert_eq!(available, vec!["A".to_string(), "B".to_string()]);
            }
            other => panic!("expected MilestoneNotFound, got {other:?}"),
        }
    }

    #[test]
    fn named_milestone_ambiguous_errors() {
        let src = "## Milestone: A\n- [ ] x\n## Milestone: A\n";
        let err = insert_task_line(src, Some("A"), "rmap", "task-x").unwrap_err();
        assert!(matches!(err, CubilError::MilestoneAmbiguous { .. }));
    }

    #[test]
    fn case_sensitive_milestone_match() {
        let src = "## Milestone: Schema ready\n";
        let err = insert_task_line(src, Some("schema ready"), "rmap", "task-x").unwrap_err();
        assert!(matches!(err, CubilError::MilestoneNotFound { .. }));
    }

    #[test]
    fn find_existing_task_detects_anywhere() {
        let src = "## Milestone: A\n- [ ] foo\n## Milestone: B\n- [✓] bar — Title\n";
        assert!(find_existing_task(src, "foo"));
        assert!(find_existing_task(src, "bar"));
        assert!(!find_existing_task(src, "baz"));
    }

    #[test]
    fn find_existing_task_ignores_non_slug_checkboxes() {
        // A free-form checkbox like "- [ ] todo something" should not be
        // treated as a task slug — its first whitespace-delimited token is
        // not slug-shaped.
        let src = "- [ ] write the docs\n";
        assert!(!find_existing_task(src, "write"));
    }

    #[test]
    fn insert_into_section_with_narrative_no_tasks() {
        let src = "## Milestone: A\nSome notes.\n## Milestone: B\n";
        let out = insert_task_line(src, Some("A"), "r", "task-x").unwrap();
        let expected = "## Milestone: A\nSome notes.\n- [ ] task-x\n## Milestone: B\n";
        assert_eq!(out, expected);
    }
}
