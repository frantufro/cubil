use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};
use crate::core::task_doc::{TaskDoc, parse_top_heading};

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

/// Task details a caller supplies for a slug when rendering a roadmap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub title: Option<String>,
}

/// A line parsed as a checkbox-style task reference.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TaskLine {
    indent: String,
    slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LineKind {
    Task(TaskLine),
    Milestone(String),
    /// Any other `## ` heading; it closes the preceding Milestone section.
    Heading,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    /// Raw text including its line terminator.
    raw: String,
    kind: LineKind,
}

impl Line {
    fn body(&self) -> &str {
        strip_eol(&self.raw)
    }

    fn eol(&self) -> &str {
        &self.raw[self.body().len()..]
    }
}

fn strip_eol(s: &str) -> &str {
    s.trim_end_matches('\n').trim_end_matches('\r')
}

/// A roadmap file parsed once into classified lines.
///
/// The task-line and Milestone grammar lives here. Every operation works on
/// the parsed lines and leaves non-task content byte-for-byte intact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roadmap {
    slug: String,
    lines: Vec<Line>,
}

impl Roadmap {
    /// Parse the contents of the roadmap file `slug`. The slug appears in
    /// error messages.
    pub fn parse(slug: &str, src: &str) -> Self {
        let lines = src
            .split_inclusive('\n')
            .map(|raw| {
                let body = strip_eol(raw);
                let kind = if let Some(t) = parse_task_line(body) {
                    LineKind::Task(t)
                } else if let Some(name) = parse_milestone_heading(body) {
                    LineKind::Milestone(name.to_string())
                } else if parse_h2_heading(body).is_some() {
                    LineKind::Heading
                } else {
                    LineKind::Other
                };
                Line {
                    raw: raw.to_string(),
                    kind,
                }
            })
            .collect();
        Roadmap {
            slug: slug.to_string(),
            lines,
        }
    }

    /// Read and parse the roadmap file at `path`.
    pub fn load(slug: &str, path: &Path) -> Result<Self> {
        Ok(Self::parse(slug, &std::fs::read_to_string(path)?))
    }

    /// The roadmap title: its first `# ` heading.
    pub fn title(&self) -> Option<&str> {
        self.lines.iter().find_map(|l| parse_top_heading(l.body()))
    }

    /// The file contents, with any edits applied.
    pub fn render(&self) -> String {
        self.lines.iter().map(|l| l.raw.as_str()).collect()
    }

    /// Render with every task line rewritten from `tasks`, which returns the
    /// status and title for a slug.
    ///
    /// - Done task: `- [✓] <slug> — <title>`
    /// - Doing: `- [~] <slug> — <title>`
    /// - Backlog or any other status folder: `- [ ] <slug> — <title>`
    /// - Slug with no task file: `- [?] <slug> — (missing)`
    ///
    /// Non-task content passes through byte-for-byte.
    pub fn render_resolved(
        &self,
        mut tasks: impl FnMut(&str) -> Result<TaskInfo>,
    ) -> Result<String> {
        let mut out = String::new();
        for line in &self.lines {
            match &line.kind {
                LineKind::Task(t) => {
                    let info = tasks(&t.slug)?;
                    out.push_str(&render_task_line(&t.indent, &t.slug, &info));
                    out.push_str(line.eol());
                }
                _ => out.push_str(&line.raw),
            }
        }
        Ok(out)
    }

    /// Whether the roadmap lists `task_slug` as a task line.
    pub fn contains_task(&self, task_slug: &str) -> bool {
        self.lines
            .iter()
            .any(|l| matches!(&l.kind, LineKind::Task(t) if t.slug == task_slug))
    }

