---
created: 2026-08-29
---

# Run the version check before the release builds anything

`release.yml` asserts that the tag, `Cargo.toml` and `package.json` carry the
same version, and that assertion sits in `publish-npm`, the last job in the
graph. By the time it fires, the GitHub release and the Homebrew formula are
already published, so a version left behind leaves half a release out on the
internet to unpick by hand.

Move the check into a `verify` job at the top of the file and give every other
job a dependency on it. Cover `Cargo.lock` and
`claude-plugin/.claude-plugin/plugin.json` too, since `script/release` writes
both. Move `npm test` up with it, which leaves `publish-npm` with a checkout
and a publish.
