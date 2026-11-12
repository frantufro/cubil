---
created: 2026-11-12
---

# Deepen the Roadmap module

Create a Roadmap module that parses a roadmap once and offers render_resolved, add_task and next_not_done.

## Problem
The task-line and Milestone grammar lives in `core/roadmap.rs`, while rewriting (`roadmap/show.rs`), inserting (`roadmap/add.rs`) and next-task selection (`roadmap/next.rs`) each re-read it. `write_atomic` is copied in show and add. `root_for_roadmap` forwards to `find_root`. About 40 unit tests pin private helpers.

## Scope
- Interface: Roadmap::parse(src), .render_resolved(tasks), .add_task(slug, milestone), .next_not_done(tasks), plus one atomic save.
- Move `TaskStatus` next to the Task store.
- Commands parse, call one method and print.

## Done when
- One copy of write_atomic.
- Unit tests for line handling move to the Roadmap interface.
- tests/roadmap.rs passes unchanged.

Depends on: deepen-the-task-store.
