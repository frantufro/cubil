use crate::core::error::{CubilError, Result};

const STATUS_DIRS: &[&str] = &["backlog", "doing", "done"];

/// Initialize `.cubil/` at the current working directory.
///
/// Creates `.cubil/` plus the three default status folders (`backlog`,
/// `doing`, `done`), each containing a `.gitkeep` so the empty folders
/// are tracked by git. Idempotent: running in an already-initialized
/// directory is a no-op. Errors if `.cubil` already exists as a file.
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
        let dir = root.join(status);
        std::fs::create_dir_all(&dir)?;
        let gitkeep = dir.join(".gitkeep");
        if !gitkeep.exists() {
            std::fs::write(&gitkeep, b"")?;
        }
    }

    let abs = root.canonicalize()?;
    println!("{}", abs.display());
    Ok(())
}
