# Contributing

Thanks for your interest in improving `cubil`. The project is small enough
that a drive-by fix is welcome; the bar is green tests and clean lint.

## Ground rules

Before opening a PR, make sure these succeed:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
cargo test --locked
```

Changes under `bin/`, `claude-plugin/skills/` or `tests/node/` also need
`npm test`, which drives the OpenCode installer against a fixture HTTP server
on localhost. It runs offline and needs no `npm install`.

`.github/workflows/ci.yml` runs all four on every pull request and on every
push to `main`. The Rust suite runs on Linux and macOS, and the installer
suite runs on node 20 and 24.

### The pinned lint toolchain

The `lint` job pins rustc to a specific version. The pedantic set grows with
each release — the same tree reported 16 findings on 1.94 and 24 on 1.98 — so
a floating toolchain would turn an untouched branch red on rustc's schedule.
To reproduce CI exactly:

```bash
rustup toolchain install 1.98.0 --component rustfmt --component clippy
rustup run 1.98.0 cargo clippy --all-targets -- -D warnings -W clippy::pedantic
```

Dependabot is told to leave that pin alone, because the action tags version
branches ahead of the Rust releases they name. Move it by hand when a new
stable is out, in the same commit as any fixes its new lints require.

## Project layout

```
src/
├── main.rs              CLI definition (clap) and command dispatch
├── core/
│   ├── root.rs          Find the .cubil/ directory from any working directory
│   ├── slug.rs          Slugify titles, scan status folders, resolve a slug
│   ├── frontmatter.rs   YAML frontmatter parse and assemble
│   ├── roadmap.rs       Roadmap line grammar: task lines, headings, milestones
│   ├── updater.rs       Self-update: target detection, download, atomic replace
│   └── error.rs         CubilError and its Display
└── commands/            One module per command, with co-located unit tests
    ├── init.rs          Create .cubil/ with its status folders
    ├── new.rs           Create a task in backlog/
    ├── list.rs          Table and JSON listings
    ├── show.rs edit.rs  Read and edit a single task
    ├── mv.rs start.rs finish.rs rm.rs   Move between status folders
    ├── update.rs        Replace the running binary with the latest release
    └── roadmap/         new, list, show, add, next, rm
```

Integration tests live in `tests/`, one file per command, and drive the built
binary through `assert_cmd`. `tests/common/mod.rs` holds a fixture HTTP server
used by the update and stale-warning suites, so no test reaches the network.

The Claude Code plugin lives in `claude-plugin/`. Its `skills/` directory is
the single source of truth for the agent skill: the npm package packs it
verbatim, and `bin/install.mjs` copies it into OpenCode's skill directory.

## Making a release

Run the script. It takes an exact version or a `patch` / `minor` / `major`
bump:

```bash
script/release patch
```

It writes the new version to `Cargo.toml`, `Cargo.lock`, `package.json`,
`claude-plugin/.claude-plugin/plugin.json` and the console sample in
`README.md`, runs both test suites, and commits on a `release/vX.Y.Z` branch.
It refuses to run off `main`, on a dirty tree, when `main` and `origin/main`
have diverged, or when the tag already exists. `script/release --check`
reports whether the five files already agree.

Then open the PR, merge it, and push the tag:

```bash
gh pr create --fill
gh pr merge --squash --delete-branch
git checkout main && git pull
git tag vX.Y.Z && git push origin vX.Y.Z
```

Pushing the tag is what publishes, which is why the script stops before it.

`release.yml` opens with a `verify` job that asserts the tag, `Cargo.toml`,
`Cargo.lock`, `package.json` and `plugin.json` all read the same version, then
runs the installer suite. Every later job depends on it, so a version left
behind stops the release before a single artifact is built. Once `verify`
passes, the workflow builds binaries for macOS aarch64 and Linux
x86_64/aarch64, publishes the GitHub release, updates the Homebrew tap
formula, and publishes `@frantufro/cubil` to npm.

The npm publish uses OIDC trusted publishing, so the repository holds no npm
credential. The trust configuration lives at
<https://www.npmjs.com/package/@frantufro/cubil/access> and names organization
`frantufro`, repository `cubil`, workflow `release.yml`, with the environment
field left empty.
