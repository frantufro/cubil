//! Shared rendering for the list commands: one table or JSON array from
//! headers and rows.

use std::fmt::Write as _;

/// Output format of a listing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Table,
    Json,
}

impl Format {
    pub fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Table }
    }
}

/// One cell of a row. `Null` renders as `-` in a table and `null` in JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Text(String),
    Number(u32),
    Null,
}

impl Cell {
    pub fn text(value: &str) -> Self {
        Self::Text(value.to_string())
    }

    pub fn opt_text(value: Option<&str>) -> Self {
        value.map_or(Self::Null, Self::text)
    }

    pub fn opt_number(value: Option<u32>) -> Self {
        value.map_or(Self::Null, Self::Number)
    }
}

/// Render rows under `headers`. Every row must have one cell per header.
///
/// Table output ends with a newline. JSON output is a single array of
/// objects keyed by header, without a trailing newline.
pub fn render(format: Format, headers: &[&str], rows: &[Vec<Cell>]) -> String {
    match format {
        Format::Table => render_table(headers, rows),
        Format::Json => render_json(headers, rows),
    }
}

/// Render and write to stdout: tables as-is, JSON followed by a newline.
pub fn print(format: Format, headers: &[&str], rows: &[Vec<Cell>]) {
    let out = render(format, headers, rows);
    match format {
        Format::Table => print!("{out}"),
        Format::Json => println!("{out}"),
    }
}

fn table_text(cell: &Cell) -> String {
    match cell {
        Cell::Text(s) => s.clone(),
        Cell::Number(n) => n.to_string(),
        Cell::Null => "-".to_string(),
    }
}

fn render_table(headers: &[&str], rows: &[Vec<Cell>]) -> String {
    // Widths are in Unicode scalar values (chars), matching how
    // `write!("{:<width$}", ...)` measures the value it pads.
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| r.iter().map(table_text).collect())
        .collect();
    for row in &cells {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    let mut out = String::new();
    let header_cells: Vec<String> = headers.iter().map(|h| (*h).to_string()).collect();
    write_row(&mut out, &header_cells, &widths);
    for row in &cells {
        write_row(&mut out, row, &widths);
    }
    out
}

fn write_row(out: &mut String, row: &[String], widths: &[usize]) {
    let last = row.len().saturating_sub(1);
    for (i, cell) in row.iter().enumerate() {
        if i > 0 {
            out.push_str("  ");
        }
        // The last column is left unpadded, so lines carry no trailing whitespace.
        if i < last {
            let _ = write!(out, "{:<width$}", cell, width = widths[i]);
        } else {
            out.push_str(cell);
        }
    }
    out.push('\n');
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).expect("serializing a string cannot fail")
}

fn render_json(headers: &[&str], rows: &[Vec<Cell>]) -> String {
    let mut out = String::from("[");
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('{');
        for (j, (header, cell)) in headers.iter().zip(row).enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&json_string(header));
            out.push(':');
            match cell {
                Cell::Text(s) => out.push_str(&json_string(s)),
                Cell::Number(n) => {
                    let _ = write!(out, "{n}");
                }
                Cell::Null => out.push_str("null"),
            }
        }
        out.push('}');
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Cell {
        Cell::text(s)
    }

    #[test]
    fn table_has_headers_and_aligns_columns() {
        let rows = vec![
            vec![t("a"), t("backlog"), Cell::Number(1)],
            vec![t("long-slug-here"), t("doing"), Cell::Null],
        ];
        let out = render(Format::Table, &["slug", "status", "priority"], &rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "slug            status   priority");
        assert_eq!(lines[1], "a               backlog  1");
        assert_eq!(lines[2], "long-slug-here  doing    -");
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn table_last_column_is_not_padded() {
        let rows = vec![vec![t("a"), t("long value")], vec![t("b"), t("x")]];
        let out = render(Format::Table, &["k", "v"], &rows);
        assert!(out.lines().all(|l| l == l.trim_end()));
    }

    #[test]
    fn table_with_no_rows_is_header_only() {
        assert_eq!(
            render(Format::Table, &["slug", "title"], &[]),
            "slug  title\n"
        );
    }

    #[test]
    fn table_pads_by_char_count_not_byte_len() {
        let rows = vec![vec![t("naïve-café"), t("x")], vec![t("abcd"), t("y")]];
        let out = render(Format::Table, &["slug", "status"], &rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[1], "naïve-café  x");
        assert_eq!(lines[2], "abcd        y");
    }

    #[test]
    fn json_emits_nulls_numbers_and_strings() {
        let rows = vec![vec![t("a"), Cell::Number(1), Cell::Null]];
        assert_eq!(
            render(Format::Json, &["slug", "priority", "created"], &rows),
            r#"[{"slug":"a","priority":1,"created":null}]"#
        );
    }

    #[test]
    fn json_separates_rows_with_commas() {
        let rows = vec![vec![t("a")], vec![t("b")]];
        assert_eq!(
            render(Format::Json, &["slug"], &rows),
            r#"[{"slug":"a"},{"slug":"b"}]"#
        );
    }

    #[test]
    fn json_empty_is_empty_array() {
        assert_eq!(render(Format::Json, &["slug"], &[]), "[]");
    }

    #[test]
    fn json_escapes_quotes_backslash_and_controls() {
        let rows = vec![vec![t("a\"b\\c\nd\te\u{01}f\r\u{08}\u{0c}\u{1f}é")]];
        assert_eq!(
            render(Format::Json, &["v"], &rows),
            r#"[{"v":"a\"b\\c\nd\te\u0001f\r\b\f\u001fé"}]"#
        );
    }

    #[test]
    fn format_from_json_flag() {
        assert_eq!(Format::from_json_flag(true), Format::Json);
        assert_eq!(Format::from_json_flag(false), Format::Table);
    }
}
