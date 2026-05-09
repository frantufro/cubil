use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_GITHUB_API_BASE: &str = "https://api.github.com";
const DEFAULT_DOWNLOAD_BASE: &str = "https://github.com";
const REPO: &str = "frantufro/cubil";
const BIN_NAME: &str = "cubil";
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;

pub const STALE_CHECK_TIMEOUT: Duration = Duration::from_millis(500);
pub const UPDATE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CacheEntry {
    pub checked_at: u64,
    pub latest: String,
}

#[derive(Debug)]
pub struct Target {
    pub triple: String,
}

#[derive(Debug)]
pub enum TargetError {
    UnsupportedOs(String),
    UnsupportedArch(String),
    MacosX86,
}

impl std::fmt::Display for TargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetError::UnsupportedOs(s) => write!(f, "Unsupported OS: {s}"),
            TargetError::UnsupportedArch(s) => write!(f, "Unsupported architecture: {s}"),
            TargetError::MacosX86 => write!(
                f,
                "x86_64 macOS is not supported. Use an Apple Silicon Mac or build from source."
            ),
        }
    }
}

pub fn detect_target() -> Result<Target, TargetError> {
    if let Ok(t) = env::var("CUBIL_TARGET_OVERRIDE") {
        return target_from_override(&t);
    }
    target_from_parts(env::consts::ARCH, env::consts::OS)
}

fn target_from_parts(arch: &str, os: &str) -> Result<Target, TargetError> {
    let os_name = match os {
        "linux" => "unknown-linux-gnu",
        "macos" => "apple-darwin",
        other => return Err(TargetError::UnsupportedOs(other.to_string())),
    };
    let arch_name = match (arch, os) {
        ("x86_64", "macos") => return Err(TargetError::MacosX86),
        ("x86_64", _) => "x86_64",
        ("aarch64" | "arm64", _) => "aarch64",
        (other, _) => return Err(TargetError::UnsupportedArch(other.to_string())),
    };
    Ok(Target {
        triple: format!("{arch_name}-{os_name}"),
    })
}

fn target_from_override(s: &str) -> Result<Target, TargetError> {
    // Test/debug hook: parse "<arch>:<os>" and route through normal logic so
    // unsupported-target paths can be exercised on any host.
    if let Some((arch, os)) = s.split_once(':') {
        return target_from_parts(arch, os);
    }
    Ok(Target {
        triple: s.to_string(),
    })
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let core = s.split('-').next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

fn cache_path() -> Option<PathBuf> {
    if let Ok(dir) = env::var("CUBIL_CACHE_DIR") {
        return Some(PathBuf::from(dir).join("latest.json"));
    }
    let base = if let Ok(xdg) = env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg)
    } else {
        PathBuf::from(env::var("HOME").ok()?).join(".cache")
    };
    Some(base.join("cubil").join("latest.json"))
}

