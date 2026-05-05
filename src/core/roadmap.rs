use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};
use crate::core::root::find_root;

pub const ROADMAPS_DIR: &str = "roadmaps";
pub const MILESTONE_PREFIX: &str = "## Milestone: ";

/// Resolve a roadmap slug to its file path under `.cubil/roadmaps/`. Errors
/// with [`CubilError::RoadmapNotFound`] if the file does not exist.
pub fn resolve_roadmap(root: &Path, slug: &str) -> Result<PathBuf> {
    let path = roadmap_path(root, slug);
    if !path.is_file() {
        return Err(CubilError::RoadmapNotFound(slug.to_string()));
    }
    Ok(path)
}

pub fn roadmap_path(root: &Path, slug: &str) -> PathBuf {
    root.join(ROADMAPS_DIR).join(format!("{slug}.md"))
}

/// Find the `.cubil/` root and ensure the `roadmaps/` subdirectory exists.
/// Returns the absolute path to the roadmaps directory.
pub fn ensure_roadmaps_dir(root: &Path) -> Result<PathBuf> {
    let dir = root.join(ROADMAPS_DIR);
    if !dir.is_dir() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// List all roadmap slugs (filenames without `.md` suffix) under the given
/// `.cubil/` root. Returns an empty vector if the `roadmaps/` directory does
/// not exist. Order is unspecified — callers sort as needed.
pub fn list_roadmap_slugs(root: &Path) -> Result<Vec<String>> {
    let dir = root.join(ROADMAPS_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut slugs = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let fname = entry.file_name();
        let Some(s) = fname.to_str() else {
            continue;
        };
        if let Some(slug) = s.strip_suffix(".md") {
            slugs.push(slug.to_string());
        }
    }
    Ok(slugs)
}

/// Walk upward to find `.cubil/`, mirroring [`find_root`]. Convenience for
/// roadmap commands that all start with the same dance.
pub fn root_for_roadmap() -> Result<PathBuf> {
    find_root(None)
}

/// Resolved status of a task slug referenced by a roadmap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Backlog,
    Doing,
    Done,
    /// Task lives in a status folder other than the three canonical ones.
    Other,
    /// No task file with this slug exists in any status folder.
    Missing,
}

impl TaskStatus {
    pub fn from_status_name(name: &str) -> Self {
        match name {
            "backlog" => TaskStatus::Backlog,
            "doing" => TaskStatus::Doing,
            "done" => TaskStatus::Done,
            _ => TaskStatus::Other,
        }
    }

    /// Marker character rendered between the brackets in `- [<x>] slug`.
    pub fn checkbox(&self) -> char {
        match self {
            TaskStatus::Done => '✓',
            TaskStatus::Doing => '~',
            TaskStatus::Backlog | TaskStatus::Other => ' ',
            TaskStatus::Missing => '?',
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self, TaskStatus::Done)
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, TaskStatus::Missing)
    }
}

/// A line parsed as a checkbox-style task reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLine {
    pub indent: String,
    pub slug: String,
}

/// Recognize `<indent>- [<x>] <slug>` optionally followed by ` — <anything>`.
///
/// Two layers of strictness protect free-form `- [ ] write the docs` checkbox
/// items in roadmap narratives from being silently rewritten as task
/// references:
///
/// 1. The slug must be slug-shaped — lowercase ASCII alphanumerics joined by
///    single `-`, no leading/trailing dash.
/// 2. The text after the slug (whitespace-trimmed) must be empty or start
///    with `—` (U+2014, em-dash). This is the form `add` and `show` write,
///    so anything else is treated as prose.
pub fn parse_task_line(line: &str) -> Option<TaskLine> {
    let indent_end = line.find(|c: char| c != ' ' && c != '\t').unwrap_or(line.len());
    let (indent, rest) = line.split_at(indent_end);
    let rest = rest.strip_prefix("- [")?;
    let mut chars = rest.chars();
    let _checkbox = chars.next()?;
    let after_checkbox = chars.as_str();
    let after_bracket = after_checkbox.strip_prefix("] ")?;
    let slug_end = after_bracket
        .find(|c: char| c.is_whitespace())
        .unwrap_or(after_bracket.len());
    let slug = &after_bracket[..slug_end];
    if !is_slug_shaped(slug) {
        return None;
    }
    let suffix = after_bracket[slug_end..].trim_start();
    if !suffix.is_empty() && !suffix.starts_with('\u{2014}') {
        return None;
    }
    Some(TaskLine {
        indent: indent.to_string(),
        slug: slug.to_string(),
    })
}

fn is_slug_shaped(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return false;
    }
    let mut prev_dash = false;
    for &b in bytes {
        if b == b'-' {
            if prev_dash {
                return false;
            }
            prev_dash = true;
        } else if b.is_ascii_lowercase() || b.is_ascii_digit() {
            prev_dash = false;
        } else {
            return false;
        }
    }
    true
}

/// If `line` is a `## Milestone: <name>` heading, return the name (trimmed).
pub fn parse_milestone_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(MILESTONE_PREFIX)?;
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// If `line` is a top-level `# <title>` heading, return the title (trimmed).
pub fn parse_top_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("# ")?;
    let trimmed = rest.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

/// If `line` is any `## ` heading, return its text. Used to find section
/// boundaries (a milestone section ends at the next `## ` heading).
pub fn parse_h2_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("## ")?;
    let trimmed = rest.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

