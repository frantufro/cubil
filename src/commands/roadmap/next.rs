use std::collections::HashMap;

use crate::core::error::Result;
use crate::core::roadmap::{Roadmap, TaskStatus, resolve_roadmap};
use crate::core::store::TaskStore;

/// Print the slug of the next not-done task in the roadmap.
///
/// See [`Roadmap::next_not_done`]. Empty stdout + zero exit if all tasks are
/// done. Designed for shell pipelines.
pub fn run(slug: &str) -> Result<()> {
    let store = TaskStore::discover()?;
    let path = resolve_roadmap(store.root(), slug)?;
    let roadmap = Roadmap::load(slug, &path)?;

    let lookup = build_status_lookup(&store)?;
    let next = roadmap.next_not_done(|task| {
        lookup
            .get(task)
            .map_or(TaskStatus::Missing, |s| TaskStatus::from_status_name(s))
    });
    if let Some(next) = next {
        println!("{next}");
    }
    Ok(())
}

fn build_status_lookup(store: &TaskStore) -> Result<HashMap<String, String>> {
    Ok(store
        .lookup()?
        .into_iter()
        .map(|(slug, entry)| (slug, entry.status))
        .collect())
}
