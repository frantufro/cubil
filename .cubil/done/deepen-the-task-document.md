---
created: 2026-11-12
---

# Deepen the Task document

Create a Task document module that parses and renders the whole Task file: frontmatter, title and body.

## Problem
`new.rs` writes the title as the first H1, and `core/roadmap.rs` (read_task_title) reads it back. `assemble` and `read_body` exist in both `commands/new.rs` and `commands/roadmap/new.rs`. `list.rs` reparses frontmatter on its own.

## Scope
- Interface: TaskDoc::new(title, body, date), TaskDoc::parse(text), .title(), .meta(), .render().
- Absorb `core/frontmatter.rs`.
- Share one read_body helper between both new commands.

## Done when
- The H1 title rule lives in one module.
- Round-trip tests run through the interface.

Depends on: deepen-the-roadmap-module.
