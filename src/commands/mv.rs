use crate::core::error::Result;
use crate::core::store::TaskStore;

/// Move task `<slug>` into `<root>/<status>/`.
///
/// If the task is already in `status`, returns `Ok(())` without touching the
/// file or printing anything. Every other rule (missing status folder, slug
/// collision, ambiguity) lives in [`TaskStore::move_to`].
pub fn run(slug: &str, status: &str) -> Result<()> {
    TaskStore::discover()?.move_to(slug, status)
}
