use std::io::Read;
use std::path::{Path, PathBuf};

use crate::core::error::{CubilError, Result};
use crate::core::roadmap::{ensure_roadmaps_dir, roadmap_path};
use crate::core::root::find_root;
use crate::core::slug::slugify;

/// Create a new roadmap file under `.cubil/roadmaps/`.
///
/// Slug is derived from the title via [`slugify`]. Body is optional and may
/// be supplied via `-m`, `-F <path>`, or `-F -` (stdin) — mirrors `cubil
/// new`. The file is plain markdown with a top-level `# <title>` heading
/// followed by the optional narrative; no frontmatter is written.
pub fn run(title: String, message: Option<String>, file: Option<PathBuf>) -> Result<()> {
    let root = find_root(None)?;
    let slug = slugify(&title)?;

    ensure_roadmaps_dir(&root)?;
    let path = roadmap_path(&root, &slug);
    if path.exists() {
        return Err(CubilError::RoadmapExists(slug));
    }

    let body = read_body(message, file.as_deref())?;
    let contents = assemble(&title, &body);
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

fn assemble(title: &str, body: &str) -> String {
    let mut out = String::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_empty_body() {
        assert_eq!(assemble("Migrate", ""), "# Migrate\n");
    }

    #[test]
    fn assemble_with_body_adds_blank_line_and_trailing_newline() {
        assert_eq!(
            assemble("Migrate", "Narrative."),
            "# Migrate\n\nNarrative.\n"
        );
    }

    #[test]
    fn assemble_preserves_body_trailing_newlines() {
        assert_eq!(
            assemble("Migrate", "line\n\n"),
            "# Migrate\n\nline\n\n"
        );
    }
}
