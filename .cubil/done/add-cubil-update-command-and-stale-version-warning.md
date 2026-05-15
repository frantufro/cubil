---
created: 2026-05-05
---

# Add cubil update command and stale-version warning

Add an `update` subcommand that upgrades the installed `cubil` binary to the latest GitHub release, plus a stale-version warning that surfaces on other commands when the running binary isn't on the latest release.

## `cubil update`

- Fetches the latest release from `https://github.com/frantufro/cubil/releases/latest` (or the GitHub API equivalent).
- If the running binary is already on the latest version, prints something like `cubil X.Y.Z is already up to date.` and exits 0.
- Otherwise, downloads the platform-appropriate tarball (mirroring `install.sh`'s OS/arch detection) and replaces the running binary in place.
- Install location: same path as the currently-running binary (resolve via `std::env::current_exe()`). Don't hardcode `~/.local/bin` — respect wherever cubil was installed.
- On macOS x86_64 (unsupported), error out with the same message `install.sh` uses.
- After successful install, print the new version.
- Exit non-zero on any failure (network error, untrusted target, write permission, etc.) with a clear message.

## Stale-version warning

- On any non-`update` command, check whether the running binary matches the latest release.
- If stale, print a one-line warning to stderr **before** the command's normal output, e.g.:
  `warning: cubil 0.1.2 is out of date (latest: 0.2.0). Run \`cubil update\` to upgrade.`
- The check must not block the command. If the network is slow or unreachable, the command runs normally with no warning.
- Cache the latest-version lookup on disk (e.g. `~/.cache/cubil/latest.json` or `$XDG_CACHE_HOME/cubil/latest.json`) with a TTL of 24h so we don't hit GitHub on every invocation.
- Respect `CUBIL_NO_UPDATE_CHECK=1` to disable the check entirely (for CI, offline use, etc.).

## Tests

- `cubil update` happy path: mock the GitHub API + tarball, verify binary swap.
- `cubil update` on already-latest: prints up-to-date message, no download.
- `cubil update` on unsupported target: clear error.
- Stale-warning prints on stderr when cache says newer version exists.
- No warning when versions match.
- No warning with `CUBIL_NO_UPDATE_CHECK=1`.
- Cache TTL respected — second invocation within 24h doesn't refetch.
- Network failure during stale check → command runs normally, no warning, no error.

## Out of scope

- Auto-update without prompting.
- Downgrade support.
- Channel selection (stable/nightly/etc.).
- Self-update for non-binary installs (e.g., Homebrew users — they should `brew upgrade`).
