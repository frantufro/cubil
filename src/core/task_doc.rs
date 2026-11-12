//! The Task document: one module that parses and renders a whole Task file.
//!
//! A Task file is optional YAML-ish frontmatter followed by a Markdown body
//! whose first `# ` heading is the title. Every rule about that layout lives
//! here: frontmatter parsing, the H1 title rule and the way `cubil new`
//! assembles a fresh file. Callers use [`TaskDoc`] and never touch the
//! delimiters or the heading syntax.

use std::fmt::Write as _;
use std::io::Read;
use std::path::Path;

use crate::core::error::Result;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TaskMeta {
    /// Raw ISO date string.
    pub created: Option<String>,
    pub priority: Option<u32>,
    /// Unknown `key: value` lines, preserved verbatim for round-tripping.
    pub extra: Vec<String>,
}

/// Parse a `---`-delimited YAML-ish frontmatter block from the start of a
/// markdown file. Only recognizes `key: value` lines; unknown keys go into
/// [`TaskMeta::extra`]. Returns `(TaskMeta, rest_of_file_starting_at_body)`.
/// If the file has no frontmatter (or the opening `---` has no matching
/// closing `---`), returns `(TaskMeta::default(), src)`.
fn parse_frontmatter(src: &str) -> (TaskMeta, &str) {
    let rest = if let Some(r) = src.strip_prefix("---\n") {
        r
    } else if let Some(r) = src.strip_prefix("---\r\n") {
        r
    } else {
        return (TaskMeta::default(), src);
    };

    let mut offset = 0usize;
    let mut fm_end: Option<usize> = None;
    let mut body_start: Option<usize> = None;
    for line in rest.split_inclusive('\n') {
        let content = line.trim_end_matches(['\r', '\n']);
        if content == "---" {
            fm_end = Some(offset);
            body_start = Some(offset + line.len());
            break;
        }
        offset += line.len();
    }

    let (Some(fm_end), Some(body_start)) = (fm_end, body_start) else {
        return (TaskMeta::default(), src);
    };

    let mut meta = TaskMeta::default();
    for line in rest[..fm_end].lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key_trimmed = key.trim();
            let value_trimmed = value.trim();
            match key_trimmed {
                "created" => meta.created = Some(value_trimmed.to_string()),
                "priority" => match value_trimmed.parse::<u32>() {
                    Ok(n) => meta.priority = Some(n),
                    Err(_) => meta.extra.push(line.to_string()),
                },
                _ => meta.extra.push(line.to_string()),
            }
        } else {
            meta.extra.push(line.to_string());
        }
    }

    (meta, &rest[body_start..])
}

