use crate::output::types::{ConflictCategory, ExecutableInfo, ManagerType, PlatformInfo, Severity};
use crate::platform::wsl;

pub struct ConflictCategorizer {
    platform: PlatformInfo,
}

impl ConflictCategorizer {
    pub fn new(platform: PlatformInfo) -> Self {
        ConflictCategorizer { platform }
    }

    pub fn categorize(&self, _binary_name: &str, instances: &[ExecutableInfo]) -> ConflictCategory {
        // Check for WSL vs Windows conflicts (only on WSL)
        if self.platform.is_wsl && self.is_wsl_vs_windows_conflict(instances) {
            return ConflictCategory::WslVsWindows;
        }

        // Check for multiple version managers
        if self.is_multiple_version_managers_conflict(instances) {
            return ConflictCategory::MultipleVersionManagers;
        }

        // Check for version manager vs system
        if self.is_version_manager_vs_system_conflict(instances) {
            return ConflictCategory::VersionManagerVsSystem;
        }

        // Check for package manager vs system
        if self.is_package_manager_vs_system_conflict(instances) {
            return ConflictCategory::PackageManagerVsSystem;
        }

        // Check for duplicate versions
        if self.has_different_versions(instances) {
            return ConflictCategory::DuplicateVersions;
        }

        // Default to shadowed binary if none of the above apply
        ConflictCategory::ShadowedBinary
    }

    pub fn assess_severity(
        &self,
        category: ConflictCategory,
        instances: &[ExecutableInfo],
    ) -> Severity {
        match category {
            ConflictCategory::WslVsWindows => {
                // WSL/Windows mixing is typically high severity
                Severity::High
            }
            ConflictCategory::MultipleVersionManagers => {
                // Multiple version managers can cause confusion
                Severity::Medium
            }
            ConflictCategory::VersionManagerVsSystem => {
                // Check if versions differ significantly
                if self.has_major_version_difference(instances) {
                    Severity::Critical
                } else {
                    Severity::Medium
                }
            }
            ConflictCategory::PackageManagerVsSystem => Severity::Low,
            ConflictCategory::DuplicateVersions => {
                if self.has_major_version_difference(instances) {
                    Severity::High
                } else {
                    Severity::Low
                }
            }
            ConflictCategory::ShadowedBinary => {
                // Check if the shadowed binary is significantly different
                if self.are_likely_same_binary(instances) {
                    Severity::Info
                } else {
                    Severity::Medium
                }
            }
            ConflictCategory::Other => Severity::Low,
        }
    }

    pub fn generate_recommendation(
        &self,
        category: ConflictCategory,
        binary_name: &str,
        instances: &[ExecutableInfo],
    ) -> Option<String> {
        match category {
            ConflictCategory::WslVsWindows => Some(format!(
                "You're running WSL but have {} in both WSL and Windows PATH. \
                Consider using only the WSL version or removing Windows paths from WSL PATH.",
                binary_name
            )),
            ConflictCategory::MultipleVersionManagers => Some(format!(
                "Multiple version managers are managing {}. \
                Consider consolidating to a single version manager for consistency.",
                binary_name
            )),
            ConflictCategory::VersionManagerVsSystem => {
                let version_manager = instances
                    .iter()
                    .find(|i| {
                        i.manager
                            .as_ref()
                            .map(|m| m.manager_type == ManagerType::VersionManager)
                            .unwrap_or(false)
                    })
                    .and_then(|i| i.manager.as_ref())
                    .map(|m| m.name.as_str())
                    .unwrap_or("version manager");

                Some(format!(
                    "Consider using {} consistently or removing the system installation of {} to avoid confusion.",
                    version_manager, binary_name
                ))
            }
            ConflictCategory::DuplicateVersions => Some(format!(
                "Multiple versions of {} found. Ensure you're using the intended version.",
                binary_name
            )),
            _ => None,
        }
    }

