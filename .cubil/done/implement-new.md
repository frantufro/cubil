---
created: 2026-04-19
priority: 1
---

# Implement `cubil new`

Create a new task file in `.cubil/backlog/`.

## Usage
- `cubil new "title"` — creates file with frontmatter + `# title`, empty body.
- `cubil new "title" -m "body"` — inline body.
- `cubil new "title" -F file.md` — body from file.
- `cubil new "title" -F -` — body from stdin.

## Behavior
- Walks upward from cwd to find `.cubil/`; errors if not found.
- Derives slug from title (kebab-case, lowercase, ASCII-only).
- Writes `.cubil/backlog/<slug>.md` with frontmatter (`created: <today>`, `priority:`) + `# <title>` + body.
- Errors with non-zero exit if a task with the same slug exists in **any** status folder. Prints existing slug and its status to stderr.
- Prints slug to stdout on success (exact: `<slug>\n`).

## Acceptance
- [ ] All three body modes (`-m`, `-F file`, `-F -`) work.
- [ ] Slug collision across any status folder errors with non-zero exit.
- [ ] Stdout on success is exactly `<slug>\n`.
- [ ] Frontmatter `created` is today's date in ISO 8601 (YYYY-MM-DD).
