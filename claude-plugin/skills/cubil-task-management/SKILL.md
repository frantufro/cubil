---
name: cubil-task-management
description: >
  Manage markdown-based tasks with cubil. Use this skill when the user wants
  to create, list, inspect, edit, or move tasks between status folders
  (backlog → doing → done) stored as plain Markdown in a .cubil/ directory.
  Covers the full task lifecycle: init, new, list, show, edit, start, finish,
  mv, rm — plus roadmaps that group tasks into ordered, milestone-divided
  sequences. TRIGGER when: user mentions cubil, a .cubil/ directory, markdown
  task files, roadmaps, milestones, or wants to capture/track work as task
  files in a repo.
allowed-tools: [Bash, Read, Glob, Grep]
---

# Cubil Task Management

Cubil stores tasks as plain Markdown files in a `.cubil/` directory. Each
subdirectory is a status: `backlog/`, `doing/`, `done/`. Tasks are files with
optional YAML frontmatter. Cubil is agent-first — `new` takes a title plus a
body and prints the slug on stdout; status transitions are explicit.

## Prerequisites

The project needs a `.cubil/` directory. If it doesn't exist, run
`cubil init` — creates `.cubil/backlog/`, `.cubil/doing/`, `.cubil/done/`,
and `.cubil/roadmaps/`.

## Task Lifecycle

### 1. Create a Task

```bash
# Inline body
cubil new "Fix login timeout" -m "Session expires after 30s instead of 30m."

# From a file
cubil new "Refactor auth module" -F notes/auth-refactor.md

# From stdin
echo "Add retry logic to the API client." | cubil new "Retry logic" -F -

# Title only (empty body)
cubil new "Quick note"
```

`new` writes to `.cubil/backlog/<slug>.md` and prints the slug to stdout.
The slug is derived from the title (lowercase, hyphen-separated).

### 2. List Tasks

```bash
# Active tasks (backlog + doing; done hidden)
cubil list

# Include done/
cubil list --all

# Filter to one status
cubil list --status doing

# JSON output (for scripting)
cubil list --json
```

### 3. Inspect a Task

```bash
# Print full markdown to stdout
cubil show <slug>

# Open in $EDITOR (falls back to vi)
cubil edit <slug>
```

### 4. Transition Statuses

```bash
# backlog/ → doing/
cubil start <slug>

# doing/ → done/
cubil finish <slug>

# Arbitrary status move (destination folder must already exist)
cubil mv <slug> <status>
```

`start` errors if the task is not in `backlog/`; `finish` errors if it's
not in `doing/`. Use `mv` for any other transition.

### 5. Delete a Task

```bash
cubil rm <slug>
```

## Command Reference

| Command                  | Purpose                                              |
|--------------------------|------------------------------------------------------|
| `cubil init`             | Create `.cubil/` with `backlog/`, `doing/`, `done/`. |
| `cubil new <title>`      | Create a task in `backlog/`. Prints the slug.        |
| `cubil list`             | List active tasks. `--all`, `--status`, `--json`.    |
| `cubil show <slug>`      | Print the task's full markdown to stdout.            |
| `cubil edit <slug>`      | Open the task in `$EDITOR`.                          |
| `cubil start <slug>`     | Move task from `backlog/` to `doing/`.               |
| `cubil finish <slug>`    | Move task from `doing/` to `done/`.                  |
| `cubil mv <slug> <dir>`  | Move task to an arbitrary status folder.             |
| `cubil rm <slug>`        | Delete a task.                                       |

### Roadmap subcommands

| Command                                  | Purpose                                                   |
|------------------------------------------|-----------------------------------------------------------|
| `cubil roadmap new <title>`              | Create a roadmap in `roadmaps/`. Prints the slug.         |
| `cubil roadmap list`                     | List all roadmaps (slug + title). `--json`.               |
| `cubil roadmap show <slug>`              | Render resolved statuses; rewrites the file on disk.      |
| `cubil roadmap add <rm-slug> <task-slug>`| Add a task to the roadmap. `--milestone <name>`.          |
| `cubil roadmap next <rm-slug>`           | Print the next not-done task slug (for pipelines).        |
| `cubil roadmap rm <slug>`                | Delete a roadmap.                                         |

