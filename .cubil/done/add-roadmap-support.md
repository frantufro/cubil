---
created: 2026-05-05
---

# Add roadmap support

Add a `roadmap` concept to cubil for grouping tasks into ordered, milestone-divided sequences. The goal is to express dependencies (via order) and step-by-step processes without adding new fields to individual tasks.

## Storage

- Roadmaps live as plain markdown files at `.cubil/roadmaps/<slug>.md`.
- Flat folder — no statuses (no `active/`, `archived/`, etc).
- `cubil init` should create `.cubil/roadmaps/` with a `.gitkeep`.

## File format

A roadmap file looks like this:

```markdown
# Migrate to Postgres

Optional narrative.

## Milestone: Schema ready
- [ ] postgres-schema-migration
- [ ] postgres-data-copy

## Milestone: Cutover
- [ ] postgres-update-connection-strings
```

- Milestones are just `## Milestone: <name>` headings — no metadata, no deadlines. Purely organizational.
- Tasks are referenced by slug in `- [ ] <task-slug>` list items.
- Order in the file expresses dependency order (task N depends on tasks 1..N-1, implicitly).
- Tasks themselves stay where they are (`backlog/`, `doing/`, `done/`) — the roadmap only references slugs, never duplicates task content.
- A task slug may appear in multiple roadmaps.

## Commands

### `cubil roadmap new <title> [-m <body> | -F <file>]`
Creates `.cubil/roadmaps/<slug>.md`. Mirrors `cubil new` semantics (slug derived from title, prints slug on stdout, supports `-m`/`-F`/`-F -`).

### `cubil roadmap list [--json]`
Lists all roadmaps. Show slug + title.

### `cubil roadmap show <slug>`
Renders the roadmap with task statuses **resolved** from the actual task files:
- `- [✓] <slug> — <title>` if the task is in `done/`
- `- [~] <slug> — <title>` if in `doing/`
- `- [ ] <slug> — <title>` if in `backlog/`
- Missing slugs (referenced in roadmap but no task file exists) render with a warning marker, e.g. `- [?] <slug> — (missing)`.

In addition to printing the resolved view, `show` **writes the updated checkboxes back to the file on disk**, so the file always reflects current state after a `show`. Task titles in the roadmap markdown are kept in sync the same way.

### `cubil roadmap add <roadmap-slug> <task-slug> [--milestone <name>]`
Appends `- [ ] <task-slug>` to the roadmap.
- Default insertion: end of the last milestone in the file. If there are no milestones, append to end of file.
- With `--milestone <name>`: append to the end of that milestone's section (error if the milestone doesn't exist).
- Validates that `<task-slug>` exists in `backlog/`, `doing/`, or `done/`. Errors if not (no forward references).

### `cubil roadmap next <roadmap-slug>`
Prints the slug of the next not-done task in the roadmap (first task whose status is `backlog` or `doing`) on stdout. Empty output + zero exit if all tasks are done. Designed for agent pipelines:

```bash
slug=$(cubil roadmap next migrate-to-postgres)
cubil start "$slug"
# ... do work ...
cubil finish "$slug"
```

### `cubil roadmap rm <slug>`
Deletes the roadmap file. Symmetric with `cubil rm`.

## Tests

- Roadmap creation, listing, show with all four status markers (`✓`, `~`, ` `, `?`).
- `show` rewrites the file with resolved markers.
- `add` default placement (end of last milestone), `--milestone` placement, missing milestone error, missing task slug error.
- `next` returns first non-done task (including `doing/`), empty when all done.
- `cubil init` creates `.cubil/roadmaps/.gitkeep`.

## Out of scope for this task

- Milestone metadata (deadlines, target dates).
- Roadmap statuses / archiving.
- Cross-roadmap views.
- Dependency graphs beyond sequential ordering.
