---
created: 2026-11-12
---

# Add one Table/JSON output module

Replace the duplicated table and JSON rendering in the two list commands with one output module.

## Problem
`commands/list.rs` and `commands/roadmap/list.rs` each define render_table, write_row, render_json and push_json_string. `serde_json` is already a dependency (used by the updater).

## Scope
- Interface: headers, rows and a format (table or JSON).
- Use serde_json for escaping.
- Keep the existing output byte for byte; tests/list.rs and tests/roadmap.rs are the guard.

## Done when
- Both copies of push_json_string are deleted.
- Row and column rules are tested once.
