pub mod edit;
pub mod finish;
pub mod init;
pub mod list;
pub mod mv;
pub mod new;
pub mod rm;
pub mod roadmap;
pub mod show;
pub mod start;
pub mod update;

use crate::core::error::Result;
use crate::core::store::TaskStore;

/// Move a task from `expected` status to `target` status.
///
/// Errors with [`crate::core::error::CubilError::StatusMismatch`] if the task
/// exists but is in a different status folder. Every other rule lives in
/// [`TaskStore::transition`].
pub(crate) fn transition(slug: &str, expected: &str, target: &str) -> Result<()> {
    TaskStore::discover()?.transition(slug, expected, target)
}
