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
