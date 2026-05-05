use crate::core::error::Result;
use crate::core::roadmap::resolve_roadmap;
use crate::core::root::find_root;

/// Delete a roadmap file. Symmetric with `cubil rm`.
///
/// Locates `.cubil/` by walking upward from cwd, checks the roadmap exists,
/// and removes the file. Silent on success.
pub fn run(slug: String) -> Result<()> {
    let root = find_root(None)?;
    let path = resolve_roadmap(&root, &slug)?;
    std::fs::remove_file(&path)?;
    Ok(())
}