/// Render a [`TaskMeta`] back to a `---`-delimited block, including the
/// trailing newline after the closing `---`.
///
/// Returns an empty string when the meta has no known fields and no extras,
/// so a file parsed with no frontmatter round-trips to itself. This means
/// `parse → render` is only lossless when the input actually had a
/// frontmatter block; rendering a default-constructed [`TaskMeta`] never
/// synthesizes one.
fn render_frontmatter(meta: &TaskMeta) -> String {
    if meta.created.is_none() && meta.priority.is_none() && meta.extra.is_empty() {
        return String::new();
    }
    let mut out = String::from("---\n");
    if let Some(c) = &meta.created {
        let _ = writeln!(out, "created: {c}");
    }
    if let Some(p) = meta.priority {
        let _ = writeln!(out, "priority: {p}");
    }
    for line in &meta.extra {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("---\n");
    out
}

/// If `line` is a top-level `# <title>` heading, return the title (trimmed).
pub(crate) fn parse_top_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("# ")?;
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Title of the first `# ` heading in `src`, if any. Shared with roadmap
/// files, which use the same heading rule.
pub fn first_h1(src: &str) -> Option<&str> {
    src.lines().find_map(parse_top_heading)
}

/// `# <title>` followed by the optional body, ending in a newline. An empty
/// body adds nothing; a non-empty body is separated by one blank line and
/// keeps its own trailing newlines.
pub fn titled_body(title: &str, body: &str) -> String {
    let mut out = String::new();
    out.push_str("# ");
    out.push_str(title);
    out.push('\n');
    if !body.is_empty() {
        out.push('\n');
        out.push_str(body);
        if !body.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

/// Resolve the body for `new` commands: `-m` text, `-F <path>`, `-F -`
/// (stdin), or empty when neither is given.
pub fn read_body(message: Option<String>, file: Option<&Path>) -> Result<String> {
    if let Some(m) = message {
        return Ok(m);
    }
    let Some(path) = file else {
        return Ok(String::new());
    };
    if path.as_os_str() == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    Ok(std::fs::read_to_string(path)?)
}

/// A parsed Task file: its metadata plus everything after the frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDoc {
    meta: TaskMeta,
    /// Everything after the frontmatter block, verbatim.
    rest: String,
}

impl TaskDoc {
    /// A new Task document created on `date` (ISO `YYYY-MM-DD`).
    pub fn new(title: &str, body: &str, date: &str) -> Self {
        let meta = TaskMeta {
            created: Some(date.to_string()),
            priority: None,
            extra: Vec::new(),
        };
        TaskDoc {
            meta,
            rest: format!("\n{}", titled_body(title, body)),
        }
    }

    /// Parse the text of a Task file. Never fails: text without frontmatter
    /// yields default metadata and the whole text as the rest.
    pub fn parse(text: &str) -> Self {
        let (meta, rest) = parse_frontmatter(text);
        TaskDoc {
            meta,
            rest: rest.to_string(),
        }
    }

    /// Title from the first `# ` heading, if the file has one.
    pub fn title(&self) -> Option<&str> {
        first_h1(&self.rest)
    }

    pub fn meta(&self) -> &TaskMeta {
        &self.meta
    }

    /// The file text. Parsing and rendering a file is lossless.
    pub fn render(&self) -> String {
        let mut out = render_frontmatter(&self.meta);
        out.push_str(&self.rest);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_frontmatter_via_doc(src: &str) -> (TaskMeta, String) {
        let doc = TaskDoc::parse(src);
        (doc.meta.clone(), doc.rest)
    }

    #[test]
    fn no_frontmatter_returns_default_and_full_input() {
        let src = "# Just a title\n\nBody here.\n";
        let (meta, body) = parse_frontmatter_via_doc(src);
        assert_eq!(meta, TaskMeta::default());
        assert_eq!(body, src);
    }

    #[test]
    fn parses_created_and_priority() {
        let src = "---\ncreated: 2026-04-19\npriority: 2\n---\nBody\n";
        let (meta, body) = parse_frontmatter_via_doc(src);
        assert_eq!(meta.created.as_deref(), Some("2026-04-19"));
        assert_eq!(meta.priority, Some(2));
        assert_eq!(meta.extra, Vec::<String>::new());
        assert_eq!(body, "Body\n");
    }

    #[test]
    fn preserves_unknown_keys_in_extra() {
        let src = "---\ncreated: 2026-04-19\nowner: alice\ntags: foo,bar\n---\n";
        let (meta, body) = parse_frontmatter_via_doc(src);
        assert_eq!(meta.created.as_deref(), Some("2026-04-19"));
        assert_eq!(meta.extra, vec!["owner: alice", "tags: foo,bar"]);
        assert_eq!(body, "");
    }

    #[test]
    fn round_trip_preserves_known_and_extra() {
        let original = "---\ncreated: 2026-04-19\npriority: 1\nowner: alice\n---\nhello\n";
        let (meta, body) = parse_frontmatter_via_doc(original);
        let _ = (meta, body);
        assert_eq!(TaskDoc::parse(original).render(), original);
    }

    #[test]
    fn missing_closing_delimiter_treats_as_no_frontmatter() {
        let src = "---\ncreated: 2026-04-19\nno closing here\n";
        let (meta, body) = parse_frontmatter_via_doc(src);
        assert_eq!(meta, TaskMeta::default());
        assert_eq!(body, src);
    }

    #[test]
    fn invalid_priority_falls_into_extra() {
        let src = "---\npriority: high\n---\n";
        let (meta, _) = parse_frontmatter_via_doc(src);
        assert_eq!(meta.priority, None);
        assert_eq!(meta.extra, vec!["priority: high"]);
    }

    #[test]
    fn render_default_meta_is_empty() {
        assert_eq!(render_frontmatter(&TaskMeta::default()), "");
    }

    #[test]
    fn round_trip_no_frontmatter_is_lossless() {
        let src = "# Just a title\n\nBody here.\n";
        let (meta, body) = parse_frontmatter_via_doc(src);
        let _ = (meta, body);
        assert_eq!(TaskDoc::parse(src).render(), src);
    }

    #[test]
    fn value_preserves_embedded_colons() {
        // `split_once(':')` returns the suffix after the *first* colon, so
        // ISO-8601 timestamps with `:` separators survive intact. This test
        // pins that: any change that starts splitting on every colon (e.g.
        // `split(':')`) would truncate values and break this assertion.
        let src = "---\ncreated: 2026-04-19T00:00:00Z\n---\n";
        let (meta, _) = parse_frontmatter_via_doc(src);
        assert_eq!(meta.created.as_deref(), Some("2026-04-19T00:00:00Z"));
    }

    #[test]
    fn new_empty_body_has_no_trailing_blank_line() {
        let doc = TaskDoc::new("Hello World", "", "2026-04-19");
        assert_eq!(
            doc.render(),
            "---\ncreated: 2026-04-19\n---\n\n# Hello World\n"
        );
    }

    #[test]
    fn new_with_body_adds_trailing_newline_when_missing() {
        let doc = TaskDoc::new("T", "line", "2026-04-19");
        assert_eq!(
            doc.render(),
            "---\ncreated: 2026-04-19\n---\n\n# T\n\nline\n"
        );
    }

    #[test]
    fn new_preserves_body_trailing_newlines() {
        let doc = TaskDoc::new("T", "line\n\n", "2026-04-19");
        assert_eq!(
            doc.render(),
            "---\ncreated: 2026-04-19\n---\n\n# T\n\nline\n\n"
        );
    }

    #[test]
    fn new_then_parse_round_trips_title_and_meta() {
        let doc = TaskDoc::new("Ship it", "Some body.", "2026-04-19");
        let parsed = TaskDoc::parse(&doc.render());
        assert_eq!(parsed, doc);
        assert_eq!(parsed.title(), Some("Ship it"));
        assert_eq!(parsed.meta().created.as_deref(), Some("2026-04-19"));
        assert_eq!(parsed.render(), doc.render());
    }

    #[test]
    fn title_reads_first_h1_after_frontmatter() {
        let doc = TaskDoc::parse("---\ncreated: 2026-04-19\n---\n\nintro\n# First\n# Second\n");
        assert_eq!(doc.title(), Some("First"));
    }

    #[test]
    fn title_is_none_without_h1() {
        assert_eq!(TaskDoc::parse("## Sub\n\ntext\n").title(), None);
        assert_eq!(TaskDoc::parse("# \n").title(), None);
    }

    #[test]
    fn titled_body_has_no_frontmatter() {
        assert_eq!(titled_body("Migrate", ""), "# Migrate\n");
        assert_eq!(
            titled_body("Migrate", "Narrative."),
            "# Migrate\n\nNarrative.\n"
        );
        assert_eq!(titled_body("Migrate", "line\n\n"), "# Migrate\n\nline\n\n");
    }

    #[test]
    fn first_h1_rejects_other_headings() {
        assert_eq!(
            first_h1("# Migrate to Postgres"),
            Some("Migrate to Postgres")
        );
        assert_eq!(first_h1("## Sub\nTitle\n# \n"), None);
    }

    #[test]
    fn read_body_prefers_message_and_defaults_to_empty() {
        assert_eq!(read_body(Some("hi".into()), None).unwrap(), "hi");
        assert_eq!(read_body(None, None).unwrap(), "");
    }
}
