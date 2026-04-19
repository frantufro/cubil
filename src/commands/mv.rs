use crate::core::error::{CubilError, Result};
use crate::core::{root, slug as slug_mod};

/// Move task `<slug>` into `<root>/<status>/`.
///
/// - Errors with [`CubilError::SlugNotFound`] / [`CubilError::SlugAmbiguous`]
///   via [`slug_mod::resolve_slug`].
/// - Errors with [`CubilError::StatusMissing`] if the destination folder does
///   not exist (users create status folders themselves).
/// - Errors with [`CubilError::SlugCollision`] if a file with the same slug
///   already exists in the destination.
/// - If the resolved status equals the target, returns `Ok(())` without
///   touching the file or printing anything.
/// - Otherwise, atomically renames the file via [`std::fs::rename`].
pub fn run(slug: String, status: String) -> Result<()> {
    let root = root::find_root(None)?;
    let (current_status, current_path) = slug_mod::resolve_slug(&root, &slug)?;

    if current_status == status {
        return Ok(());
    }

    let dest_dir = root.join(&status);
    if !dest_dir.is_dir() {
        return Err(CubilError::StatusMissing(status));
    }

    let dest_path = dest_dir.join(format!("{slug}.md"));
    if dest_path.exists() {
        return Err(CubilError::SlugCollision { slug, status });
    }

    std::fs::rename(&current_path, &dest_path)?;
    Ok(())
}
