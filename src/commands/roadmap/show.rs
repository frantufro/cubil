use std::collections::HashMap;
use std::io::Write as _;
use std::path::PathBuf;

use crate::core::error::Result;
use crate::core::roadmap::{Roadmap, TaskInfo, TaskStatus, read_task_title, resolve_roadmap};
use crate::core::store::TaskStore;

/// Render the roadmap with task statuses resolved from the actual task files,
/// then write the resolved view back to disk.
///
/// See [`Roadmap::render_resolved`] for the resolution rules.
pub fn run(slug: &str) -> Result<()> {
    let store = TaskStore::discover()?;
    let path = resolve_roadmap(store.root(), slug)?;
    let roadmap = Roadmap::load(slug, &path)?;

    let lookup = build_lookup(&store)?;
    let new_src = roadmap.render_resolved(|task| resolve(task, &lookup))?;

    Roadmap::parse(slug, &new_src).save(&path)?;

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(new_src.as_bytes())?;
    Ok(())
}

/// Slug to `(status folder name, task file path)`.
type Lookup = HashMap<String, (String, PathBuf)>;

fn build_lookup(store: &TaskStore) -> Result<Lookup> {
    // First-write-wins for ambiguous slugs. Roadmap rendering is robust
    // to ambiguity: it just picks one. Users discover the duplicate via
    // `cubil list` or `cubil show`.
    Ok(store
        .lookup()?
        .into_iter()
        .map(|(slug, entry)| (slug, (entry.status, entry.path)))
        .collect())
}

fn resolve(slug: &str, lookup: &Lookup) -> Result<TaskInfo> {
    match lookup.get(slug) {
        None => Ok(TaskInfo {
            status: TaskStatus::Missing,
            title: None,
        }),
        Some((status_name, path)) => Ok(TaskInfo {
            status: TaskStatus::from_status_name(status_name),
            title: read_task_title(path)?,
        }),
    }
}
