---
created: 2026-04-19
priority: 2
---

# Implement `cubil mv`

Move a task between status folders.

## Usage
- `cubil mv <slug> <status>`

## Behavior
- Exact slug match across all status folders.
- Moves the file to `.cubil/<status>/<slug>.md`.
- Errors non-zero if the destination status folder does not exist — users create new statuses by running `mkdir .cubil/<status>/` themselves (filesystem is the source of truth).
- Errors non-zero if slug not found.
- No-op (silent success) if the task is already in the target status.

## Acceptance
- [ ] Moving across existing folders works.
- [ ] Moving to a nonexistent status folder errors.
- [ ] Moving to current status is a silent no-op.
- [ ] Missing slug errors.
