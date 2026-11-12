use crate::core::error::{CubilError, Result};

/// Convert a human title into a kebab-case, lowercase, ASCII-only slug.
///
/// Non-alphanumeric ASCII characters (including any non-ASCII) are treated as
/// separators, collapsed into single `-`. Leading and trailing `-` are
/// trimmed. Returns [`CubilError::InvalidSlug`] if the result is empty.
pub fn slugify(title: &str) -> Result<String> {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = true;
    for c in title.chars() {
        let lower = c.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        return Err(CubilError::InvalidSlug);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World").unwrap(), "hello-world");
    }

    #[test]
    fn slugify_collapses_separators_and_trims() {
        assert_eq!(slugify("  Foo --- Bar / Baz!!!").unwrap(), "foo-bar-baz");
    }

    #[test]
    fn slugify_strips_non_ascii() {
        assert_eq!(slugify("Café résumé").unwrap(), "caf-r-sum");
    }

    #[test]
    fn slugify_preserves_digits() {
        assert_eq!(slugify("release v1.2.3").unwrap(), "release-v1-2-3");
    }

    #[test]
    fn slugify_rejects_empty_result() {
        assert!(matches!(
            slugify("!!!").unwrap_err(),
            CubilError::InvalidSlug
        ));
        assert!(matches!(slugify("").unwrap_err(), CubilError::InvalidSlug));
    }
}
