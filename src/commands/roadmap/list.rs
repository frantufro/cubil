use std::fmt::Write as _;
use std::path::Path;

use crate::core::error::Result;
use crate::core::roadmap::{extract_roadmap_title, list_roadmap_slugs, roadmap_path};
use crate::core::root::find_root;

struct Row {
    slug: String,
    title: Option<String>,
}

pub fn run(json: bool) -> Result<()> {
    let root = find_root(None)?;
    let rows = collect_rows(&root)?;
    if json {
        println!("{}", render_json(&rows));
    } else {
        print!("{}", render_table(&rows));
    }
    Ok(())
}

fn collect_rows(root: &Path) -> Result<Vec<Row>> {
    let mut slugs = list_roadmap_slugs(root)?;
    slugs.sort();
    let mut rows = Vec::with_capacity(slugs.len());
    for slug in slugs {
        let path = roadmap_path(root, &slug);
        let title = match std::fs::read_to_string(&path) {
            Ok(src) => extract_roadmap_title(&src),
            Err(_) => None,
        };
        rows.push(Row { slug, title });
    }
    Ok(rows)
}

fn render_table(rows: &[Row]) -> String {
    let headers = ["slug", "title"];
    let mut widths = [headers[0].chars().count(), headers[1].chars().count()];
    let cells: Vec<[String; 2]> = rows
        .iter()
        .map(|r| {
            [
                r.slug.clone(),
                r.title.clone().unwrap_or_else(|| "-".into()),
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

fn write_row(out: &mut String, row: &[String; 2], widths: &[usize; 2]) {
    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            out.push_str("  ");
        }
        if i < 1 {
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
        out.push_str(",\"title\":");
        match &r.title {
            Some(t) => push_json_string(&mut out, t),
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

    fn row(slug: &str, title: Option<&str>) -> Row {
        Row {
            slug: slug.into(),
            title: title.map(String::from),
        }
    }

    #[test]
    fn render_table_has_headers() {
        let rows = vec![row("foo", Some("Foo Roadmap")), row("bar", None)];
        let out = render_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("slug"));
        assert!(lines[0].contains("title"));
    }

    #[test]
    fn render_table_pads_slug_column() {
        let rows = vec![row("a", Some("X")), row("long-slug", Some("Y"))];
        let out = render_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        // "long-slug" is 9 chars, plus two spaces.
        assert!(lines[1].starts_with("a        "));
        assert!(lines[2].starts_with("long-slug"));
    }

    #[test]
    fn render_json_emits_null_for_missing_title() {
        let rows = vec![row("foo", None)];
        assert_eq!(render_json(&rows), r#"[{"slug":"foo","title":null}]"#);
    }

    #[test]
    fn render_json_empty() {
        assert_eq!(render_json(&[]), "[]");
    }

    #[test]
    fn render_json_escapes_title() {
        let rows = vec![row("foo", Some("a\"b"))];
        assert_eq!(render_json(&rows), r#"[{"slug":"foo","title":"a\"b"}]"#);
    }
}
