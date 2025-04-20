use std::env;
use std::path::PathBuf;

/// Returns a list of possible paths where REPO game save data might be stored
/// based on the current platform.
///
/// This function returns different paths depending on the operating system:
/// - Windows: Checks User profile folders and standard game save locations
/// - macOS: Checks Application Support and other common save locations
/// - Linux: Checks .local/share and other XDG directories
///
/// # Returns
/// A vector of PathBuf objects representing potential save directory locations
pub fn get_repo_save_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    return get_windows_repo_save_paths();

    #[cfg(target_os = "macos")]
    return get_macos_repo_save_paths();

    #[cfg(target_os = "linux")]
    return get_linux_repo_save_paths();

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        log::warn!("Unsupported platform for REPO save detection");
        return Vec::new();
    }
}

#[cfg(target_os = "windows")]
fn get_windows_repo_save_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Try to get user profile directory
    if let Ok(user_profile) = env::var("USERPROFILE") {
        let user_profile = PathBuf::from(user_profile);

        // Common Windows save locations
        paths.push(user_profile.join("AppData\\LocalLow\\smiwork\\REPO\\saves"));
    }

    paths
}

#[cfg(target_os = "macos")]
fn get_macos_repo_save_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Try to get home directory
    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);

        // Crossover save location
        paths.push(home.join("Library/Application Support/CrossOver/Bottles/Steam/drive_c/users/crossover/AppData/LocalLow/semiwork/Repo/saves"))
    }

    paths
}

#[cfg(target_os = "linux")]
fn get_linux_repo_save_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Try to get home directory
    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);

        // Steam Proton paths
        paths.push(home.join(".steam/debian-installation/steamapps/compatdata/3241660/pfx/drive_c/users/steamuser/AppData/LocalLow/semiwork/Repo/saves/"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_repo_save_paths {
        use super::*;

        #[test]
        fn test_get_repo_save_paths_returns_paths() {
            let paths = get_repo_save_paths();
            // Just verify we get some paths, regardless of platform
            assert!(
                !paths.is_empty(),
                "Should return at least one potential save path"
            );

            // Check that all returned paths are absolute
            for path in &paths {
                assert!(path.is_absolute(), "All returned paths should be absolute");
            }
        }
    }
}
