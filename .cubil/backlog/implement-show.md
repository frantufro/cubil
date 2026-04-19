---
created: 2026-04-19
priority: 2
---

# Implement `cubil show`

Print a task's full markdown content to stdout.

## Usage
- `cubil show <slug>` — prints file contents verbatim.

## Behavior
- Exact slug match across all status folders.
- Output is the raw markdown (frontmatter + body), byte-identical to the file.
- Errors non-zero if slug not found.
- Walks upward from cwd to find `.cubil/`.

## Acceptance
- [ ] Missing slug errors with non-zero exit.
- [ ] Output is byte-identical to file contents.
- [ ] Resolves regardless of which status folder holds the task.