## Common Workflows

### Capture an idea, work on it, finish it
```bash
cubil new "Add dark mode" -m "Toggle in settings, persist in localStorage."
# → prints slug: add-dark-mode
cubil start add-dark-mode        # move to doing/
# ... do the work ...
cubil finish add-dark-mode       # move to done/
```

### Break a big task into subtasks
```bash
cubil new "Migrate to Postgres" -F plan.md
cubil new "Postgres: schema migration" -m "DDL + data copy."
cubil new "Postgres: update connection strings" -m "Env vars + CI."
cubil list --status backlog
```

### Quick status check
```bash
cubil list                       # what's active
cubil show <slug>                # drill into one task
```

### Agent pipeline (slug on stdout)
```bash
slug=$(cubil new "Auto-generated task" -m "Body from the agent.")
cubil start "$slug"
# ... agent does the work ...
cubil finish "$slug"
```

## Roadmaps

A roadmap groups existing tasks into an ordered, milestone-divided sequence.
It expresses dependency order (task N depends on tasks before it) and
step-by-step processes **without adding fields to individual tasks**. Roadmaps
live as plain Markdown at `.cubil/roadmaps/<slug>.md` in a flat folder — no
statuses, no archiving.

### File format

```markdown
# Migrate to Postgres

Optional narrative.

## Milestone: Schema ready
- [ ] postgres-schema-migration
- [ ] postgres-data-copy

## Milestone: Cutover
- [ ] postgres-update-connection-strings
```

- Milestones are just `## Milestone: <name>` headings — purely organizational,
  no metadata or deadlines.
- Tasks are referenced by slug in `- [ ] <task-slug>` list items. The roadmap
  never duplicates task content; the task files stay in `backlog/`, `doing/`,
  or `done/`.
- File order expresses dependency order. A slug may appear in multiple roadmaps.

### Create, list, inspect

```bash
# Create (mirrors `cubil new`: -m / -F <file> / -F - for stdin)
cubil roadmap new "Migrate to Postgres" -m "Optional narrative."
# → prints slug: migrate-to-postgres

cubil roadmap list               # slug + title; --json for scripting
cubil roadmap show migrate-to-postgres
```

`cubil roadmap show` resolves each slug's status from the actual task files and
**writes the resolved checkboxes (and synced titles) back to the file**, so the
roadmap always reflects current state after a `show`. Status markers:

| Marker      | Meaning                                          |
|-------------|--------------------------------------------------|
| `- [✓]`     | Task is in `done/`.                              |
| `- [~]`     | Task is in `doing/`.                             |
| `- [ ]`     | Task is in `backlog/` (or any other status).     |
| `- [?]`     | Slug referenced but no task file exists.         |

### Add tasks to a roadmap

```bash
# Append to the end of the last milestone (or end of file if none)
cubil roadmap add migrate-to-postgres postgres-data-copy

# Append into a specific milestone (errors if it doesn't exist)
cubil roadmap add migrate-to-postgres postgres-cutover --milestone "Cutover"
```

`add` validates that the task slug exists in `backlog/`, `doing/`, or `done/` —
no forward references to nonexistent tasks.

### Drive work through a roadmap (agent pipeline)

`cubil roadmap next` prints the slug of the first not-done task (status
`backlog` or `doing`), or empty output with a zero exit when all are done:

```bash
slug=$(cubil roadmap next migrate-to-postgres)
cubil start "$slug"
# ... do the work ...
cubil finish "$slug"
```

### Delete a roadmap

```bash
cubil roadmap rm migrate-to-postgres
```

## Important Notes

- Tasks are plain Markdown files — version them with git like anything else.
- `.cubil/` is project-local; each repo has its own task board.
- Slug collisions across statuses are surfaced as errors (use `cubil show
  <slug>` to confirm which task you mean if a slug is ambiguous).
- `cubil start` and `cubil finish` are strict about source status — this is
  intentional. Use `cubil mv` for non-linear transitions.
- YAML frontmatter is optional but `cubil` preserves whatever it finds.
- Roadmaps only reference tasks by slug — deleting a task leaves a `- [?]`
  marker in any roadmap that referenced it, surfaced on the next `roadmap show`.
