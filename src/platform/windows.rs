use std::path::Path;

pub fn is_executable_windows(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    // On Windows, check for executable extensions
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(ext_lower.as_str(), "exe" | "bat" | "cmd" | "ps1" | "com")
    } else {
        false
    }
}

pub fn expand_windows_env_vars(path: &str) -> String {
    let mut result = path.to_string();

    // Expand %VAR% style environment variables
    if result.contains('%') {
        let parts: Vec<&str> = result.split('%').collect();
        if parts.len() >= 3 {
            let mut expanded = String::new();
            let mut in_var = false;

            for part in parts {
                if in_var {
                    if let Ok(value) = std::env::var(part) {
                        expanded.push_str(&value);
                    } else {
                        expanded.push('%');
                        expanded.push_str(part);
                        expanded.push('%');
                    }
                    in_var = false;
                } else {
                    expanded.push_str(part);
                    in_var = true;
                }
            }

            result = expanded;
        }
    }

    result
}

pub fn is_windows_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("windows\\system32")
        || path_str.contains("windows\\system")
        || path_str.contains("program files")
        || path_str.contains("programdata")
}

#[cfg(windows)]
pub fn get_file_version_windows(_path: &Path) -> Option<String> {
    // TODO: Implement Windows file version extraction using winapi
    // This requires proper parsing of PE file version info
    // For now, return None and rely on command execution for version detection
    None
}

#[cfg(not(windows))]
pub fn get_file_version_windows(_path: &Path) -> Option<String> {
    None
}
