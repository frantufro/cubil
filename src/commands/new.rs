use std::io::Read;
use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};
use crate::core::frontmatter::{TaskMeta, render_frontmatter};
use crate::core::root::find_root;
use crate::core::slug::{scan_all, slugify};

const BACKLOG: &str = "backlog";

/// Create a new task file under `.cubil/backlog/`.
///
/// Prints the resulting slug to stdout on success. `message` and `file` are
/// mutually exclusive (enforced by clap); passing neither produces an empty
/// body. When `file` is `Some("-")`, the body is read from stdin.
pub fn run(title: String, message: Option<String>, file: Option<PathBuf>) -> Result<()> {
    let root = find_root(None)?;
    let slug = slugify(&title)?;

    for entry in scan_all(&root)? {
        if entry.slug == slug {
            return Err(CubilError::SlugCollision {
                slug,
                status: entry.status,
            });
        }
    }

    let body = read_body(message, file.as_deref())?;

    let meta = TaskMeta {
        created: Some(today_iso()),
        priority: None,
        extra: Vec::new(),
    };

    let contents = assemble(&meta, &title, &body);

    let path = root.join(BACKLOG).join(format!("{slug}.md"));
    std::fs::write(&path, contents)?;

    println!("{slug}");
    Ok(())
}

fn read_body(message: Option<String>, file: Option<&Path>) -> Result<String> {
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

fn assemble(meta: &TaskMeta, title: &str, body: &str) -> String {
    let mut out = render_frontmatter(meta);
    out.push('\n');
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

/// Today's date in ISO 8601 (`YYYY-MM-DD`), in the **user's local timezone**.
///
/// A personal task tool should agree with the user's wall clock — writing
/// "yesterday" in a task created at 1am local time (UTC+0 midnight) would be
/// surprising. Falls back to UTC only if the local offset can't be determined
/// (e.g. on platforms where `localtime_r` is unsafe to call from a
/// multithreaded process — not a concern for this single-threaded CLI, but
/// the fallback keeps us honest).
fn today_iso() -> String {
    let dt = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let d = dt.date();
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_iso_is_well_formed() {
        let s = today_iso();
        assert_eq!(s.len(), 10);
        assert_eq!(s.as_bytes()[4], b'-');
        assert_eq!(s.as_bytes()[7], b'-');
        assert!(s[0..4].chars().all(|c| c.is_ascii_digit()));
        assert!(s[5..7].chars().all(|c| c.is_ascii_digit()));
        assert!(s[8..10].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn assemble_empty_body_has_no_trailing_blank_line() {
        let meta = TaskMeta {
            created: Some("2026-04-19".into()),
            priority: None,
            extra: Vec::new(),
        };
        let out = assemble(&meta, "Hello World", "");
        assert_eq!(out, "---\ncreated: 2026-04-19\n---\n\n# Hello World\n");
    }

    #[test]
    fn assemble_with_body_adds_trailing_newline_when_missing() {
        let meta = TaskMeta {
            created: Some("2026-04-19".into()),
            priority: None,
            extra: Vec::new(),
        };
        let out = assemble(&meta, "T", "line");
        assert_eq!(out, "---\ncreated: 2026-04-19\n---\n\n# T\n\nline\n");
    }

    #[test]
    fn assemble_preserves_body_trailing_newlines() {
        let meta = TaskMeta {
            created: Some("2026-04-19".into()),
            priority: None,
            extra: Vec::new(),
        };
        let out = assemble(&meta, "T", "line\n\n");
        assert_eq!(out, "---\ncreated: 2026-04-19\n---\n\n# T\n\nline\n\n");
    }
}
