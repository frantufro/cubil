---
created: 2026-04-19
priority: 1
---

# Implement `cubil init` + shared core module

This task is the foundation for every other `cubil` command. In addition to
implementing `init`, it establishes the shared `core` module that the other
six commands (`new`, `list`, `show`, `edit`, `mv`, `rm`) will depend on.

Keep the public API of `core` small and stable — the downstream tasks will be
implemented in parallel against it, so changes after merge will ripple.

## Part 1 — `cubil init`

### Behavior
- Creates `.cubil/backlog/`, `.cubil/doing/`, `.cubil/done/` in the current
  working directory if they don't exist.
- Idempotent: running again is a no-op; does not error.
- Fails loudly if `.cubil/` already exists as a file (not directory).
- Prints the created/confirmed absolute path of `.cubil/` on success.
- Creates `.cubil/` at cwd only — does NOT walk upward (unlike other
  commands). If the user is inside an existing cubil tree, `init` still
  creates a fresh `.cubil/` in cwd.

### Acceptance
- [ ] `cubil init` in an empty directory creates all three status folders.
- [ ] Running twice does not error or duplicate.
- [ ] Creates `.cubil/` at cwd (does not walk upward).
- [ ] Errors clearly if `.cubil` exists as a file.

## Part 2 — Shared `core` module

Create `src/core/` as a module tree. Everything below must be `pub` from
`core` (or its submodules re-exported through `core`) because the other six
command implementations will import from here.

### Module layout

```
src/
  main.rs           # clap wiring + dispatch (keep thin)
  commands/         # one file per command; this task only adds init.rs
    mod.rs
    init.rs
  core/
    mod.rs
    root.rs         # find_root()
    slug.rs         # slugify(), resolve_slug()
    frontmatter.rs  # parse_frontmatter(), TaskMeta
    error.rs        # CubilError + Result alias
```

`commands/mod.rs` re-exports each command's entry point (`pub fn run(...)`).
`main.rs` should only parse `Cli` and dispatch to `commands::<name>::run`.
Downstream agents will add `commands/new.rs`, `commands/list.rs`, etc.,
following the same pattern — do NOT stub them here; each is its own task.

### `core::root`

```rust
/// Walk upward from `start` (or cwd if None) looking for a `.cubil/` dir.
/// Returns the absolute path to the `.cubil/` directory itself.
/// Errors with `CubilError::RootNotFound` if no `.cubil/` is found before
/// filesystem root.
pub fn find_root(start: Option<&Path>) -> Result<PathBuf>;
```

### `core::slug`

```rust
/// Convert a human title into a kebab-case, lowercase, ASCII-only slug.
/// - Unicode → ASCII via best-effort transliteration (a simple lowercase +
///   strip-non-ascii-alnum is fine; do NOT pull in heavy deps).
/// - Collapses runs of non-alphanumeric chars into single `-`.
/// - Trims leading/trailing `-`.
/// - Returns `CubilError::InvalidSlug` if the result is empty.
pub fn slugify(title: &str) -> Result<String>;

/// Locate a task file by exact slug across every status folder under
/// `.cubil/`. Returns (status_name, absolute_path_to_md_file).
/// Errors with `CubilError::SlugNotFound` if no match.
/// Errors with `CubilError::SlugAmbiguous` if the same slug exists in
/// multiple status folders (shouldn't happen, but surface it clearly).
pub fn resolve_slug(root: &Path, slug: &str) -> Result<(String, PathBuf)>;

/// List every (status, slug, absolute_path) triple under `.cubil/`.
/// Used by `list` and by `new` to check for collisions.
pub fn scan_all(root: &Path) -> Result<Vec<TaskEntry>>;

pub struct TaskEntry {
    pub status: String,
    pub slug: String,
    pub path: PathBuf,
}
```

### `core::frontmatter`

```rust
pub struct TaskMeta {
    pub created: Option<String>,   // raw ISO date string
    pub priority: Option<u32>,
    // Unknown keys are preserved as raw lines for round-tripping.
    pub extra: Vec<String>,
}

/// Parse a `---`-delimited YAML-ish frontmatter block from the start of a
/// markdown file. Only recognizes `key: value` lines; does NOT pull in
/// serde_yaml. Unknown keys go into `extra`.
/// Returns (TaskMeta, rest_of_file_starting_at_body).
/// If the file has no frontmatter, returns (default TaskMeta, full input).
pub fn parse_frontmatter(src: &str) -> (TaskMeta, &str);

/// Render a TaskMeta back to a `---` block (trailing newline included).
pub fn render_frontmatter(meta: &TaskMeta) -> String;
```

Keep this hand-rolled — no new dependencies. The frontmatter is always
simple `key: value` lines in this project.

### `core::error`

```rust
pub type Result<T> = std::result::Result<T, CubilError>;

#[derive(Debug)]
pub enum CubilError {
    RootNotFound,
    RootIsFile(PathBuf),
    SlugNotFound(String),
    SlugAmbiguous { slug: String, statuses: Vec<String> },
    SlugCollision { slug: String, status: String },
    InvalidSlug,
    StatusMissing(String),
    Io(std::io::Error),
    Other(String),
}
```

- Implement `std::fmt::Display` with terse, user-readable messages
  (printed to stderr by `main.rs`).
- Implement `From<std::io::Error>` so `?` works on IO calls.
- Implement `std::error::Error`.
- Each variant should map to a non-zero exit. `main.rs` should print the
  `Display` form to stderr and `std::process::exit(1)`.

## Testing

- Unit tests in each `core/*.rs` file (same-file `#[cfg(test)] mod tests`).
  Cover at minimum: `slugify` normalization, `find_root` walk-up, frontmatter
  round-trip, `resolve_slug` across multiple statuses.
- An integration test under `tests/init.rs` that runs `cubil init` in a
  tempdir and asserts the three folders exist. Use `assert_cmd` + `tempfile`
  — add them as `[dev-dependencies]` in `Cargo.toml`.

## Out of scope

- Do NOT implement `new`, `list`, `show`, `edit`, `mv`, or `rm` — they are
  separate tasks that will run in parallel after this one merges.
- Do NOT add `start` / `finish` sugar commands — also separate tasks.
- Do NOT add `serde`, `serde_yaml`, `chrono`, or `slug` crate dependencies.
  Hand-roll it; the scope is small.

## Workflow for this agent

1. Implement on the branch skulk put you on (you're already in a worktree).
2. Before opening a PR, all of these must pass:
   - `cargo build`
   - `cargo test`
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
3. Commit with a clear message, push the branch, and open a PR — either via
   `gh pr create` directly or by leaving the session idle for the human to
   `skulk ship`. Either is fine.
4. The PR description should spell out the public API of `core` (signatures
   of `find_root`, `slugify`, `resolve_slug`, `scan_all`, the frontmatter
   functions, and `CubilError` variants) since six downstream tasks will be
   implemented in parallel against it.
