---
status: doing
title: Set up release infrastructure
created: 2026-04-20
---

Add the installation and release infrastructure for cubil: MIT `LICENSE`, a
`curl | sh` `install.sh`, a tag-triggered `.github/workflows/release.yml` that
builds for macOS arm64 and Linux x86_64/arm64, publishes a GitHub Release with
tarballs, and updates the Homebrew formula in `frantufro/homebrew-tap`. Also
write the project `README.md` (pitch, example session, install, command list,
link to Skulk). Pure infra — no Rust code changes.
