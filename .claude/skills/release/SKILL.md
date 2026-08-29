---
name: release
description: Cut a new cubil release. Use when the user asks to release, publish, ship or tag a new version, or to bump the version number.
allowed-tools: Read, Bash, Grep
argument-hint: [version | patch | minor | major]
---

# Release cubil

A release is a git tag. Pushing `vX.Y.Z` to `origin` runs `release.yml`, which
builds three binaries, creates the GitHub release, updates the Homebrew tap
formula and publishes `@frantufro/cubil` to npm. Everything before the tag is
reversible; the tag push is the point of no return.

## 1. Decide whether there is anything to release

```bash
git log --oneline "$(git describe --tags --abbrev=0)"..main
git diff --stat "$(git describe --tags --abbrev=0)"..main -- src/ Cargo.toml Cargo.lock bin/ claude-plugin/
```

Commits touching only `.github/`, `docs/`, `.cubil/` or `README.md` change
nothing users receive. Dependency bumps in `Cargo.lock` do ship, since the
binaries are rebuilt from the tag.

Pick the bump from what changed:

- `patch` — dependency bumps, bug fixes, refactors with no visible effect
- `minor` — a new command, a new flag, new output a script could read
- `major` — a command or flag removed, renamed, or given different meaning

## 2. Run the script

```bash
script/release patch      # or minor, major, or an exact 1.2.3
```

It writes the version to `Cargo.toml`, `Cargo.lock`, `package.json`,
`claude-plugin/.claude-plugin/plugin.json` and the console sample in
`README.md`, runs `cargo test` and `npm test`, and commits on a
`release/vX.Y.Z` branch. It stops there.

It refuses to run off `main`, on a dirty tree, when `main` and `origin/main`
have diverged, or when the tag already exists. Each refusal names a real
problem — fix the cause and run it again.

`script/release --check` reports whether the five files already agree, which is
the same assertion CI makes against the tag.

## 3. Land the bump

Every change reaches `main` through a pull request.

```bash
git push -u origin release/vX.Y.Z
gh pr create --fill
gh pr checks --watch
gh pr merge --squash --delete-branch
git checkout main && git pull
```

## 4. Push the tag

This publishes. Confirm with the user before running it, every time.

```bash
git tag vX.Y.Z && git push origin vX.Y.Z
```

Then watch the run:

```bash
gh run watch "$(gh run list --workflow release.yml --limit 1 --json databaseId -q '.[0].databaseId')" --exit-status
```

The first job is `verify`. It re-checks the tag against all four version files
and runs the installer suite, and every other job depends on it, so a version
left behind stops the release before anything is built or published.

Confirm the result:

```bash
gh release view vX.Y.Z --json tagName,assets -q '.tagName + " " + ([.assets[].name] | join(", "))'
npm view @frantufro/cubil version
```

Three tarballs and the new npm version mean the release is complete.

## When something fails after the tag is pushed

A failure in `verify` leaves nothing published; delete the tag locally and on
origin, fix the versions, and tag again.

A failure in a later job may leave a partial release. Read the run log before
touching anything, and repair whatever actually published. `npm` refuses to
republish a version, so a broken publish needs the next patch number.

## Facts worth knowing

- npm publishing uses OIDC trusted publishing. The repository holds no npm
  credential, and the trust config names workflow `release.yml`, so renaming
  that file breaks publishing.
- `HOMEBREW_TAP_TOKEN` is the repository's only secret, used to push the
  formula to `frantufro/homebrew-tap`.
- The npm package version has no bearing on which binary
  `npx @frantufro/cubil install` downloads; that always tracks the latest
  GitHub release.
