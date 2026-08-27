# Cubil

Cubil tracks tasks as Markdown files in a `.cubil/` directory, and ships an
agent skill so coding agents drive that lifecycle the same way a person does.
This glossary fixes the words used across the CLI, the skill, and the
installers.

## Language

### Tasks

**Task**:
A Markdown file in `.cubil/` describing one piece of work.
_Avoid_: ticket, issue, card, story

**Status**:
A subdirectory of `.cubil/` whose name is the state of every task inside it.
_Avoid_: state, column, stage

**Slug**:
The lowercase hyphenated identifier of a task, derived from its title and used
as its filename stem.
_Avoid_: id, key, name

**Roadmap**:
A Markdown file in `.cubil/roadmaps/` listing task slugs in dependency order.
_Avoid_: epic, plan, sprint

**Milestone**:
A `## Milestone: <name>` heading grouping consecutive entries of a roadmap.
_Avoid_: phase, release, stage

### Skill distribution

**Skill**:
A directory containing a `SKILL.md` whose YAML frontmatter carries a `name` and
a `description`, which an agent host loads to teach a model a capability.
_Avoid_: prompt, instructions, rule, agent

**Host**:
A program that discovers and loads skills. Cubil targets OpenCode and Claude
Code.
_Avoid_: client, editor, tool, IDE

**Scope**:
Which pair of directories an installation writes to. `global` is the host's
user config directory; `project` is a directory inside the current repository.
_Avoid_: level, location, target

**Installer**:
`bin/install.mjs`, published to npm as `cubil`, which places skills in a host's
directory and fetches the binary when one is missing.
_Avoid_: plugin, bootstrapper, setup script

**Binary**:
The compiled `cubil` executable that skill instructions invoke.
_Avoid_: CLI, tool, program

## Relationships

- A **Task** lives in exactly one **Status** and is addressed by its **Slug**
- A **Roadmap** references many **Tasks** by **Slug**, grouped under
  **Milestones**; a **Slug** may appear in several **Roadmaps**
- A **Skill** describes the **Binary**; the **Installer** delivers both
- The **Installer** writes one **Scope** per run, into one **Host**

## Flagged ambiguities

- "install" covers two operations that behave differently. `npx @frantufro/cubil install`
  places a **Skill** and, at most once, fetches a missing **Binary**. `cubil
  update` replaces an existing **Binary** in place. The **Installer** leaves
  any **Binary** it finds on `PATH` untouched and points at `cubil update`.
- "skills directory" is ambiguous between the packaged source
  (`claude-plugin/skills/`, the single source of truth for both hosts) and an
  installed **Scope** (`~/.config/opencode/skills/`,
  `./.opencode/skills/`). The source is authored; a scope is written.
- "cubil" names three things: this project, the **Binary**, and the npm
  package. The npm package publishes an **Installer** whose executable is
  `cubil-install`, which keeps `cubil` on `PATH` meaning the **Binary**.

## Example dialogue

> **Dev:** "If I add the skill with `--project`, does that replace what's in my
> home directory?"
>
> **Maintainer:** "Each run writes one **Scope**. `--project` writes
> `./.opencode/skills/`, and your global copy stays where it is. OpenCode loads
> both, and when two **Skills** share a `name` it warns and keeps the last one
> it read."
>
> **Dev:** "So the project copy wins?"
>
> **Maintainer:** "Treat that as undefined and keep one **Scope** per machine.
> Commit the project copy when the team should share the **Skill**; use the
> global copy for your own machines."
