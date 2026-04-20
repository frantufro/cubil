use std::cmp::Ordering;
use std::fmt::Write as _;
use std::path::Path;

use crate::core::error::{CubilError, Result};
use crate::core::frontmatter::parse_frontmatter;
use crate::core::root::find_root;
use crate::core::slug::{TaskEntry, scan_all};

const DONE: &str = "done";

struct Row {
    slug: String,
    status: String,
    priority: Option<u32>,
    created: Option<String>,
}

pub fn run(all: bool, status: Option<String>, json: bool) -> Result<()> {
    let root = find_root(None)?;
    let rows = collect_rows(&root, all, status.as_deref())?;
    if json {
        println!("{}", render_json(&rows));
    } else {
        print!("{}", render_table(&rows));
    }
    Ok(())
}

fn collect_rows(root: &Path, all: bool, status: Option<&str>) -> Result<Vec<Row>> {
    if let Some(name) = status
        && !root.join(name).is_dir()
    {
        return Err(CubilError::StatusMissing(name.to_string()));
    }
    let mut rows: Vec<Row> = Vec::new();
    for TaskEntry {
        status: entry_status,
        slug,
        path,
    } in scan_all(root)?
    {
        let keep = match status {
            Some(name) => entry_status == name,
            None if all => true,
            None => entry_status != DONE,
        };
        if !keep {
            continue;
        }
        let src = std::fs::read_to_string(&path)?;
        let (meta, _body) = parse_frontmatter(&src);
        rows.push(Row {
            slug,
            status: entry_status,
            priority: meta.priority,
            created: meta.created,
        });
    }
    rows.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| cmp_priority(a.priority, b.priority))
            .then_with(|| a.slug.cmp(&b.slug))
    });
    Ok(rows)
}

fn cmp_priority(a: Option<u32>, b: Option<u32>) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn render_table(rows: &[Row]) -> String {
    let headers = ["slug", "status", "priority", "created"];
    // Widths are in Unicode scalar values (chars), matching how
    // `write!("{:<width$}", ...)` measures the value it pads.
    let mut widths = [
        headers[0].chars().count(),
        headers[1].chars().count(),
        headers[2].chars().count(),
        headers[3].chars().count(),
    ];
    let cells: Vec<[String; 4]> = rows
        .iter()
        .map(|r| {
            [
                r.slug.clone(),
                r.status.clone(),
                r.priority
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "-".into()),
                r.created.clone().unwrap_or_else(|| "-".into()),
            ]
        })
        .collect();
    for row in &cells {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    let mut out = String::new();
    write_row(&mut out, &headers.map(String::from), &widths);
    for row in &cells {
        write_row(&mut out, row, &widths);
    }
    out
}

fn write_row(out: &mut String, row: &[String; 4], widths: &[usize; 4]) {
    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            out.push_str("  ");
        }
        // Only pad columns that precede the last non-empty column to avoid
        // trailing whitespace on every line.
        if i < 3 {
            let _ = write!(out, "{:<width$}", cell, width = widths[i]);
        } else {
            out.push_str(cell);
        }
    }
    out.push('\n');
}

fn render_json(rows: &[Row]) -> String {
    let mut out = String::from("[");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('{');
        out.push_str("\"slug\":");
        push_json_string(&mut out, &r.slug);
        out.push_str(",\"status\":");
        push_json_string(&mut out, &r.status);
        out.push_str(",\"priority\":");
        match r.priority {
            Some(p) => {
                let _ = write!(out, "{p}");
            }
            None => out.push_str("null"),
        }
        out.push_str(",\"created\":");
        match &r.created {
            Some(c) => push_json_string(&mut out, c),
            None => out.push_str("null"),
        }
        out.push('}');
    }
    out.push(']');
    out
}

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(slug: &str, status: &str, priority: Option<u32>, created: Option<&str>) -> Row {
        Row {
            slug: slug.into(),
            status: status.into(),
            priority,
            created: created.map(String::from),
        }
    }

    #[test]
    fn render_table_has_headers_and_aligns_columns() {
        let rows = vec![
            row("a", "backlog", Some(1), Some("2026-04-19")),
            row("long-slug-here", "doing", None, None),
        ];
        let out = render_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("slug"));
        assert!(lines[0].contains("status"));
        assert!(lines[0].contains("priority"));
        assert!(lines[0].contains("created"));
        // All rows share the slug column width (14 = "long-slug-here").
        assert!(lines[1].starts_with("a             "));
        assert!(lines[2].starts_with("long-slug-here"));
        // Missing values render as "-".
        assert!(lines[2].contains("-"));
    }

    #[test]
    fn render_json_emits_nulls_for_missing() {
        let rows = vec![row("tidy-readme", "backlog", None, None)];
        let out = render_json(&rows);
        assert_eq!(
            out,
            r#"[{"slug":"tidy-readme","status":"backlog","priority":null,"created":null}]"#
        );
    }

    #[test]
    fn render_json_emits_number_for_priority() {
        let rows = vec![row("implement-new", "doing", Some(1), Some("2026-04-19"))];
        let out = render_json(&rows);
        assert_eq!(
            out,
            r#"[{"slug":"implement-new","status":"doing","priority":1,"created":"2026-04-19"}]"#
        );
    }

    #[test]
    fn render_json_empty_is_empty_array() {
        assert_eq!(render_json(&[]), "[]");
    }

    #[test]
    fn push_json_string_escapes_quotes_backslash_and_controls() {
        let mut out = String::new();
        push_json_string(&mut out, "a\"b\\c\nd\te\u{01}f");
        assert_eq!(out, r#""a\"b\\c\nd\te\u0001f""#);
    }

    #[test]
    fn cmp_priority_puts_none_last() {
        assert_eq!(cmp_priority(Some(1), Some(2)), Ordering::Less);
        assert_eq!(cmp_priority(Some(1), None), Ordering::Less);
        assert_eq!(cmp_priority(None, Some(1)), Ordering::Greater);
        assert_eq!(cmp_priority(None, None), Ordering::Equal);
    }

    #[test]
    fn render_table_pads_by_char_count_not_byte_len() {
        // "naïve-café" is 10 chars / 12 bytes. Pairing it with a pure-ASCII
        // slug of 4 chars exercises byte/char divergence in the width calc.
        // If widths were computed in bytes, the ASCII row would over-pad and
        // the columns would drift apart when rendered.
        let rows = vec![
            row("naïve-café", "backlog", Some(1), Some("2026-04-19")),
            row("abcd", "doing", None, None),
        ];
        let out = render_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        // Max slug width across header + data = 10 chars ("naïve-café"),
        // then 2 spaces, then status.
        let char_offset = |line: &str, needle: &str| -> usize {
            line.char_indices()
                .position(|(i, _)| line[i..].starts_with(needle))
                .expect("needle present")
        };
        assert_eq!(char_offset(lines[1], "backlog"), 12);
        assert_eq!(char_offset(lines[2], "doing"), 12);
    }
}
