---
created: 2026-04-19
priority: 1
---

# Implement `cubil list`

Display tasks in a human-readable table, with JSON opt-in for agents.

## Usage
- `cubil list` — active tasks only (excludes `done/`).
- `cubil list --all` — include done.
- `cubil list --status <name>` — only that status folder.
- `cubil list --json` — JSON array, suitable for scripting.

## Behavior
- Auto-discovers status folders: any subdirectory of `.cubil/` is a status.
- Columns: `slug`, `status`, `priority`, `created`.
- JSON output: array of objects with the same keys. Body is NOT included; use `cubil show` for body.
- Empty status folders are fine (rendered as empty sections or omitted).
- Walks upward from cwd to find `.cubil/`.

## Acceptance
- [ ] Default view hides `done/`.
- [ ] Table is column-aligned.
- [ ] `--json` emits valid, parseable JSON.
- [ ] `--status done` includes done even without `--all`.
- [ ] Works when a status folder is empty.
