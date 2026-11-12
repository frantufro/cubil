---
created: 2026-11-12
---

# Deepen the Task store

Create a Task store module that owns finding, listing, creating, moving and removing Tasks across Statuses.

## Problem
Every command repeats root discovery, scanning and slug resolution. `commands/mv.rs` repeats `transition()` in `commands/mod.rs` almost line for line. `roadmap/show.rs`, `roadmap/next.rs` and `roadmap/add.rs` each build their own slug-to-status lookup. The "backlog" literal appears in several files.

## Scope
- Interface: find(slug), tasks(), create(slug, text), move_to(slug, status), remove(slug).
- Collision, ambiguity, reserved-directory and missing-Status rules live behind the interface.
- Files: src/core/slug.rs, src/commands/{mod,mv,rm,new,show,edit,start,finish,list}.rs, src/commands/roadmap/{show,next,add}.rs.

## Done when
- mv and start/finish share one move path.
- The roadmap commands take their lookup from the Task store.
- Tests exercise the Task store through its interface against a tempdir.
