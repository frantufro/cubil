---
created: 2026-08-28
---

# Add a release script that bumps the version everywhere

A release moves the version in five places: `Cargo.toml`, `Cargo.lock`,
`package.json`, the Claude Code plugin manifest, and the console sample in
`README.md`. Doing that by hand let the plugin manifest sit at `0.1.1` through
three releases, because the release workflow only compared the tag against
`Cargo.toml` and `package.json`.

Add `script/release`, which takes an exact version or `patch`/`minor`/`major`,
writes all five files, refreshes `Cargo.lock`, runs the Rust and Node test
suites, and commits the result on a `release/vX.Y.Z` branch. It prints the PR
and tag commands and stops there, so pushing the tag — the act that publishes
to GitHub Releases, Homebrew and npm — stays deliberate. `script/release
--check` reports whether the files already agree.

Maintainer tooling lives in `script/` because `package.json` listed `bin` as a
whole directory, which would have shipped a release script to every `npx` user.
The `files` entry now names `bin/install.mjs` directly.
