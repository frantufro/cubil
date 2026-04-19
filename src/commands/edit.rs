use std::process::Command;

use crate::core::error::Result;
use crate::core::root::find_root;
use crate::core::slug::resolve_slug;

/// Open the task identified by `slug` in the user's editor.
///
/// Locates `.cubil/` by walking upward from the current working directory,
/// resolves the slug to a path, then launches `$EDITOR` with the path as a
/// final argument. Falls back to `vi` if `$EDITOR` is unset or empty.
///
/// `$EDITOR` is split on ASCII whitespace; the first token is the program
/// and the rest become leading arguments. We do **not** invoke a shell, so
/// quoting, globbing, and `$VAR` expansion in `$EDITOR` are not supported.
/// Values like `code --wait` or `vim -O` work; values that rely on shell
/// features (e.g. `EDITOR='emacs "$@"'`) do not.
///
/// Blocks until the editor exits. We can't reliably distinguish a clean
/// `:wq` from a deliberate `:cq`, so any successful spawn-and-wait is
/// treated as success regardless of the editor's exit code.
pub fn run(slug: String) -> Result<()> {
    let root = find_root(None)?;
    let (_status, path) = resolve_slug(&root, &slug)?;

    let editor = std::env::var("EDITOR").unwrap_or_default();
    let mut parts = editor.split_ascii_whitespace();
    let program = parts.next().unwrap_or("vi");
    let leading_args: Vec<&str> = parts.collect();

    Command::new(program)
        .args(&leading_args)
        .arg(&path)
        .status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn split_editor_handles_args() {
        let editor = "code --wait";
        let mut parts = editor.split_ascii_whitespace();
        assert_eq!(parts.next(), Some("code"));
        let rest: Vec<&str> = parts.collect();
        assert_eq!(rest, vec!["--wait"]);
    }

    #[test]
    fn split_editor_falls_back_when_empty() {
        let mut parts = "".split_ascii_whitespace();
        assert_eq!(parts.next().unwrap_or("vi"), "vi");
    }

    #[test]
    fn split_editor_falls_back_when_whitespace_only() {
        let mut parts = "   ".split_ascii_whitespace();
        assert_eq!(parts.next().unwrap_or("vi"), "vi");
    }
}
