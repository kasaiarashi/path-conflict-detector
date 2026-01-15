use std::path::{Path, PathBuf};

pub fn detect_homebrew_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Check for Homebrew on Apple Silicon (ARM)
    if Path::new("/opt/homebrew").exists() {
        paths.push(PathBuf::from("/opt/homebrew/bin"));
        paths.push(PathBuf::from("/opt/homebrew/sbin"));
    }

    // Check for Homebrew on Intel
    if Path::new("/usr/local/Cellar").exists() {
        paths.push(PathBuf::from("/usr/local/bin"));
        paths.push(PathBuf::from("/usr/local/sbin"));
    }

    // Check HOMEBREW_PREFIX environment variable
    if let Ok(prefix) = std::env::var("HOMEBREW_PREFIX") {
        let bin_path = PathBuf::from(&prefix).join("bin");
        let sbin_path = PathBuf::from(&prefix).join("sbin");
        if !paths.contains(&bin_path) {
            paths.push(bin_path);
        }
        if !paths.contains(&sbin_path) {
            paths.push(sbin_path);
        }
    }

    paths
}

pub fn is_homebrew_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/opt/homebrew/")
        || path_str.starts_with("/usr/local/Cellar/")
        || path_str.contains("/Homebrew/")
}

pub fn get_macos_bundle_version(_path: &Path) -> Option<String> {
    // TODO: Implement parsing of Info.plist for .app bundles
    // This would require plist parsing library
    None
}

pub fn is_macos_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/usr/bin")
        || path_str.starts_with("/bin")
        || path_str.starts_with("/sbin")
        || path_str.starts_with("/usr/sbin")
        || path_str.starts_with("/System/")
        || path_str.starts_with("/Library/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_homebrew_path() {
        assert!(is_homebrew_path(Path::new("/opt/homebrew/bin/python3")));
        assert!(is_homebrew_path(Path::new(
            "/usr/local/Cellar/node/18.0.0/bin/node"
        )));
        assert!(!is_homebrew_path(Path::new("/usr/bin/python")));
    }

    #[test]
    fn test_is_macos_system_path() {
        assert!(is_macos_system_path(Path::new("/usr/bin/python")));
        assert!(is_macos_system_path(Path::new(
            "/System/Library/Frameworks"
        )));
        assert!(!is_macos_system_path(Path::new("/opt/homebrew/bin/python")));
    }
}
