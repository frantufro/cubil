use std::fs::File;
use std::io;

use crate::core::error::Result;
use crate::core::store::TaskStore;

/// Print a task's full markdown contents to stdout, byte-identical.
///
/// Resolves the slug across every status folder and streams the file bytes
/// verbatim: no re-encoding, no trailing newline injection.
pub fn run(slug: &str) -> Result<()> {
    let entry = TaskStore::discover()?.find(slug)?;
    let mut file = File::open(&entry.path)?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    io::copy(&mut file, &mut handle)?;
    Ok(())
}
