# The npm package named `cubil` installs the OpenCode skill

OpenCode reads skills from directories on disk, so shipping cubil's agent skill
to OpenCode users means getting a `SKILL.md` into one of them. We publish a
zero-dependency npm package named `cubil` whose whole job is `npx cubil
install`: it copies `claude-plugin/skills/` into `~/.config/opencode/skills/`
(or `./.opencode/skills/` with `--project`) and downloads the release binary
when `cubil` is absent from `PATH`. The bare npm name was unclaimed and is the
one people reach for, which keeps the install line short.

## Consequences

The package's executable is named `cubil-install`, and that name is load
bearing. An npm package named `cubil` that put an executable named `cubil` on
`PATH` would offer an installer where users expect the task CLI, and a global
install could shadow the Homebrew or `install.sh` copy with `PATH` order
silently deciding which one runs. A differently named executable makes `npm i -g
cubil` harmless: npm exposes `cubil-install` and the real binary keeps its
place. `npx cubil install` still works, because npm runs a package's single
declared executable whatever that executable is called.

Two more consequences follow from the shape:

- The installer copies skill files into place. An OpenCode plugin can add its
  own directory to `config.skills.paths` at startup, which serves the skill
  with no copy at all, and published OpenCode plugins do work that way. We
  chose the one-shot installer so the skill keeps working with no plugin
  loaded, no entry in `opencode.json`, and under `opencode --pure`. Updates
  then happen when someone runs the installer again.
- `install` downloads the latest release. The npm package's own version plays
  no part in choosing it, so an old package still lands a current binary, and
  the shipped skill file can describe a different command set than the binary
  beside it. `npx cubil@latest install` refreshes both together.
