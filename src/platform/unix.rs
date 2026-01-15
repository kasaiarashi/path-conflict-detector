use std::path::Path;

pub fn is_executable_unix(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let permissions = metadata.permissions();
            // Check if any execute bit is set (user, group, or other)
            return permissions.mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(not(unix))]
    {
        // Fallback for non-Unix systems
        path.is_file()
    }
}

pub fn expand_unix_env_vars(path: &str) -> String {
    let mut result = path.to_string();

    // Expand $VAR and ${VAR} style environment variables
    if result.contains('$') {
        // Handle ${VAR} first
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];
                if let Ok(value) = std::env::var(var_name) {
                    result.replace_range(start..start + end + 1, &value);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Handle $VAR
        let parts: Vec<&str> = result.split('$').collect();
        if parts.len() > 1 {
            let mut expanded = parts[0].to_string();
            for part in &parts[1..] {
                // Extract variable name (alphanumeric and underscore)
                let var_name: String = part
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();

                if !var_name.is_empty() {
                    if let Ok(value) = std::env::var(&var_name) {
                        expanded.push_str(&value);
                        expanded.push_str(&part[var_name.len()..]);
                    } else {
                        expanded.push('$');
                        expanded.push_str(part);
                    }
                } else {
                    expanded.push('$');
                    expanded.push_str(part);
                }
            }
            result = expanded;
        }
    }

    // Expand tilde (~) for home directory
    if result.starts_with("~/") || result == "~" {
        if let Ok(home) = std::env::var("HOME") {
            result = result.replacen('~', &home, 1);
        }
    }

    result
}

pub fn is_system_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.starts_with("/usr/bin")
        || path_str.starts_with("/usr/local/bin")
        || path_str.starts_with("/bin")
        || path_str.starts_with("/sbin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_unix_env_vars() {
        std::env::set_var("TEST_VAR", "test_value");
        assert_eq!(expand_unix_env_vars("$TEST_VAR/bin"), "test_value/bin");
        assert_eq!(
            expand_unix_env_vars("${TEST_VAR}/bin"),
            "test_value/bin"
        );
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_is_system_path() {
        assert!(is_system_path(Path::new("/usr/bin/python")));
        assert!(is_system_path(Path::new("/usr/local/bin/node")));
        assert!(!is_system_path(Path::new("/home/user/.nvm/bin/node")));
    }
}
