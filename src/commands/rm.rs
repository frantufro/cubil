use crate::core::error::Result;
use crate::core::store::TaskStore;

/// Delete a task file.
///
/// Resolves the slug across all status folders and removes the file. Silent
/// on success.
pub fn run(slug: &str) -> Result<()> {
    TaskStore::discover()?.remove(slug)
}
