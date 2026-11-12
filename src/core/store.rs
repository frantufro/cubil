//! The Task store: finding, listing, creating, moving and removing Tasks
//! across Statuses under a `.cubil/` root.
//!
//! Every Status is a subdirectory of the root, except the
//! [`RESERVED_DIRS`]. A Task is a `<slug>.md` file inside a Status folder.
//! Slug resolution, collision, ambiguity and missing-Status rules all live
//! here, so commands only state what they want done.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};
use crate::core::root::find_root;

/// Status that new Tasks start in.
pub const BACKLOG: &str = "backlog";

/// Reserved subdirectory names under `.cubil/` that are not Status folders.
/// Files inside these directories are not surfaced by [`TaskStore::tasks`].
pub const RESERVED_DIRS: &[&str] = &["roadmaps"];

#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub status: String,
    pub slug: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TaskStore {
    root: PathBuf,
}

impl TaskStore {
    /// Open the store whose `.cubil/` directory is at `root`.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Open the store found by walking upward from the working directory.
    pub fn discover() -> Result<Self> {
        Ok(Self::new(find_root(None)?))
    }

    /// The `.cubil/` directory this store reads and writes.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether a Status folder named `status` exists.
    #[must_use]
    pub fn has_status(&self, status: &str) -> bool {
        self.root.join(status).is_dir()
    }

