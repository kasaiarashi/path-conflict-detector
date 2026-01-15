use crate::output::types::{ExecutableInfo, ManagerInfo, ManagerType};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

struct ManagerPattern {
    manager_type: ManagerType,
    name: &'static str,
    description: &'static str,
    path_patterns: Vec<&'static str>,
}

static MANAGER_PATTERNS: Lazy<Vec<ManagerPattern>> = Lazy::new(|| {
    vec![
        // Version Managers
        ManagerPattern {
            manager_type: ManagerType::VersionManager,
            name: "nvm",
            description: "Node Version Manager",
            path_patterns: vec![r"\.nvm/", r"/nvm/"],
        },
        ManagerPattern {
            manager_type: ManagerType::VersionManager,
            name: "pyenv",
            description: "Python Version Manager",
            path_patterns: vec![r"\.pyenv/", r"/pyenv/"],
        },
        ManagerPattern {
            manager_type: ManagerType::VersionManager,
            name: "rbenv",
            description: "Ruby Version Manager",
            path_patterns: vec![r"\.rbenv/", r"/rbenv/"],
        },
        ManagerPattern {
            manager_type: ManagerType::VersionManager,
            name: "rustup",
            description: "Rust Toolchain Manager",
            path_patterns: vec![r"\.cargo/bin", r"\.rustup/"],
        },
        ManagerPattern {
            manager_type: ManagerType::VersionManager,
            name: "asdf",
            description: "Multiple Runtime Version Manager",
            path_patterns: vec![r"\.asdf/"],
        },
        ManagerPattern {
            manager_type: ManagerType::VersionManager,
            name: "sdkman",
            description: "Software Development Kit Manager",
            path_patterns: vec![r"\.sdkman/"],
        },
        // Package Managers
        ManagerPattern {
            manager_type: ManagerType::PackageManager,
            name: "Homebrew",
            description: "Package Manager for macOS",
            path_patterns: vec![r"/opt/homebrew/", r"/usr/local/Cellar/", r"Homebrew/"],
        },
        ManagerPattern {
            manager_type: ManagerType::PackageManager,
            name: "Chocolatey",
            description: "Package Manager for Windows",
            path_patterns: vec![r"chocolatey/", r"\\chocolatey\\"],
        },
        ManagerPattern {
            manager_type: ManagerType::PackageManager,
            name: "Scoop",
            description: "Package Manager for Windows",
            path_patterns: vec![r"\\scoop\\", r"/scoop/"],
        },
        // System paths
        ManagerPattern {
            manager_type: ManagerType::SystemInstall,
            name: "System",
            description: "System Installation",
            path_patterns: vec![
                r"^/usr/bin",
                r"^/usr/local/bin",
                r"^/bin",
                r"^/sbin",
                r"^C:\\Windows\\",
                r"^C:\\Program Files\\",
                r"^/System/",
            ],
        },
    ]
});

pub struct ManagerDetector {}

impl ManagerDetector {
    pub fn new() -> Self {
        ManagerDetector {}
    }

    pub fn detect_managers(&self, executables: &mut [ExecutableInfo]) {
        for executable in executables.iter_mut() {
            executable.manager = self.detect(&executable.resolved_path);
        }
    }

    pub fn detect(&self, path: &Path) -> Option<ManagerInfo> {
        let path_str = path.to_string_lossy();

        // Check each pattern
        for pattern in MANAGER_PATTERNS.iter() {
            for path_pattern in &pattern.path_patterns {
                if let Ok(regex) = Regex::new(path_pattern) {
                    if regex.is_match(&path_str) {
                        return Some(ManagerInfo {
                            manager_type: pattern.manager_type,
                            name: pattern.name.to_string(),
                            description: pattern.description.to_string(),
                        });
                    }
                }
            }
        }

        // Check environment variables for additional hints
        if self.check_env_vars(path) {
            // Already handled by patterns, this is a fallback
        }

        // If no specific manager detected, determine if it's manual install
        if !path_str.contains("usr/") && !path_str.contains("Windows") {
            return Some(ManagerInfo {
                manager_type: ManagerType::ManualInstall,
                name: "Manual".to_string(),
                description: "Manually Installed".to_string(),
            });
        }

        None
    }

    fn check_env_vars(&self, _path: &Path) -> bool {
        // Check for manager-specific environment variables
        if std::env::var("NVM_DIR").is_ok() {
            return true;
        }
        if std::env::var("PYENV_ROOT").is_ok() {
            return true;
        }
        if std::env::var("RBENV_ROOT").is_ok() {
            return true;
        }
        if std::env::var("RUSTUP_HOME").is_ok() || std::env::var("CARGO_HOME").is_ok() {
            return true;
        }
        if std::env::var("HOMEBREW_PREFIX").is_ok() {
            return true;
        }

        false
    }
}

impl Default for ManagerDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_nvm() {
        let detector = ManagerDetector::new();
        let path = PathBuf::from("/home/user/.nvm/versions/node/v18.0.0/bin/node");
        let result = detector.detect(&path);

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.name, "nvm");
        assert_eq!(info.manager_type, ManagerType::VersionManager);
    }

    #[test]
    fn test_detect_system() {
        let detector = ManagerDetector::new();
        let path = PathBuf::from("/usr/bin/python");
        let result = detector.detect(&path);

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.name, "System");
        assert_eq!(info.manager_type, ManagerType::SystemInstall);
    }

    #[test]
    fn test_detect_homebrew() {
        let detector = ManagerDetector::new();
        let path = PathBuf::from("/opt/homebrew/bin/python3");
        let result = detector.detect(&path);

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.name, "Homebrew");
        assert_eq!(info.manager_type, ManagerType::PackageManager);
    }
}
