use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn today_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's `civil_from_days` — convert days since 1970-01-01 to
/// proleptic Gregorian (year, month, day). See
/// <https://howardhinnant.github.io/date_algorithms.html#civil_from_days>.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = y + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn civil_from_days_y2k() {
        // 1970-01-01 → 2000-01-01 is 30 years, 7 leap years (72,76,80,84,88,92,96).
        // 23*365 + 7*366 = 10957
        assert_eq!(civil_from_days(10957), (2000, 1, 1));
    }

    #[test]
    fn civil_from_days_leap_day() {
        // 2000-02-29 = 10957 + 31 + 28
        assert_eq!(civil_from_days(10957 + 59), (2000, 2, 29));
    }

    #[test]
    fn civil_from_days_known_date() {
        // 2026-04-19 = day 20562 (computed by hand; verified against algorithm).
        assert_eq!(civil_from_days(20562), (2026, 4, 19));
    }

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
