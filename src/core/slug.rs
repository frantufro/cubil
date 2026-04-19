use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};

#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub status: String,
    pub slug: String,
    pub path: PathBuf,
}

/// Convert a human title into a kebab-case, lowercase, ASCII-only slug.
///
/// Non-alphanumeric ASCII characters (including any non-ASCII) are treated as
/// separators, collapsed into single `-`. Leading and trailing `-` are
/// trimmed. Returns [`CubilError::InvalidSlug`] if the result is empty.
pub fn slugify(title: &str) -> Result<String> {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = true;
    for c in title.chars() {
        let lower = c.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        return Err(CubilError::InvalidSlug);
    }
    Ok(out)
}

/// List every `(status, slug, absolute_path)` triple of `*.md` files under
/// the given `.cubil/` root. Each immediate subdirectory of `root` is treated
/// as a status folder.
pub fn scan_all(root: &Path) -> Result<Vec<TaskEntry>> {
    let mut entries = Vec::new();
    for status_entry in std::fs::read_dir(root)? {
        let status_entry = status_entry?;
        if !status_entry.file_type()?.is_dir() {
            continue;
        }
        let status = match status_entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        let status_path = status_entry.path();
        for file in std::fs::read_dir(&status_path)? {
            let file = file?;
            if !file.file_type()?.is_file() {
                continue;
            }
            let fname = file.file_name();
            let Some(fname_str) = fname.to_str() else {
                continue;
            };
            if let Some(slug) = fname_str.strip_suffix(".md") {
                entries.push(TaskEntry {
                    status: status.clone(),
                    slug: slug.to_string(),
                    path: file.path(),
                });
            }
        }
    }
    Ok(entries)
}

/// Locate a task file by exact slug across every status folder under
/// `.cubil/`. Returns `(status_name, absolute_path_to_md_file)`. Errors with
/// [`CubilError::SlugNotFound`] if no match, or
/// [`CubilError::SlugAmbiguous`] if the same slug exists in multiple status
/// folders.
pub fn resolve_slug(root: &Path, slug: &str) -> Result<(String, PathBuf)> {
    let mut hits: Vec<(String, PathBuf)> = Vec::new();
    for entry in scan_all(root)? {
        if entry.slug == slug {
            hits.push((entry.status, entry.path));
        }
    }
    match hits.len() {
        0 => Err(CubilError::SlugNotFound(slug.to_string())),
        1 => Ok(hits.pop().expect("len checked")),
        _ => {
            let statuses = hits.into_iter().map(|(s, _)| s).collect();
            Err(CubilError::SlugAmbiguous {
                slug: slug.to_string(),
                statuses,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World").unwrap(), "hello-world");
    }

    #[test]
    fn slugify_collapses_separators_and_trims() {
        assert_eq!(slugify("  Foo --- Bar / Baz!!!").unwrap(), "foo-bar-baz");
    }

    #[test]
    fn slugify_strips_non_ascii() {
        assert_eq!(slugify("Café résumé").unwrap(), "caf-r-sum");
    }

    #[test]
    fn slugify_preserves_digits() {
        assert_eq!(slugify("release v1.2.3").unwrap(), "release-v1-2-3");
    }

    #[test]
    fn slugify_rejects_empty_result() {
        assert!(matches!(
            slugify("!!!").unwrap_err(),
            CubilError::InvalidSlug
        ));
        assert!(matches!(slugify("").unwrap_err(), CubilError::InvalidSlug));
    }

    #[test]
    fn resolve_slug_finds_single_match() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("backlog")).unwrap();
        std::fs::create_dir(root.join("doing")).unwrap();
        std::fs::write(root.join("backlog").join("foo.md"), "x").unwrap();
        let (status, path) = resolve_slug(root, "foo").unwrap();
        assert_eq!(status, "backlog");
        assert_eq!(path, root.join("backlog").join("foo.md"));
    }

    #[test]
    fn resolve_slug_errors_when_missing() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("backlog")).unwrap();
        let err = resolve_slug(root, "nope").unwrap_err();
        assert!(matches!(err, CubilError::SlugNotFound(s) if s == "nope"));
    }

    #[test]
    fn resolve_slug_errors_when_ambiguous() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("backlog")).unwrap();
        std::fs::create_dir(root.join("done")).unwrap();
        std::fs::write(root.join("backlog").join("dup.md"), "x").unwrap();
        std::fs::write(root.join("done").join("dup.md"), "y").unwrap();
        let err = resolve_slug(root, "dup").unwrap_err();
        match err {
            CubilError::SlugAmbiguous { slug, mut statuses } => {
                assert_eq!(slug, "dup");
                statuses.sort();
                assert_eq!(statuses, vec!["backlog".to_string(), "done".to_string()]);
            }
            other => panic!("expected SlugAmbiguous, got {other:?}"),
        }
    }

    #[test]
    fn scan_all_ignores_non_md_files() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("backlog")).unwrap();
        std::fs::write(root.join("backlog").join("a.md"), "x").unwrap();
        std::fs::write(root.join("backlog").join("notes.txt"), "x").unwrap();
        let entries = scan_all(root).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].slug, "a");
    }
}