pub fn read_cache() -> Option<CacheEntry> {
    let path = cache_path()?;
    let bytes = fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn write_cache(latest: &str) -> io::Result<()> {
    let Some(path) = cache_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry = CacheEntry {
        checked_at: now,
        latest: latest.to_string(),
    };
    let bytes = serde_json::to_vec(&entry)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, bytes)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn cache_is_fresh(entry: &CacheEntry) -> bool {
    now_secs().saturating_sub(entry.checked_at) < CACHE_TTL_SECS
}

fn github_api_base() -> String {
    env::var("CUBIL_GITHUB_API_BASE").unwrap_or_else(|_| DEFAULT_GITHUB_API_BASE.into())
}

fn download_base() -> String {
    env::var("CUBIL_DOWNLOAD_BASE").unwrap_or_else(|_| DEFAULT_DOWNLOAD_BASE.into())
}

pub fn fetch_latest_version(timeout: Duration) -> Result<String, String> {
    let url = format!("{}/repos/{}/releases/latest", github_api_base(), REPO);
    let agent = ureq::AgentBuilder::new()
        .timeout(timeout)
        .user_agent(&format!("cubil/{}", current_version()))
        .build();
    let resp = agent
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("network error: {e}"))?;
    let body = resp
        .into_string()
        .map_err(|e| format!("invalid response: {e}"))?;
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("invalid JSON: {e}"))?;
    let tag = json
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "missing tag_name in API response".to_string())?;
    Ok(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// Best-effort: never errors. Returns Some(warning) only when we know the
/// running binary is older than the latest release.
pub fn stale_warning() -> Option<String> {
    if env::var_os("CUBIL_NO_UPDATE_CHECK").is_some() {
        return None;
    }
    let latest = match read_cache() {
        Some(entry) if cache_is_fresh(&entry) => entry.latest,
        _ => match fetch_latest_version(STALE_CHECK_TIMEOUT) {
            Ok(v) => {
                let _ = write_cache(&v);
                v
            }
            Err(_) => return None,
        },
    };
    let current = current_version();
    if is_newer(&latest, current) {
        Some(format!(
            "warning: cubil {current} is out of date (latest: {latest}). Run `cubil update` to upgrade."
        ))
    } else {
        None
    }
}

pub fn download_tarball(
    target: &Target,
    version: &str,
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    let url = format!(
        "{}/{}/releases/download/v{}/{}-{}.tar.gz",
        download_base(),
        REPO,
        version,
        BIN_NAME,
        target.triple
    );
    let agent = ureq::AgentBuilder::new()
        .timeout(timeout)
        .user_agent(&format!("cubil/{}", current_version()))
        .build();
    let resp = agent
        .get(&url)
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let mut buf = Vec::new();
    // 50 MB cap — release binary is ~5 MB; this is a safety net against a
    // misconfigured download URL serving something pathological.
    resp.into_reader()
        .take(50_000_000)
        .read_to_end(&mut buf)
        .map_err(|e| format!("read failed: {e}"))?;
    Ok(buf)
}

pub fn extract_binary(tarball: &[u8]) -> Result<Vec<u8>, String> {
    let gz = flate2::read::GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(gz);
    let entries = archive.entries().map_err(|e| format!("tar error: {e}"))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("tar entry error: {e}"))?;
        let path = entry.path().map_err(|e| format!("tar path error: {e}"))?;
        if path.file_name() == Some(std::ffi::OsStr::new(BIN_NAME)) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("tar read error: {e}"))?;
            return Ok(buf);
        }
    }
    Err(format!("'{BIN_NAME}' not found in tarball"))
}

pub fn atomic_replace(target: &Path, new_bytes: &[u8]) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "target binary has no parent directory".to_string())?;
    let tmp = parent.join(format!(".{}.update.{}.tmp", BIN_NAME, std::process::id()));
    fs::write(&tmp, new_bytes).map_err(|e| format!("write temp failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755)) {
            let _ = fs::remove_file(&tmp);
            return Err(format!("chmod failed: {e}"));
        }
    }
    if let Err(e) = fs::rename(&tmp, target) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("replace failed: {e}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_strips_v_and_prerelease() {
        assert_eq!(parse_version("0.1.2"), Some((0, 1, 2)));
        assert_eq!(parse_version("v0.1.2"), Some((0, 1, 2)));
        assert_eq!(parse_version("v1.2.3-rc.1"), Some((1, 2, 3)));
        assert_eq!(parse_version("garbage"), None);
        assert_eq!(parse_version("1.2"), None);
    }

    #[test]
    fn is_newer_compares_semver_tuples() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("1.0.0", "0.99.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(!is_newer("garbage", "0.1.0"));
    }

    #[test]
    fn target_macos_x86_is_unsupported() {
        let err = target_from_parts("x86_64", "macos").unwrap_err();
        assert!(matches!(err, TargetError::MacosX86));
        assert!(err.to_string().contains("x86_64 macOS is not supported"));
    }

    #[test]
    fn target_linux_x86_64() {
        let t = target_from_parts("x86_64", "linux").unwrap();
        assert_eq!(t.triple, "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn target_linux_aarch64() {
        let t = target_from_parts("aarch64", "linux").unwrap();
        assert_eq!(t.triple, "aarch64-unknown-linux-gnu");
    }

    #[test]
    fn target_macos_aarch64() {
        let t = target_from_parts("aarch64", "macos").unwrap();
        assert_eq!(t.triple, "aarch64-apple-darwin");
    }
}
