---
created: 2026-05-04
---

# Add .gitkeep to cubil status folders

The `.cubil/` directory contains `backlog/`, `doing/`, and `done/` subfolders. When any of these is empty, git won't track it, which means `cubil init` doesn't produce a committable structure for fresh repos.

Add a `.gitkeep` file to each of the three status folders (`backlog/`, `doing/`, `done/`) so they're always present in version control even when empty. Update `cubil init` to create these `.gitkeep` files as part of initialization.
