---
created: 2026-08-29
---

# Run fmt, clippy and the test suites on every pull request

The only automation on a cubil pull request is the release workflow, which
fires on tags and never on a PR. Nothing compiles the branch, runs the 175
Rust tests, or runs the 14 installer tests before a merge.

Add `.github/workflows/ci.yml` on `pull_request` and on `push` to `main`:

- `lint` on a pinned 1.98.0 toolchain: `cargo fmt --check` and
  `cargo clippy --all-targets -- -D warnings -W clippy::pedantic`
- `test` on `ubuntu-22.04` and `macos-14`, `cargo test --locked`
- `installer` on node 18 and 24, `npm test`

The tree needs a formatting pass and 24 pedantic fixes before the workflow can
go green. Add `.github/dependabot.yml` for the cargo and github-actions
ecosystems, weekly.

Branch protection stays off until the workflow has a few green runs behind it.