    fn is_wsl_vs_windows_conflict(&self, instances: &[ExecutableInfo]) -> bool {
        if instances.len() < 2 {
            return false;
        }

        let has_wsl = instances.iter().any(|i| {
            wsl::is_wsl_path(&i.resolved_path) && !wsl::is_windows_path_in_wsl(&i.resolved_path)
        });

        let has_windows = instances.iter().any(|i| {
            wsl::is_windows_path_in_wsl(&i.resolved_path)
                || wsl::is_windows_executable_in_wsl(&i.resolved_path)
        });

        has_wsl && has_windows
    }

    fn is_multiple_version_managers_conflict(&self, instances: &[ExecutableInfo]) -> bool {
        let version_managers: Vec<_> = instances
            .iter()
            .filter_map(|i| i.manager.as_ref())
            .filter(|m| m.manager_type == ManagerType::VersionManager)
            .map(|m| m.name.as_str())
            .collect();

        let unique_managers: std::collections::HashSet<_> = version_managers.iter().collect();
        unique_managers.len() > 1
    }

    fn is_version_manager_vs_system_conflict(&self, instances: &[ExecutableInfo]) -> bool {
        let has_version_manager = instances.iter().any(|i| {
            i.manager
                .as_ref()
                .map(|m| m.manager_type == ManagerType::VersionManager)
                .unwrap_or(false)
        });

        let has_system = instances.iter().any(|i| {
            i.manager
                .as_ref()
                .map(|m| m.manager_type == ManagerType::SystemInstall)
                .unwrap_or(false)
        });

        has_version_manager && has_system
    }

    fn is_package_manager_vs_system_conflict(&self, instances: &[ExecutableInfo]) -> bool {
        let has_package_manager = instances.iter().any(|i| {
            i.manager
                .as_ref()
                .map(|m| m.manager_type == ManagerType::PackageManager)
                .unwrap_or(false)
        });

        let has_system = instances.iter().any(|i| {
            i.manager
                .as_ref()
                .map(|m| m.manager_type == ManagerType::SystemInstall)
                .unwrap_or(false)
        });

        has_package_manager && has_system
    }

    fn has_different_versions(&self, instances: &[ExecutableInfo]) -> bool {
        let versions: Vec<_> = instances
            .iter()
            .filter_map(|i| i.version.as_ref())
            .map(|v| v.raw.as_str())
            .collect();

        let unique_versions: std::collections::HashSet<_> = versions.iter().collect();
        unique_versions.len() > 1
    }

    fn has_major_version_difference(&self, instances: &[ExecutableInfo]) -> bool {
        let versions: Vec<_> = instances
            .iter()
            .filter_map(|i| i.version.as_ref())
            .filter_map(|v| self.extract_major_version(&v.raw))
            .collect();

        if versions.len() < 2 {
            return false;
        }

        let unique_major_versions: std::collections::HashSet<_> = versions.iter().collect();
        unique_major_versions.len() > 1
    }

    fn extract_major_version(&self, version: &str) -> Option<u32> {
        // Simple extraction of major version number
        let parts: Vec<&str> = version.split(&['.', '-', ' '][..]).collect();
        if let Some(first) = parts.first() {
            // Try to parse the first numeric part
            let numeric: String = first.chars().filter(|c| c.is_numeric()).collect();
            numeric.parse().ok()
        } else {
            None
        }
    }

    fn are_likely_same_binary(&self, instances: &[ExecutableInfo]) -> bool {
        if instances.len() < 2 {
            return false;
        }

        // Check if resolved paths are the same
        let resolved_paths: std::collections::HashSet<_> =
            instances.iter().map(|i| &i.resolved_path).collect();

        resolved_paths.len() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_platform() -> PlatformInfo {
        PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            is_wsl: true,
            wsl_version: Some("WSL2".to_string()),
            wsl_distro: Some("Ubuntu".to_string()),
        }
    }

    #[test]
    fn test_extract_major_version() {
        let categorizer = ConflictCategorizer::new(create_test_platform());

        assert_eq!(categorizer.extract_major_version("3.11.0"), Some(3));
        assert_eq!(categorizer.extract_major_version("v18.0.0"), Some(18));
        assert_eq!(categorizer.extract_major_version("1.70.0"), Some(1));
    }
}
