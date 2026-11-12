use std::path::Path;

use crate::core::error::{CubilError, Result};
use crate::core::roadmap::{ensure_roadmaps_dir, roadmap_path};
use crate::core::root::find_root;
use crate::core::slug::slugify;
use crate::core::task_doc::{read_body, titled_body};

/// Create a new roadmap file under `.cubil/roadmaps/`.
///
/// Slug is derived from the title via [`slugify`]. Body is optional and may
/// be supplied via `-m`, `-F <path>`, or `-F -` (stdin) — mirrors `cubil
/// new`. The file is plain markdown with a top-level `# <title>` heading
/// followed by the optional narrative; no frontmatter is written.
pub fn run(title: &str, message: Option<String>, file: Option<&Path>) -> Result<()> {
    let root = find_root(None)?;
    let slug = slugify(title)?;

    ensure_roadmaps_dir(&root)?;
    let path = roadmap_path(&root, &slug);
    if path.exists() {
        return Err(CubilError::RoadmapExists(slug));
    }

    let body = read_body(message, file)?;
    let contents = titled_body(title, &body);
    std::fs::write(&path, contents)?;

    println!("{slug}");
    Ok(())
}
