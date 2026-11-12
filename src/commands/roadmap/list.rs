use std::path::Path;

use crate::commands::output::{self, Cell, Format};
use crate::core::error::Result;
use crate::core::roadmap::{extract_roadmap_title, list_roadmap_slugs, roadmap_path};
use crate::core::root::find_root;

const HEADERS: &[&str] = &["slug", "title"];

struct Row {
    slug: String,
    title: Option<String>,
}

pub fn run(json: bool) -> Result<()> {
    let root = find_root(None)?;
    let rows = collect_rows(&root)?;
    output::print(Format::from_json_flag(json), HEADERS, &to_cells(&rows));
    Ok(())
}

fn collect_rows(root: &Path) -> Result<Vec<Row>> {
    let mut slugs = list_roadmap_slugs(root)?;
    slugs.sort();
    let mut rows = Vec::with_capacity(slugs.len());
    for slug in slugs {
        let path = roadmap_path(root, &slug);
        let title = match std::fs::read_to_string(&path) {
            Ok(src) => extract_roadmap_title(&src),
            Err(_) => None,
        };
        rows.push(Row { slug, title });
    }
    Ok(rows)
}

fn to_cells(rows: &[Row]) -> Vec<Vec<Cell>> {
    rows.iter()
        .map(|r| vec![Cell::text(&r.slug), Cell::opt_text(r.title.as_deref())])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(slug: &str, title: Option<&str>) -> Row {
        Row {
            slug: slug.into(),
            title: title.map(String::from),
        }
    }

    #[test]
    fn to_cells_maps_missing_title_to_null() {
        let rows = vec![row("foo", Some("Foo Roadmap")), row("bar", None)];
        assert_eq!(
            to_cells(&rows),
            vec![
                vec![Cell::text("foo"), Cell::text("Foo Roadmap")],
                vec![Cell::text("bar"), Cell::Null],
            ]
        );
    }
}