    /// Append `- [ ] <task_slug>` to a Milestone section.
    ///
    /// With `milestone == None` the line goes at the end of the last
    /// Milestone, or at end of file when there are none. With `Some(name)`
    /// it goes at the end of that Milestone (an error if it is missing or
    /// ambiguous). Errors if the task is already listed.
    pub fn add_task(&mut self, task_slug: &str, milestone: Option<&str>) -> Result<()> {
        if self.contains_task(task_slug) {
            return Err(CubilError::TaskAlreadyInRoadmap {
                roadmap: self.slug.clone(),
                task: task_slug.to_string(),
            });
        }
        let section = match milestone {
            Some(name) => Some(self.find_named_milestone(name)?),
            None => self.find_last_milestone(),
        };
        let at = match section {
            Some((start, end)) => self.insert_position_in_section(start, end),
            None => self.lines.len(),
        };
        // A final line without a terminator gets one before the new line.
        if at > 0 && at == self.lines.len() && self.lines[at - 1].eol().is_empty() {
            self.lines[at - 1].raw.push('\n');
        }
        self.lines.insert(
            at,
            Line {
                raw: format!("- [ ] {task_slug}\n"),
                kind: LineKind::Task(TaskLine {
                    indent: String::new(),
                    slug: task_slug.to_string(),
                }),
            },
        );
        Ok(())
    }

    /// Slug of the first task line, in file order, whose task exists and is
    /// not done. `tasks` returns the status for a slug. Missing slugs are
    /// skipped, since `next` drives `cubil start <slug>`.
    pub fn next_not_done(&self, mut tasks: impl FnMut(&str) -> TaskStatus) -> Option<&str> {
        self.lines.iter().find_map(|l| match &l.kind {
            LineKind::Task(t) => {
                let status = tasks(&t.slug);
                (!status.is_missing() && !status.is_done()).then_some(t.slug.as_str())
            }
            _ => None,
        })
    }

    /// Write the roadmap to `path` atomically: a temp file in the same
    /// directory is renamed over the target, so readers never see a partial
    /// file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let parent = path.parent().expect("roadmap path has parent");
        let tmp = parent.join(format!(
            ".{}.tmp",
            path.file_name()
                .expect("roadmap has filename")
                .to_string_lossy()
        ));
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(self.render().as_bytes())?;
        drop(f);
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    fn milestone_names(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter_map(|l| match &l.kind {
                LineKind::Milestone(n) => Some(n.clone()),
                _ => None,
            })
            .collect()
    }

    /// Line indices `(start, end)` of the section under the named Milestone.
    fn find_named_milestone(&self, target: &str) -> Result<(usize, usize)> {
        let hits: Vec<usize> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| matches!(&l.kind, LineKind::Milestone(n) if n == target))
            .map(|(i, _)| i)
            .collect();
        match hits.as_slice() {
            [] => Err(CubilError::MilestoneNotFound {
                roadmap: self.slug.clone(),
                milestone: target.to_string(),
                available: self.milestone_names(),
            }),
            [start] => Ok((*start, self.section_end(*start))),
            _ => Err(CubilError::MilestoneAmbiguous {
                roadmap: self.slug.clone(),
                milestone: target.to_string(),
            }),
        }
    }

    fn find_last_milestone(&self) -> Option<(usize, usize)> {
        let start = self
            .lines
            .iter()
            .rposition(|l| matches!(l.kind, LineKind::Milestone(_)))?;
        Some((start, self.section_end(start)))
    }

    /// A section runs from its `## ` heading (inclusive) to the next `## `
    /// heading (exclusive), or end of file.
    fn section_end(&self, start: usize) -> usize {
        self.lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, l)| matches!(l.kind, LineKind::Milestone(_) | LineKind::Heading))
            .map_or(self.lines.len(), |(i, _)| i)
    }

    /// Insertion index: just after the last non-blank line of the section
    /// body, or right after the heading when the body is empty.
    fn insert_position_in_section(&self, start: usize, end: usize) -> usize {
        let mut last_non_blank = start;
        for (i, line) in self.lines.iter().enumerate().take(end).skip(start + 1) {
            if !line.body().trim().is_empty() {
                last_non_blank = i;
            }
        }
        last_non_blank + 1
    }
}

