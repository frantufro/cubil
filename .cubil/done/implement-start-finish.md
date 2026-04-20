---
status: doing
title: Implement start and finish sugar commands
created: 2026-04-20
---

Add `cubil start <slug>` (backlog → doing) and `cubil finish <slug>` (doing → done)
as thin sugar wrappers over the existing move logic. Introduce a `StatusMismatch`
error variant so callers get a clear message when the task is in an unexpected
source status.
