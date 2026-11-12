use crate::core::error::{CubilError, Result};
use crate::core::roadmap::{Roadmap, resolve_roadmap};
use crate::core::store::TaskStore;

/// Append `- [ ] <task_slug>` to a roadmap.
///
/// Validates that `task_slug` exists in some status folder (backlog, doing,
/// done, or any custom one). No forward references. See
/// [`Roadmap::add_task`] for placement and duplicate rules.
pub fn run(roadmap_slug: &str, task_slug: String, milestone: Option<&str>) -> Result<()> {
    let store = TaskStore::discover()?;
    let path = resolve_roadmap(store.root(), roadmap_slug)?;

    // Validate task exists somewhere. Ambiguity is tolerated here.
    if !store.lookup()?.contains_key(&task_slug) {
        return Err(CubilError::SlugNotFound(task_slug));
    }

    let mut roadmap = Roadmap::load(roadmap_slug, &path)?;
    roadmap.add_task(&task_slug, milestone)?;
    roadmap.save(&path)
}
