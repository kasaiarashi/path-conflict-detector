use crate::error::Result;
use std::fs;
use std::path::Path;

pub fn detect_wsl() -> Result<(bool, Option<String>, Option<String>)> {
    // Check if /proc/version exists and contains "Microsoft" or "WSL"
    if let Ok(version_content) = fs::read_to_string("/proc/version") {
        let version_lower = version_content.to_lowercase();
        if version_lower.contains("microsoft") || version_lower.contains("wsl") {
            let wsl_version = detect_wsl_version(&version_content);
            let wsl_distro = std::env::var("WSL_DISTRO_NAME").ok();
            return Ok((true, wsl_version, wsl_distro));
        }
    }

    // Check for WSL_DISTRO_NAME environment variable
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return Ok((true, None, std::env::var("WSL_DISTRO_NAME").ok()));
    }

    // Check if /mnt/c exists (common WSL mount point)
    if Path::new("/mnt/c").exists() && Path::new("/proc/version").exists() {
        return Ok((true, None, None));
    }

    Ok((false, None, None))
}

fn detect_wsl_version(proc_version: &str) -> Option<String> {
    if proc_version.contains("WSL2") {
        Some("WSL2".to_string())
    } else if proc_version.contains("Microsoft") {
        // WSL1 typically has "Microsoft" but not "WSL2"
        Some("WSL1".to_string())
    } else {
        None
    }
}

pub fn is_wsl_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/mnt/") || is_unix_style_path(&path_str)
}

pub fn is_windows_path_in_wsl(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check for /mnt/c/, /mnt/d/, etc.
    if path_str.starts_with("/mnt/") && path_str.len() >= 7 {
        let drive_char = path_str.chars().nth(5);
        let slash_char = path_str.chars().nth(6);
        if let (Some(drive), Some(slash)) = (drive_char, slash_char) {
            return drive.is_ascii_alphabetic() && slash == '/';
        }
    }

    // Check for Windows-style paths with drive letters
    if path_str.len() >= 2 {
        let chars: Vec<char> = path_str.chars().collect();
        if chars.len() >= 3
            && chars[0].is_ascii_alphabetic()
            && (chars[1] == ':' || (chars[1] == '\\' || chars[1] == '/'))
        {
            return true;
        }
    }

    false
}

pub fn is_windows_executable_in_wsl(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        if matches!(ext_lower.as_str(), "exe" | "bat" | "cmd" | "ps1") {
            return true;
        }
    }
    false
}

fn is_unix_style_path(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("/mnt/")
}

pub fn convert_wsl_to_windows_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    // Convert /mnt/c/... to C:\...
    if path_str.starts_with("/mnt/") && path_str.len() > 6 {
        let drive_letter = path_str.chars().nth(5)?;
        if drive_letter.is_ascii_alphabetic() {
            let rest = &path_str[7..]; // Skip "/mnt/c/" to get the rest
            let windows_path = if rest.is_empty() {
                format!("{}:\\", drive_letter.to_uppercase())
            } else {
                format!("{}:\\{}", drive_letter.to_uppercase(), rest.replace('/', "\\"))
            };
            return Some(windows_path);
        }
    }

    None
}

pub fn categorize_wsl_path_mix(path1: &Path, path2: &Path) -> bool {
    let is_path1_windows = is_windows_path_in_wsl(path1);
    let is_path2_windows = is_windows_path_in_wsl(path2);
    let is_path1_wsl = is_wsl_path(path1) && !is_path1_windows;
    let is_path2_wsl = is_wsl_path(path2) && !is_path2_windows;

    (is_path1_windows && is_path2_wsl) || (is_path1_wsl && is_path2_windows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_windows_path_in_wsl() {
        assert!(is_windows_path_in_wsl(Path::new("/mnt/c/Windows")));
        assert!(is_windows_path_in_wsl(Path::new("/mnt/d/Projects")));
        assert!(!is_windows_path_in_wsl(Path::new("/usr/bin")));
        assert!(!is_windows_path_in_wsl(Path::new("/home/user")));
    }

    #[test]
    fn test_convert_wsl_to_windows_path() {
        assert_eq!(
            convert_wsl_to_windows_path(Path::new("/mnt/c/Windows")),
            Some("C:\\Windows".to_string())
        );
        assert_eq!(
            convert_wsl_to_windows_path(Path::new("/mnt/d/Projects/test")),
            Some("D:\\Projects\\test".to_string())
        );
        assert_eq!(convert_wsl_to_windows_path(Path::new("/usr/bin")), None);
    }
}
