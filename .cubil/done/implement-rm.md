---
created: 2026-04-19
priority: 3
---

# Implement `cubil rm`

Delete a task file.

## Usage
- `cubil rm <slug>`

## Behavior
- Exact slug match across all status folders.
- Deletes the file.
- Agent-first: no confirmation prompt; silent on success.
- Errors non-zero if slug not found.

## Acceptance
- [ ] Silent success on deletion.
- [ ] Missing slug errors with non-zero exit.
- [ ] Works regardless of which status folder contains the file.
