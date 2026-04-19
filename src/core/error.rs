use std::fmt;
use std::path::PathBuf;

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
            CubilError::InvalidSlug => write!(f, "title produced an empty slug"),
            CubilError::StatusMissing(s) => write!(f, "status folder missing: {s}"),
            CubilError::Io(e) => write!(f, "io error: {e}"),
            CubilError::Other(s) => write!(f, "{s}"),
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
