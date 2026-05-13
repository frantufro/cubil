use crate::core::error::{CubilError, Result};
use crate::core::updater::{self, UPDATE_TIMEOUT};

/// Replace the running `cubil` binary with the latest GitHub release.
///
/// Resolves the install location via [`std::env::current_exe`], so the
/// binary is replaced in-place wherever it lives. Errors out cleanly on
/// unsupported targets, network failures, and write-permission failures.
pub fn run() -> Result<()> {
    let target = updater::detect_target().map_err(|e| CubilError::Update(e.to_string()))?;
    let latest = updater::fetch_latest_version(UPDATE_TIMEOUT)
        .map_err(CubilError::Update)?;
    let current = updater::current_version();

    let _ = updater::write_cache(&latest);

    if !updater::is_newer(&latest, current) {
        println!("cubil {current} is already up to date.");
        return Ok(());
    }

    let exe = std::env::current_exe()
        .map_err(|e| CubilError::Update(format!("could not resolve current exe: {e}")))?;

    println!("Downloading cubil {latest} for {}...", target.triple);
    let tarball = updater::download_tarball(&target, &latest, UPDATE_TIMEOUT)
        .map_err(CubilError::Update)?;
    let bin = updater::extract_binary(&tarball).map_err(CubilError::Update)?;
    updater::atomic_replace(&exe, &bin).map_err(CubilError::Update)?;

    println!("Updated cubil to {latest}");
    Ok(())
}
