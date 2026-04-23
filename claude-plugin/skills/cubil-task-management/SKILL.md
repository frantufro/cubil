---
name: cubil-task-management
description: >
  Manage markdown-based tasks with cubil. Use this skill when the user wants
  to create, list, inspect, edit, or move tasks between status folders
  (backlog → doing → done) stored as plain Markdown in a .cubil/ directory.
  Covers the full task lifecycle: init, new, list, show, edit, start, finish,
  mv, rm. TRIGGER when: user mentions cubil, a .cubil/ directory, markdown
  task files, or wants to capture/track work as task files in a repo.
allowed-tools: [Bash, Read, Glob, Grep]
---

# Cubil Task Management

Cubil stores tasks as plain Markdown files in a `.cubil/` directory. Each
subdirectory is a status: `backlog/`, `doing/`, `done/`. Tasks are files with
optional YAML frontmatter. Cubil is agent-first — `new` takes a title plus a
body and prints the slug on stdout; status transitions are explicit.

## Prerequisites

The project needs a `.cubil/` directory. If it doesn't exist, run
`cubil init` — creates `.cubil/backlog/`, `.cubil/doing/`, `.cubil/done/`.

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

## Important Notes

- Tasks are plain Markdown files — version them with git like anything else.
- `.cubil/` is project-local; each repo has its own task board.
- Slug collisions across statuses are surfaced as errors (use `cubil show
  <slug>` to confirm which task you mean if a slug is ambiguous).
- `cubil start` and `cubil finish` are strict about source status — this is
  intentional. Use `cubil mv` for non-linear transitions.
- YAML frontmatter is optional but `cubil` preserves whatever it finds.
