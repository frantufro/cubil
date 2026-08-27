# cubil

Markdown task files. Git-native. Agent-friendly.

Cubil is a tiny CLI for tracking tasks as plain Markdown files in a `.cubil/`
directory. Each subdirectory is a status — `backlog/`, `doing/`, `done/` —
and moving a task between statuses is just moving the file. No database,
no server, no config. It works with git out of the box, and it's the
companion to [Skulk](https://github.com/frantufro/skulk) so coding agents
can pick up tasks the same way humans do.

## Example

```console
$ cubil init
/tmp/demo/.cubil

$ cubil new 'fix login bug' -m 'Session cookies dropped on Safari after redirect.'
fix-login-bug

$ cubil list
slug           status   priority  created
fix-login-bug  backlog  -         2026-04-20

$ cubil start fix-login-bug

$ cubil show fix-login-bug
---
created: 2026-04-20
---

# fix login bug

Session cookies dropped on Safari after redirect.

$ cubil finish fix-login-bug

$ cubil list --all
slug           status  priority  created
fix-login-bug  done    -         2026-04-20
```

Under the hood, that's just `.cubil/backlog/fix-login-bug.md` being renamed
into `.cubil/doing/` and then `.cubil/done/`. Commit the directory and your
task board is versioned with your code.

## Install

```bash
curl -sSL https://raw.githubusercontent.com/frantufro/cubil/main/install.sh | sh
```

Or via Homebrew (macOS and Linux):

```bash
brew install frantufro/tap/cubil
```

Or build and install from source:

```bash
git clone https://github.com/frantufro/cubil.git
cd cubil
cargo install --path .
```

### Use cubil from OpenCode

Cubil ships an agent skill that teaches a coding agent the whole task
lifecycle. One command installs it into
[OpenCode](https://opencode.ai) and fetches the `cubil` binary if the
machine does not have one yet:

```console
$ npx cubil install
  ✓ skill   installed  ~/.config/opencode/skills/cubil-task-management
  ↓ binary  downloading cubil 0.1.4 for aarch64-apple-darwin…
  ✓ binary  ~/.local/bin/cubil
```

OpenCode discovers the skill on its next start and offers it to the model
whenever a session touches tasks, roadmaps or a `.cubil/` directory.

| Flag | Effect |
| --- | --- |
| `--project` | Install into `./.opencode/skills/`, so the skill is committed with the repo and shared with the team. |
| `--no-binary` | Install the skill alone. Use this when `cubil` comes from Homebrew or `install.sh`. |
| `--force` | Overwrite an existing `SKILL.md` without asking. |
| `--skip-existing` | Keep an existing `SKILL.md` and exit successfully. |

Re-running `install` reports whether the skill is current and rewrites it
when it is stale. When the file has local edits the installer asks before
replacing it, and with no terminal to ask on it stops and points at
`--force` or `--skip-existing`.

An existing `cubil` on `PATH` is left where it is. Upgrades go through
`cubil update`, which replaces the binary in place wherever it was
installed. Run `npx cubil@latest install` to pick up a newer skill file.

Claude Code users get the same skill through the plugin in
[`claude-plugin/`](./claude-plugin).

## Commands

| Command | Description |
| --- | --- |
| `cubil init` | Create `.cubil/` with default status folders (`backlog`, `doing`, `done`). |
| `cubil new <title>` | Create a task in `backlog/`. Body via `-m <text>`, `-F <path>`, or `-F -` for stdin. Prints the slug. |
| `cubil list` | List active tasks. Use `--all` to include `done/`, `--status <name>` to filter, `--json` for machine output. |
| `cubil show <slug>` | Print a task's full markdown to stdout. |
| `cubil edit <slug>` | Open a task in `$EDITOR` (falls back to `vi`). |
| `cubil mv <slug> <status>` | Move a task to a different status folder. |
| `cubil start <slug>` | Move a task from `backlog/` to `doing/`. |
| `cubil finish <slug>` | Move a task from `doing/` to `done/`. |
| `cubil rm <slug>` | Delete a task. |

## Companion to Skulk

[Skulk](https://github.com/frantufro/skulk) orchestrates remote coding agents.
Cubil is the task layer they share with you: agents read `.cubil/backlog/`,
move things into `doing/`, write their working notes into the task body,
and land them in `done/` — all as plain files in git.

## License

MIT
