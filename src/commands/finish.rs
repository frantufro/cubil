use crate::core::error::Result;

/// Move task `<slug>` from `doing/` to `done/`.
pub fn run(slug: &str) -> Result<()> {
    super::transition(slug, "doing", "done")
}
