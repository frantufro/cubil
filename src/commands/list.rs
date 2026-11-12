use std::cmp::Ordering;

use crate::commands::output::{self, Cell, Format};
use crate::core::error::{CubilError, Result};
use crate::core::store::{TaskEntry, TaskStore};
use crate::core::task_doc::TaskDoc;

const DONE: &str = "done";
const HEADERS: &[&str] = &["slug", "status", "priority", "created"];

struct Row {
    slug: String,
    status: String,
    priority: Option<u32>,
    created: Option<String>,
}

pub fn run(all: bool, status: Option<&str>, json: bool) -> Result<()> {
    let store = TaskStore::discover()?;
    let rows = collect_rows(&store, all, status)?;
    output::print(Format::from_json_flag(json), HEADERS, &to_cells(&rows));
    Ok(())
}

fn collect_rows(store: &TaskStore, all: bool, status: Option<&str>) -> Result<Vec<Row>> {
    if let Some(name) = status
        && !store.has_status(name)
    {
        return Err(CubilError::StatusMissing(name.to_string()));
    }
    let mut rows: Vec<Row> = Vec::new();
    for TaskEntry {
        status: entry_status,
        slug,
        path,
    } in store.tasks()?
    {
        let keep = match status {
            Some(name) => entry_status == name,
            None if all => true,
            None => entry_status != DONE,
        };
        if !keep {
            continue;
        }
        let src = std::fs::read_to_string(&path)?;
        let doc = TaskDoc::parse(&src);
        let meta = doc.meta();
        rows.push(Row {
            slug,
            status: entry_status,
            priority: meta.priority,
            created: meta.created.clone(),
        });
    }
    rows.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| cmp_priority(a.priority, b.priority))
            .then_with(|| a.slug.cmp(&b.slug))
    });
    Ok(rows)
}

fn cmp_priority(a: Option<u32>, b: Option<u32>) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn to_cells(rows: &[Row]) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|r| {
            vec![
                Cell::text(&r.slug),
                Cell::text(&r.status),
                Cell::opt_number(r.priority),
                Cell::opt_text(r.created.as_deref()),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(slug: &str, status: &str, priority: Option<u32>, created: Option<&str>) -> Row {
        Row {
            slug: slug.into(),
            status: status.into(),
            priority,
            created: created.map(String::from),
        }
    }

    #[test]
    fn to_cells_maps_missing_values_to_null() {
        let rows = vec![
            row("a", "backlog", Some(1), Some("2026-04-19")),
            row("b", "doing", None, None),
        ];
        assert_eq!(
            to_cells(&rows),
            vec![
                vec![
                    Cell::text("a"),
                    Cell::text("backlog"),
                    Cell::Number(1),
                    Cell::text("2026-04-19")
                ],
                vec![Cell::text("b"), Cell::text("doing"), Cell::Null, Cell::Null],
            ]
        );
    }

    #[test]
    fn cmp_priority_puts_none_last() {
        assert_eq!(cmp_priority(Some(1), Some(2)), Ordering::Less);
        assert_eq!(cmp_priority(Some(1), None), Ordering::Less);
        assert_eq!(cmp_priority(None, Some(1)), Ordering::Greater);
        assert_eq!(cmp_priority(None, None), Ordering::Equal);
    }
}
