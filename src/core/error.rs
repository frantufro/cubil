use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, CubilError>;

#[derive(Debug)]
pub enum CubilError {
    RootNotFound,
    RootIsFile(PathBuf),
    SlugNotFound(String),
    SlugAmbiguous {
        slug: String,
        statuses: Vec<String>,
    },
    SlugCollision {
        slug: String,
        status: String,
    },
    StatusMismatch {
        slug: String,
        expected: String,
        actual: String,
    },
    InvalidSlug,
    StatusMissing(String),
    RoadmapNotFound(String),
    RoadmapExists(String),
    MilestoneNotFound {
        roadmap: String,
        milestone: String,
        available: Vec<String>,
    },
    MilestoneAmbiguous {
        roadmap: String,
        milestone: String,
    },
    TaskAlreadyInRoadmap {
        roadmap: String,
        task: String,
    },
    Io(std::io::Error),
    Update(String),
}

impl fmt::Display for CubilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CubilError::RootNotFound => {
                write!(f, "no .cubil/ directory found (run `cubil init` first)")
            }
            CubilError::RootIsFile(p) => {
                write!(
                    f,
                    ".cubil exists as a file, not a directory: {}",
                    p.display()
                )
            }
            CubilError::SlugNotFound(s) => write!(f, "task not found: {s}"),
            CubilError::SlugAmbiguous { slug, statuses } => write!(
                f,
                "slug `{slug}` exists in multiple statuses: {}",
                statuses.join(", ")
            ),
            CubilError::SlugCollision { slug, status } => {
                write!(f, "slug `{slug}` already exists in {status}/")
            }
            CubilError::StatusMismatch {
                slug,
                expected,
                actual,
            } => write!(f, "task `{slug}` is in `{actual}`, not `{expected}`"),
            CubilError::InvalidSlug => write!(f, "title produced an empty slug"),
            CubilError::StatusMissing(s) => write!(f, "status folder missing: {s}"),
            CubilError::RoadmapNotFound(s) => write!(f, "roadmap not found: {s}"),
            CubilError::RoadmapExists(s) => write!(f, "roadmap already exists: {s}"),
            CubilError::MilestoneNotFound {
                roadmap,
                milestone,
                available,
            } => {
                if available.is_empty() {
                    write!(
                        f,
                        "roadmap `{roadmap}` has no milestone `{milestone}` (no milestones defined)"
                    )
                } else {
                    write!(
                        f,
                        "roadmap `{roadmap}` has no milestone `{milestone}` (available: {})",
                        available.join(", ")
                    )
                }
            }
            CubilError::MilestoneAmbiguous { roadmap, milestone } => write!(
                f,
                "roadmap `{roadmap}` has multiple milestones named `{milestone}`"
            ),
            CubilError::TaskAlreadyInRoadmap { roadmap, task } => write!(
                f,
                "task `{task}` is already in roadmap `{roadmap}`"
            ),
            CubilError::Io(e) => write!(f, "io error: {e}"),
            CubilError::Update(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CubilError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CubilError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CubilError {
    fn from(e: std::io::Error) -> Self {
        CubilError::Io(e)
    }
}
