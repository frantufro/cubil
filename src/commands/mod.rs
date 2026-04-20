pub mod edit;
pub mod finish;
pub mod init;
pub mod list;
pub mod mv;
pub mod new;
pub mod rm;
pub mod show;
pub mod start;

use crate::core::error::{CubilError, Result};
use crate::core::{root, slug as slug_mod};

/// Move a task from `expected` status to `target` status.
///
/// Errors with [`CubilError::StatusMismatch`] if the task exists but is in a
/// different status folder. `SlugNotFound` / `SlugAmbiguous` propagate from
/// [`slug_mod::resolve_slug`]. Errors with [`CubilError::StatusMissing`] if
/// the destination folder does not exist, and [`CubilError::SlugCollision`]
/// if a file with the same slug already exists at the destination.
pub(crate) fn transition(slug: String, expected: &str, target: &str) -> Result<()> {
    let root = root::find_root(None)?;
    let (current_status, current_path) = slug_mod::resolve_slug(&root, &slug)?;

    if current_status != expected {
        return Err(CubilError::StatusMismatch {
            slug,
            expected: expected.to_string(),
            actual: current_status,
        });
    }

    let dest_dir = root.join(target);
    if !dest_dir.is_dir() {
        return Err(CubilError::StatusMissing(target.to_string()));
    }

    let file_name = current_path
        .file_name()
        .expect("resolve_slug returns a file path");
    let dest_path = dest_dir.join(file_name);
    // NOTE: local single-user tool — TOCTOU window between exists() check and
    // rename() is acceptable.
    if dest_path.exists() {
        return Err(CubilError::SlugCollision {
            slug,
            status: target.to_string(),
        });
    }

    std::fs::rename(&current_path, &dest_path)?;
    Ok(())
}
