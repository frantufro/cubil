use crate::core::error::Result;

/// Move task `<slug>` from `backlog/` to `doing/`.
pub fn run(slug: &str) -> Result<()> {
    super::transition(slug, "backlog", "doing")
}
