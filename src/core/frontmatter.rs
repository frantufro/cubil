use std::fmt::Write as _;

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
pub fn parse_frontmatter(src: &str) -> (TaskMeta, &str) {
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
pub fn render_frontmatter(meta: &TaskMeta) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_returns_default_and_full_input() {
        let src = "# Just a title\n\nBody here.\n";
        let (meta, body) = parse_frontmatter(src);
        assert_eq!(meta, TaskMeta::default());
        assert_eq!(body, src);
    }

    #[test]
    fn parses_created_and_priority() {
        let src = "---\ncreated: 2026-04-19\npriority: 2\n---\nBody\n";
        let (meta, body) = parse_frontmatter(src);
        assert_eq!(meta.created.as_deref(), Some("2026-04-19"));
        assert_eq!(meta.priority, Some(2));
        assert!(meta.extra.is_empty());
        assert_eq!(body, "Body\n");
    }

    #[test]
    fn preserves_unknown_keys_in_extra() {
        let src = "---\ncreated: 2026-04-19\nowner: alice\ntags: foo,bar\n---\n";
        let (meta, body) = parse_frontmatter(src);
        assert_eq!(meta.created.as_deref(), Some("2026-04-19"));
        assert_eq!(meta.extra, vec!["owner: alice", "tags: foo,bar"]);
        assert_eq!(body, "");
    }

    #[test]
    fn round_trip_preserves_known_and_extra() {
        let original = "---\ncreated: 2026-04-19\npriority: 1\nowner: alice\n---\nhello\n";
        let (meta, body) = parse_frontmatter(original);
        let rendered = format!("{}{}", render_frontmatter(&meta), body);
        assert_eq!(rendered, original);
    }

    #[test]
    fn missing_closing_delimiter_treats_as_no_frontmatter() {
        let src = "---\ncreated: 2026-04-19\nno closing here\n";
        let (meta, body) = parse_frontmatter(src);
        assert_eq!(meta, TaskMeta::default());
        assert_eq!(body, src);
    }

    #[test]
    fn invalid_priority_falls_into_extra() {
        let src = "---\npriority: high\n---\n";
        let (meta, _) = parse_frontmatter(src);
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
        let (meta, body) = parse_frontmatter(src);
        let rendered = format!("{}{}", render_frontmatter(&meta), body);
        assert_eq!(rendered, src);
    }

    #[test]
    fn value_preserves_embedded_colons() {
        // `split_once(':')` returns the suffix after the *first* colon, so
        // ISO-8601 timestamps with `:` separators survive intact. This test
        // pins that: any change that starts splitting on every colon (e.g.
        // `split(':')`) would truncate values and break this assertion.
        let src = "---\ncreated: 2026-04-19T00:00:00Z\n---\n";
        let (meta, _) = parse_frontmatter(src);
        assert_eq!(meta.created.as_deref(), Some("2026-04-19T00:00:00Z"));
    }
}
