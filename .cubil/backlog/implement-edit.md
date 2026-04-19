---
created: 2026-04-19
priority: 3
---

# Implement `cubil edit`

Open a task file in the user's `$EDITOR` for manual editing.

## Usage
- `cubil edit <slug>`

## Behavior
- Exact slug match across all status folders.
- Launches `$EDITOR` on the file; falls back to `vi` if unset.
- Blocks until the editor exits.
- Errors non-zero if slug not found.

## Acceptance
- [ ] Uses `$EDITOR` when set.
- [ ] Falls back to `vi` when unset.
- [ ] Missing slug errors with non-zero exit.