/// Read the first `# <title>` heading from a task file. Returns `None` if no
/// such heading exists. Used by roadmap rendering to enrich task references.
pub fn read_task_title(path: &Path) -> Result<Option<String>> {
    let src = std::fs::read_to_string(path)?;
    Ok(extract_first_h1(&src).map(str::to_string))
}

fn extract_first_h1(src: &str) -> Option<&str> {
    for line in src.lines() {
        if let Some(t) = parse_top_heading(line) {
            return Some(t);
        }
    }
    None
}

/// Extract the roadmap title (first `# ` heading) from raw file contents.
pub fn extract_roadmap_title(src: &str) -> Option<String> {
    extract_first_h1(src).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_task_line_bare() {
        let l = parse_task_line("- [ ] foo").unwrap();
        assert_eq!(l.indent, "");
        assert_eq!(l.slug, "foo");
    }

    #[test]
    fn parse_task_line_with_indent() {
        let l = parse_task_line("  - [✓] my-slug — Title").unwrap();
        assert_eq!(l.indent, "  ");
        assert_eq!(l.slug, "my-slug");
    }

    #[test]
    fn parse_task_line_recognizes_all_checkboxes() {
        for c in [' ', '~', '✓', '?', 'x', 'X'] {
            let line = format!("- [{c}] foo");
            assert!(parse_task_line(&line).is_some(), "should parse: {line}");
        }
    }

    #[test]
    fn parse_task_line_rejects_non_slug_text() {
        assert!(parse_task_line("- [ ] Buy milk").is_none());
        assert!(parse_task_line("- [ ] UPPER").is_none());
        assert!(parse_task_line("- [ ] -leading-dash").is_none());
        assert!(parse_task_line("- [ ] trailing-dash-").is_none());
        assert!(parse_task_line("- [ ] double--dash").is_none());
    }

    #[test]
    fn parse_task_line_rejects_prose_after_slug_shaped_word() {
        // Free-form prose checkbox: first token is slug-shaped but suffix
        // isn't an em-dash title. Should NOT be treated as a task reference.
        assert!(parse_task_line("- [ ] write the docs").is_none());
        assert!(parse_task_line("- [ ] foo bar baz").is_none());
    }

    #[test]
    fn parse_task_line_accepts_em_dash_title() {
        let l = parse_task_line("- [✓] foo \u{2014} The Foo Title").unwrap();
        assert_eq!(l.slug, "foo");
    }

    #[test]
    fn parse_task_line_rejects_non_checkbox_lines() {
        assert!(parse_task_line("- foo").is_none());
        assert!(parse_task_line("foo").is_none());
        assert!(parse_task_line("# foo").is_none());
        assert!(parse_task_line("## Milestone: foo").is_none());
    }

    #[test]
    fn parse_milestone_heading_recognizes() {
        assert_eq!(parse_milestone_heading("## Milestone: Schema ready"), Some("Schema ready"));
        assert_eq!(parse_milestone_heading("## Milestone:   Padded  "), Some("Padded"));
    }

    #[test]
    fn parse_milestone_heading_rejects_non_milestone() {
        assert!(parse_milestone_heading("# Title").is_none());
        assert!(parse_milestone_heading("## Other").is_none());
        assert!(parse_milestone_heading("## Milestone:").is_none());
    }

    #[test]
    fn parse_top_heading_recognizes() {
        assert_eq!(parse_top_heading("# Migrate to Postgres"), Some("Migrate to Postgres"));
    }

    #[test]
    fn parse_top_heading_rejects_other() {
        assert!(parse_top_heading("## Sub").is_none());
        assert!(parse_top_heading("Title").is_none());
        assert!(parse_top_heading("# ").is_none());
    }

    #[test]
    fn extract_roadmap_title_finds_first_h1() {
        let src = "# Migrate\n\n## Milestone: A\n";
        assert_eq!(extract_roadmap_title(src).as_deref(), Some("Migrate"));
    }

    #[test]
    fn extract_roadmap_title_none_when_absent() {
        let src = "## Milestone: A\n- [ ] x\n";
        assert_eq!(extract_roadmap_title(src), None);
    }

    #[test]
    fn task_status_checkbox() {
        assert_eq!(TaskStatus::Backlog.checkbox(), ' ');
        assert_eq!(TaskStatus::Doing.checkbox(), '~');
        assert_eq!(TaskStatus::Done.checkbox(), '✓');
        assert_eq!(TaskStatus::Other.checkbox(), ' ');
        assert_eq!(TaskStatus::Missing.checkbox(), '?');
    }

    #[test]
    fn is_slug_shaped_accepts_normal() {
        assert!(is_slug_shaped("foo"));
        assert!(is_slug_shaped("foo-bar"));
        assert!(is_slug_shaped("a1b2"));
        assert!(is_slug_shaped("foo-bar-baz-1-2-3"));
    }

    #[test]
    fn is_slug_shaped_rejects_bad() {
        assert!(!is_slug_shaped(""));
        assert!(!is_slug_shaped("Foo"));
        assert!(!is_slug_shaped("foo bar"));
        assert!(!is_slug_shaped("-foo"));
        assert!(!is_slug_shaped("foo-"));
        assert!(!is_slug_shaped("foo--bar"));
    }
}
