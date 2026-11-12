use std::path::Path;

use crate::core::error::{CubilError, Result};
use crate::core::slug::slugify;
use crate::core::store::TaskStore;
use crate::core::task_doc::{TaskDoc, read_body};

/// Create a new task file under `.cubil/backlog/`.
///
/// Prints the resulting slug to stdout on success. `message` and `file` are
/// mutually exclusive (enforced by clap); passing neither produces an empty
/// body. When `file` is `Some("-")`, the body is read from stdin.
pub fn run(title: &str, message: Option<String>, file: Option<&Path>) -> Result<()> {
    let store = TaskStore::discover()?;
    let slug = slugify(title)?;

    // Fail on a collision before reading the body, which may block on stdin.
    if let Some(existing) = store.lookup()?.get(&slug) {
        return Err(CubilError::SlugCollision {
            slug,
            status: existing.status.clone(),
        });
    }

    let body = read_body(message, file)?;

    let contents = TaskDoc::new(title, &body, &today_iso()).render();

    store.create(&slug, &contents)?;

    println!("{slug}");
    Ok(())
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
}
