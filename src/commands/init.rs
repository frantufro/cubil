use crate::core::error::{CubilError, Result};

const STATUS_DIRS: &[&str] = &["backlog", "doing", "done"];

/// Initialize `.cubil/` at the current working directory.
///
/// Creates `.cubil/` plus the three default status folders (`backlog`,
/// `doing`, `done`). Idempotent: running in an already-initialized directory
/// is a no-op. Errors if `.cubil` already exists as a file.
///
/// Unlike the other commands, `init` does not walk upward — it always acts
/// on `cwd`.
pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let root = cwd.join(".cubil");

    if root.exists() && !root.is_dir() {
        return Err(CubilError::RootIsFile(root));
    }

    std::fs::create_dir_all(&root)?;
    for status in STATUS_DIRS {
        std::fs::create_dir_all(root.join(status))?;
    }

    let abs = root.canonicalize()?;
    println!("{}", abs.display());
    Ok(())
}
