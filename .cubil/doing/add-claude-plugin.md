---
status: doing
title: Add Claude Code plugin
created: 2026-04-20
---

Ship a Claude Code plugin for `cubil` under `claude-plugin/`, mirroring
skulk's layout. Two files: `.claude-plugin/plugin.json` (manifest) and
`skills/cubil-task-management/SKILL.md` (the skill that teaches Claude
the task lifecycle: init → new → start → finish, plus list/show/edit/mv/rm).
Pure infra — no Rust changes.
