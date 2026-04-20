use crate::core::error::Result;
use crate::core::{root, slug};

/// Delete a task file.
///
/// Locates `.cubil/` by walking upward from cwd, resolves the slug across all
/// status folders, and removes the file. Silent on success.
pub fn run(slug: String) -> Result<()> {
    let root = root::find_root(None)?;
    let (_status, path) = slug::resolve_slug(&root, &slug)?;
    std::fs::remove_file(&path)?;
    Ok(())
}
