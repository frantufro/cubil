---
created: 2026-04-19
priority: 1
---

# Implement `cubil init`

Create the `.cubil/` directory structure in the current working directory.

## Behavior
- Creates `.cubil/backlog/`, `.cubil/doing/`, `.cubil/done/` if they don't exist.
- Idempotent: running again is a no-op; does not error.
- Fails loudly if `.cubil/` already exists as a file (not directory).
- Prints the created/confirmed path on success.

## Acceptance
- [ ] `cubil init` in an empty directory creates all three status folders.
- [ ] Running twice does not error or duplicate.
- [ ] Creates `.cubil/` at cwd (does not walk upward).
