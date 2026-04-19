use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};

/// Walk upward from `start` (or cwd if None) looking for a `.cubil/` directory.
/// Returns the absolute path to the `.cubil/` directory itself. Errors with
/// [`CubilError::RootNotFound`] if no `.cubil/` is found before the filesystem
/// root.
pub fn find_root(start: Option<&Path>) -> Result<PathBuf> {
    let mut current: PathBuf = match start {
        Some(p) if p.is_absolute() => p.to_path_buf(),
        Some(p) => std::env::current_dir()?.join(p),
        None => std::env::current_dir()?,
    };

    loop {
        let candidate = current.join(".cubil");
        if candidate.is_dir() {
            return candidate.canonicalize().map_err(CubilError::from);
        }
        if !current.pop() {
            return Err(CubilError::RootNotFound);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn finds_root_in_current_dir() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".cubil")).unwrap();
        let found = find_root(Some(dir.path())).unwrap();
        assert_eq!(found, dir.path().join(".cubil").canonicalize().unwrap());
    }

    #[test]
    fn walks_upward_through_nested_dirs() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".cubil")).unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        let found = find_root(Some(&nested)).unwrap();
        assert_eq!(found, dir.path().join(".cubil").canonicalize().unwrap());
    }

    #[test]
    fn returns_root_not_found_when_absent() {
        let dir = tempdir().unwrap();
        let err = find_root(Some(dir.path())).unwrap_err();
        assert!(matches!(err, CubilError::RootNotFound));
    }
}
