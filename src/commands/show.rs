use std::fs::File;
use std::io;

use crate::core::error::Result;
use crate::core::{root, slug};

/// Print a task's full markdown contents to stdout, byte-identical.
///
/// Walks upward from cwd to find `.cubil/`, resolves the slug across every
/// status folder, and streams the file bytes verbatim — no re-encoding, no
/// trailing newline injection.
pub fn run(slug: String) -> Result<()> {
    let root = root::find_root(None)?;
    let (_status, path) = slug::resolve_slug(&root, &slug)?;
    let mut file = File::open(&path)?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    io::copy(&mut file, &mut handle)?;
    Ok(())
}