    /// Every Task, across all Statuses, in directory order.
    pub fn tasks(&self) -> Result<Vec<TaskEntry>> {
        let mut entries = Vec::new();
        for status_entry in std::fs::read_dir(&self.root)? {
            let status_entry = status_entry?;
            if !status_entry.file_type()?.is_dir() {
                continue;
            }
            let Ok(status) = status_entry.file_name().into_string() else {
                continue;
            };
            if RESERVED_DIRS.contains(&status.as_str()) {
                continue;
            }
            for file in std::fs::read_dir(status_entry.path())? {
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

    /// Slug-to-Task map for callers that tolerate ambiguity. When a slug
    /// exists in several Statuses, the first one scanned wins.
    pub fn lookup(&self) -> Result<HashMap<String, TaskEntry>> {
        let mut map = HashMap::new();
        for entry in self.tasks()? {
            map.entry(entry.slug.clone()).or_insert(entry);
        }
        Ok(map)
    }

    /// Locate a Task by exact slug. Errors with
    /// [`CubilError::SlugNotFound`] if there is no match and
    /// [`CubilError::SlugAmbiguous`] if the slug exists in several Statuses.
    pub fn find(&self, slug: &str) -> Result<TaskEntry> {
        let mut hits: Vec<TaskEntry> = self
            .tasks()?
            .into_iter()
            .filter(|e| e.slug == slug)
            .collect();
        match hits.len() {
            0 => Err(CubilError::SlugNotFound(slug.to_string())),
            1 => Ok(hits.pop().expect("len checked")),
            _ => Err(CubilError::SlugAmbiguous {
                slug: slug.to_string(),
                statuses: hits.into_iter().map(|e| e.status).collect(),
            }),
        }
    }

    /// Create Task `slug` in [`BACKLOG`] with file contents `text`. Errors
    /// with [`CubilError::SlugCollision`] if the slug exists in any Status.
    pub fn create(&self, slug: &str, text: &str) -> Result<PathBuf> {
        if let Some(existing) = self.tasks()?.into_iter().find(|e| e.slug == slug) {
            return Err(CubilError::SlugCollision {
                slug: slug.to_string(),
                status: existing.status,
            });
        }
        let path = self.root.join(BACKLOG).join(format!("{slug}.md"));
        std::fs::write(&path, text)?;
        Ok(path)
    }

    /// Move Task `slug` into Status `status`.
    ///
    /// Moving a Task into the Status it is already in does nothing. Errors
    /// with [`CubilError::StatusMissing`] if the destination folder does not
    /// exist (users create Status folders themselves) and
    /// [`CubilError::SlugCollision`] if the destination already holds a file
    /// with that slug.
    pub fn move_to(&self, slug: &str, status: &str) -> Result<()> {
        let current = self.find(slug)?;
        self.move_entry(&current, status)
    }

    /// Move Task `slug` from Status `expected` into `target`. Errors with
    /// [`CubilError::StatusMismatch`] if the Task is in another Status; the
    /// remaining errors are those of [`TaskStore::move_to`].
    pub fn transition(&self, slug: &str, expected: &str, target: &str) -> Result<()> {
        let current = self.find(slug)?;
        if current.status != expected {
            return Err(CubilError::StatusMismatch {
                slug: slug.to_string(),
                expected: expected.to_string(),
                actual: current.status,
            });
        }
        self.move_entry(&current, target)
    }

    fn move_entry(&self, current: &TaskEntry, status: &str) -> Result<()> {
        if current.status == status {
            return Ok(());
        }
        let dest_dir = self.root.join(status);
        if !dest_dir.is_dir() {
            return Err(CubilError::StatusMissing(status.to_string()));
        }
        let file_name = current
            .path
            .file_name()
            .expect("tasks() returns file paths");
        let dest_path = dest_dir.join(file_name);
        // NOTE: local single-user tool: the TOCTOU window between exists()
        // and rename() is acceptable.
        if dest_path.exists() {
            return Err(CubilError::SlugCollision {
                slug: current.slug.clone(),
                status: status.to_string(),
            });
        }
        std::fs::rename(&current.path, &dest_path)?;
        Ok(())
    }

    /// Delete Task `slug`.
    pub fn remove(&self, slug: &str) -> Result<()> {
        let entry = self.find(slug)?;
        std::fs::remove_file(&entry.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, tempdir};

    fn store_with(statuses: &[&str]) -> (TempDir, TaskStore) {
        let dir = tempdir().unwrap();
        for s in statuses {
            std::fs::create_dir(dir.path().join(s)).unwrap();
        }
        let store = TaskStore::new(dir.path().to_path_buf());
        (dir, store)
    }

    fn put(dir: &TempDir, status: &str, slug: &str) {
        std::fs::write(dir.path().join(status).join(format!("{slug}.md")), "x").unwrap();
    }

    #[test]
    fn find_returns_single_match() {
        let (dir, store) = store_with(&["backlog", "doing"]);
        put(&dir, "backlog", "foo");
        let entry = store.find("foo").unwrap();
        assert_eq!(entry.status, "backlog");
        assert_eq!(entry.path, dir.path().join("backlog").join("foo.md"));
    }

    #[test]
    fn find_errors_when_missing() {
        let (_dir, store) = store_with(&["backlog"]);
        let err = store.find("nope").unwrap_err();
        assert!(matches!(err, CubilError::SlugNotFound(s) if s == "nope"));
    }

    #[test]
    fn find_errors_when_ambiguous() {
        let (dir, store) = store_with(&["backlog", "done"]);
        put(&dir, "backlog", "dup");
        put(&dir, "done", "dup");
        match store.find("dup").unwrap_err() {
            CubilError::SlugAmbiguous { slug, mut statuses } => {
                assert_eq!(slug, "dup");
                statuses.sort();
                assert_eq!(statuses, vec!["backlog".to_string(), "done".to_string()]);
            }
            other => panic!("expected SlugAmbiguous, got {other:?}"),
        }
    }

    #[test]
    fn tasks_ignores_non_md_files_and_reserved_dirs() {
        let (dir, store) = store_with(&["backlog", "roadmaps"]);
        put(&dir, "backlog", "a");
        put(&dir, "roadmaps", "r");
        std::fs::write(dir.path().join("backlog").join("notes.txt"), "x").unwrap();
        let entries = store.tasks().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].slug, "a");
    }

    #[test]
    fn lookup_maps_slug_to_entry() {
        let (dir, store) = store_with(&["backlog", "doing"]);
        put(&dir, "backlog", "a");
        put(&dir, "doing", "b");
        let map = store.lookup().unwrap();
        assert_eq!(map["a"].status, "backlog");
        assert_eq!(map["b"].status, "doing");
        assert!(!map.contains_key("c"));
    }

    #[test]
    fn create_writes_into_backlog() {
        let (dir, store) = store_with(&["backlog"]);
        let path = store.create("new-one", "hello").unwrap();
        assert_eq!(path, dir.path().join("backlog").join("new-one.md"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "hello");
    }

    #[test]
    fn create_rejects_slug_in_any_status() {
        let (dir, store) = store_with(&["backlog", "done"]);
        put(&dir, "done", "taken");
        let err = store.create("taken", "x").unwrap_err();
        assert!(
            matches!(err, CubilError::SlugCollision { slug, status } if slug == "taken" && status == "done")
        );
    }

    #[test]
    fn move_to_renames_file() {
        let (dir, store) = store_with(&["backlog", "doing"]);
        put(&dir, "backlog", "t");
        store.move_to("t", "doing").unwrap();
        assert!(!dir.path().join("backlog").join("t.md").exists());
        assert!(dir.path().join("doing").join("t.md").exists());
    }

    #[test]
    fn move_to_same_status_is_noop() {
        let (dir, store) = store_with(&["backlog"]);
        put(&dir, "backlog", "t");
        store.move_to("t", "backlog").unwrap();
        assert!(dir.path().join("backlog").join("t.md").exists());
    }

    #[test]
    fn move_to_errors_when_status_missing() {
        let (dir, store) = store_with(&["backlog"]);
        put(&dir, "backlog", "t");
        let err = store.move_to("t", "doing").unwrap_err();
        assert!(matches!(err, CubilError::StatusMissing(s) if s == "doing"));
    }

    #[test]
    fn move_to_errors_when_destination_holds_slug() {
        let (dir, store) = store_with(&["backlog", "doing"]);
        put(&dir, "backlog", "t");
        // A directory named like the Task is not a Task, so find() ignores
        // it, but it still blocks the rename.
        std::fs::create_dir(dir.path().join("doing").join("t.md")).unwrap();
        let err = store.move_to("t", "doing").unwrap_err();
        assert!(matches!(err, CubilError::SlugCollision { status, .. } if status == "doing"));
    }

    #[test]
    fn move_to_errors_on_unknown_slug() {
        let (_dir, store) = store_with(&["backlog", "doing"]);
        let err = store.move_to("t", "doing").unwrap_err();
        assert!(matches!(err, CubilError::SlugNotFound(_)));
    }

    #[test]
    fn transition_moves_when_status_matches() {
        let (dir, store) = store_with(&["backlog", "doing"]);
        put(&dir, "backlog", "t");
        store.transition("t", "backlog", "doing").unwrap();
        assert!(dir.path().join("doing").join("t.md").exists());
    }

    #[test]
    fn transition_errors_on_status_mismatch() {
        let (dir, store) = store_with(&["backlog", "doing"]);
        put(&dir, "doing", "t");
        let err = store.transition("t", "backlog", "doing").unwrap_err();
        assert!(matches!(
            err,
            CubilError::StatusMismatch { expected, actual, .. }
                if expected == "backlog" && actual == "doing"
        ));
    }

    #[test]
    fn remove_deletes_file() {
        let (dir, store) = store_with(&["backlog"]);
        put(&dir, "backlog", "t");
        store.remove("t").unwrap();
        assert!(!dir.path().join("backlog").join("t.md").exists());
        assert!(matches!(
            store.remove("t").unwrap_err(),
            CubilError::SlugNotFound(_)
        ));
    }

    #[test]
    fn has_status_checks_directory() {
        let (_dir, store) = store_with(&["backlog"]);
        assert!(store.has_status("backlog"));
        assert!(!store.has_status("doing"));
    }
}