fn render_task_line(indent: &str, slug: &str, info: &TaskInfo) -> String {
    let checkbox = info.status.checkbox();
    let suffix = if info.status.is_missing() {
        "(missing)"
    } else {
        info.title.as_deref().unwrap_or("(untitled)")
    };
    format!("{indent}- [{checkbox}] {slug} \u{2014} {suffix}")
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
fn parse_task_line(line: &str) -> Option<TaskLine> {
    let indent_end = line
        .find(|c: char| c != ' ' && c != '\t')
        .unwrap_or(line.len());
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
fn parse_milestone_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(MILESTONE_PREFIX)?;
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// If `line` is any `## ` heading, return its text. Used to find section
/// boundaries (a Milestone section ends at the next `## ` heading).
fn parse_h2_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("## ")?;
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Read the title of a task file. Thin wrapper over [`TaskDoc::title`].
pub fn read_task_title(path: &Path) -> Result<Option<String>> {
    let src = std::fs::read_to_string(path)?;
    Ok(TaskDoc::parse(&src).title().map(str::to_string))
}

/// Extract the roadmap title (first `# ` heading) from raw file contents.
pub fn extract_roadmap_title(src: &str) -> Option<String> {
    Roadmap::parse("", src).title().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rm(src: &str) -> Roadmap {
        Roadmap::parse("rmap", src)
    }

    fn info(status: TaskStatus, title: Option<&str>) -> TaskInfo {
        TaskInfo {
            status,
            title: title.map(String::from),
        }
    }

    fn resolver(items: &[(&str, &str, Option<&str>)]) -> impl FnMut(&str) -> Result<TaskInfo> {
        let items: Vec<(String, String, Option<String>)> = items
            .iter()
            .map(|(s, st, t)| ((*s).into(), (*st).into(), t.map(String::from)))
            .collect();
        move |slug| {
            Ok(match items.iter().find(|(s, _, _)| s == slug) {
                Some((_, st, t)) => TaskInfo {
                    status: TaskStatus::from_status_name(st),
                    title: t.clone(),
                },
                None => info(TaskStatus::Missing, None),
            })
        }
    }

    fn statuses(items: &[(&str, &str)]) -> impl FnMut(&str) -> TaskStatus {
        let items: Vec<(String, String)> = items
            .iter()
            .map(|(s, t)| ((*s).into(), (*t).into()))
            .collect();
        move |slug| match items.iter().find(|(s, _)| s == slug) {
            Some((_, st)) => TaskStatus::from_status_name(st),
            None => TaskStatus::Missing,
        }
    }

    fn added(src: &str, milestone: Option<&str>, slug: &str) -> String {
        let mut r = rm(src);
        r.add_task(slug, milestone).unwrap();
        r.render()
    }

    // ---- line grammar ----

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
        assert_eq!(
            parse_milestone_heading("## Milestone: Schema ready"),
            Some("Schema ready")
        );
        assert_eq!(
            parse_milestone_heading("## Milestone:   Padded  "),
            Some("Padded")
        );
    }

    #[test]
    fn parse_milestone_heading_rejects_non_milestone() {
        assert!(parse_milestone_heading("# Title").is_none());
        assert!(parse_milestone_heading("## Other").is_none());
        assert!(parse_milestone_heading("## Milestone:").is_none());
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

    // ---- title ----

    #[test]
    fn title_finds_first_h1() {
        assert_eq!(
            rm("# Migrate\n\n## Milestone: A\n").title(),
            Some("Migrate")
        );
        assert_eq!(
            extract_roadmap_title("# Migrate\n").as_deref(),
            Some("Migrate")
        );
    }

    #[test]
    fn title_none_when_absent() {
        assert_eq!(rm("## Milestone: A\n- [ ] x\n").title(), None);
    }

    #[test]
    fn task_status_checkbox() {
        assert_eq!(TaskStatus::Backlog.checkbox(), ' ');
        assert_eq!(TaskStatus::Doing.checkbox(), '~');
        assert_eq!(TaskStatus::Done.checkbox(), '✓');
        assert_eq!(TaskStatus::Other.checkbox(), ' ');
        assert_eq!(TaskStatus::Missing.checkbox(), '?');
    }

    // ---- render_resolved ----

    #[test]
    fn render_resolved_resolves_all_status_markers() {
        let tasks = resolver(&[
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
        let expected = "\
# Roadmap

## Milestone: M
- [\u{2713}] a \u{2014} Task A
- [~] b \u{2014} Task B
- [ ] c \u{2014} Task C
- [?] missing-task \u{2014} (missing)
";
        assert_eq!(rm(src).render_resolved(tasks).unwrap(), expected);
    }

    #[test]
    fn render_resolved_preserves_non_task_lines_byte_for_byte() {
        let tasks = resolver(&[("a", "done", Some("A"))]);
        let src = "\
# Title

Some narrative.

## Milestone: First
Notes inside section.
- [ ] a
  - sub bullet
- not a checkbox
";
        let out = rm(src).render_resolved(tasks).unwrap();
        assert!(out.contains("Some narrative.\n"));
        assert!(out.contains("Notes inside section.\n"));
        assert!(out.contains("  - sub bullet\n"));
        assert!(out.contains("- not a checkbox\n"));
        assert!(out.contains("- [\u{2713}] a \u{2014} A\n"));
    }

    #[test]
    fn render_resolved_handles_already_resolved_lines() {
        let tasks = resolver(&[("a", "done", Some("New Title"))]);
        let out = rm("- [ ] a — Old Title\n").render_resolved(tasks).unwrap();
        assert_eq!(out, "- [\u{2713}] a \u{2014} New Title\n");
    }

    #[test]
    fn render_resolved_uses_untitled_when_task_has_no_h1() {
        let tasks = resolver(&[("a", "done", None)]);
        let out = rm("- [ ] a\n").render_resolved(tasks).unwrap();
        assert_eq!(out, "- [\u{2713}] a \u{2014} (untitled)\n");
    }

    #[test]
    fn render_resolved_treats_unknown_status_as_backlog_marker() {
        let tasks = resolver(&[("a", "review", Some("A"))]);
        let out = rm("- [✓] a — Old\n").render_resolved(tasks).unwrap();
        assert_eq!(out, "- [ ] a \u{2014} A\n");
    }

    #[test]
    fn render_resolved_keeps_crlf_endings() {
        let tasks = resolver(&[("a", "done", Some("A"))]);
        let out = rm("# T\r\n- [ ] a\r\n").render_resolved(tasks).unwrap();
        assert_eq!(out, "# T\r\n- [\u{2713}] a \u{2014} A\r\n");
    }

    // ---- add_task ----

    #[test]
    fn add_into_file_with_no_milestones_appends_at_eof() {
        let out = added("# Roadmap\n\nNarrative.\n", None, "task-1");
        assert_eq!(out, "# Roadmap\n\nNarrative.\n- [ ] task-1\n");
    }

    #[test]
    fn add_into_file_with_no_trailing_newline_adds_one() {
        assert_eq!(
            added("# Roadmap", None, "task-1"),
            "# Roadmap\n- [ ] task-1\n"
        );
    }

    #[test]
    fn add_default_appends_to_last_milestone() {
        let src = "\
# Roadmap

## Milestone: A
- [ ] task-1

## Milestone: B
- [ ] task-3
";
        let expected = "\
# Roadmap

## Milestone: A
- [ ] task-1

## Milestone: B
- [ ] task-3
- [ ] task-4
";
        assert_eq!(added(src, None, "task-4"), expected);
    }

    #[test]
    fn add_named_milestone_inserts_before_trailing_blank() {
        let src = "\
# Roadmap

## Milestone: A
- [ ] task-1
- [ ] task-2

## Milestone: B
- [ ] task-3
";
        let expected = "\
# Roadmap

## Milestone: A
- [ ] task-1
- [ ] task-2
- [ ] task-x

## Milestone: B
- [ ] task-3
";
        assert_eq!(added(src, Some("A"), "task-x"), expected);
    }

    #[test]
    fn add_named_milestone_with_no_body_inserts_right_after_heading() {
        let src = "## Milestone: A\n## Milestone: B\n- [ ] task-3\n";
        let expected = "## Milestone: A\n- [ ] task-x\n## Milestone: B\n- [ ] task-3\n";
        assert_eq!(added(src, Some("A"), "task-x"), expected);
    }

    #[test]
    fn add_stops_section_at_any_h2_heading() {
        let src = "## Milestone: A\n- [ ] one\n## Notes\nprose\n";
        let expected = "## Milestone: A\n- [ ] one\n- [ ] two\n## Notes\nprose\n";
        assert_eq!(added(src, None, "two"), expected);
    }

    #[test]
    fn add_named_milestone_missing_lists_available() {
        let src = "## Milestone: A\n- [ ] x\n## Milestone: B\n";
        let err = rm(src).add_task("task-x", Some("Z")).unwrap_err();
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
    fn add_named_milestone_ambiguous_errors() {
        let src = "## Milestone: A\n- [ ] x\n## Milestone: A\n";
        let err = rm(src).add_task("task-x", Some("A")).unwrap_err();
        assert!(matches!(err, CubilError::MilestoneAmbiguous { .. }));
    }

    #[test]
    fn add_milestone_match_is_case_sensitive() {
        let err = rm("## Milestone: Schema ready\n")
            .add_task("task-x", Some("schema ready"))
            .unwrap_err();
        assert!(matches!(err, CubilError::MilestoneNotFound { .. }));
    }

    #[test]
    fn add_rejects_task_already_listed() {
        let err = rm("## Milestone: A\n- [✓] foo — T\n")
            .add_task("foo", None)
            .unwrap_err();
        assert!(matches!(err, CubilError::TaskAlreadyInRoadmap { .. }));
    }

    #[test]
    fn add_into_section_with_narrative_no_tasks() {
        let src = "## Milestone: A\nSome notes.\n## Milestone: B\n";
        let expected = "## Milestone: A\nSome notes.\n- [ ] task-x\n## Milestone: B\n";
        assert_eq!(added(src, Some("A"), "task-x"), expected);
    }

    #[test]
    fn contains_task_detects_anywhere() {
        let r = rm("## Milestone: A\n- [ ] foo\n## Milestone: B\n- [✓] bar — Title\n");
        assert!(r.contains_task("foo"));
        assert!(r.contains_task("bar"));
        assert!(!r.contains_task("baz"));
    }

    #[test]
    fn contains_task_ignores_non_slug_checkboxes() {
        assert!(!rm("- [ ] write the docs\n").contains_task("write"));
    }

    // ---- next_not_done ----

    #[test]
    fn next_returns_first_backlog_task() {
        let r = rm("- [ ] a\n- [ ] b\n- [ ] c\n");
        let t = statuses(&[("a", "done"), ("b", "backlog"), ("c", "backlog")]);
        assert_eq!(r.next_not_done(t), Some("b"));
    }

    #[test]
    fn next_returns_first_doing_task_over_later_backlog() {
        let r = rm("- [ ] a\n- [ ] b\n- [ ] c\n");
        let t = statuses(&[("a", "done"), ("b", "doing"), ("c", "backlog")]);
        assert_eq!(r.next_not_done(t), Some("b"));
    }

    #[test]
    fn next_returns_none_when_all_done() {
        let r = rm("- [ ] a\n- [ ] b\n");
        assert_eq!(
            r.next_not_done(statuses(&[("a", "done"), ("b", "done")])),
            None
        );
    }

    #[test]
    fn next_returns_none_for_empty_roadmap() {
        assert_eq!(rm("# Just a title\n").next_not_done(statuses(&[])), None);
    }

    #[test]
    fn next_skips_missing_tasks() {
        let r = rm("- [ ] a\n- [ ] missing\n- [ ] c\n");
        let t = statuses(&[("a", "done"), ("c", "backlog")]);
        assert_eq!(r.next_not_done(t), Some("c"));
    }

    #[test]
    fn next_ignores_milestone_headings_and_narrative() {
        let src = "# Roadmap\n\nNarrative.\n\n## Milestone: M\n- [ ] first-task\n";
        let t = statuses(&[("first-task", "backlog")]);
        assert_eq!(rm(src).next_not_done(t), Some("first-task"));
    }

    // ---- save ----

    #[test]
    fn save_writes_edits_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("r.md");
        std::fs::write(&path, "## Milestone: A\n").unwrap();
        let mut r = Roadmap::load("r", &path).unwrap();
        r.add_task("x", None).unwrap();
        r.save(&path).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "## Milestone: A\n- [ ] x\n"
        );
        assert!(!dir.path().join(".r.md.tmp").exists());
    }
}
