use crate::analyzers::ConflictCategorizer;
use crate::error::Result;
use crate::output::types::{Conflict, ExecutableInfo, PathEntry, PlatformInfo};
use std::collections::HashMap;

pub struct ConflictDetector {
    categorizer: ConflictCategorizer,
}

impl ConflictDetector {
    pub fn new(platform: PlatformInfo) -> Self {
        ConflictDetector {
            categorizer: ConflictCategorizer::new(platform),
        }
    }

    pub fn detect_conflicts(&self, path_entries: &[PathEntry]) -> Result<Vec<Conflict>> {
        // Build an index of all executables by binary name
        let mut executable_index: HashMap<String, Vec<ExecutableInfo>> = HashMap::new();

        for entry in path_entries {
            for executable in &entry.executables {
                executable_index
                    .entry(executable.name.clone())
                    .or_insert_with(Vec::new)
                    .push(executable.clone());
            }
        }

        // Find all binaries with multiple instances (conflicts)
        let mut conflicts = Vec::new();

        for (binary_name, mut instances) in executable_index {
            if instances.len() <= 1 {
                // No conflict, skip
                continue;
            }

            // Sort instances by PATH order (lower order = higher priority)
            instances.sort_by_key(|i| i.path_order);

            // The first instance is the active one (what gets executed)
            let active_instance = instances[0].clone();

            // Categorize the conflict
            let category = self.categorizer.categorize(&binary_name, &instances);

            // Assess severity
            let severity = self.categorizer.assess_severity(category, &instances);

            // Generate description
            let description = self.generate_description(&binary_name, &instances, &active_instance);

            // Generate recommendation
            let recommendation = self
                .categorizer
                .generate_recommendation(category, &binary_name, &instances);

            conflicts.push(Conflict {
                binary_name,
                instances,
                active_instance,
                category,
                severity,
                description,
                recommendation,
            });
        }

        // Sort conflicts by severity (critical first)
        conflicts.sort_by(|a, b| b.severity.cmp(&a.severity));

        Ok(conflicts)
    }

    pub fn find_binary_conflicts(&self, path_entries: &[PathEntry], binary_name: &str) -> Result<Option<Conflict>> {
        let all_conflicts = self.detect_conflicts(path_entries)?;
        Ok(all_conflicts
            .into_iter()
            .find(|c| c.binary_name == binary_name))
    }

    fn generate_description(
        &self,
        binary_name: &str,
        instances: &[ExecutableInfo],
        active_instance: &ExecutableInfo,
    ) -> String {
        let count = instances.len();
        let shadowed_count = count - 1;

        let active_path = active_instance.full_path.display();
        let active_version = active_instance
            .version
            .as_ref()
            .map(|v| format!(" ({})", v.raw))
            .unwrap_or_default();

        if shadowed_count == 1 {
            format!(
                "{} has 1 shadowed instance. Active: {}{}",
                binary_name, active_path, active_version
            )
        } else {
            format!(
                "{} has {} shadowed instances. Active: {}{}",
                binary_name, shadowed_count, active_path, active_version
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::types::PlatformInfo;
    use std::path::PathBuf;

    fn create_test_platform() -> PlatformInfo {
        PlatformInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            is_wsl: false,
            wsl_version: None,
            wsl_distro: None,
        }
    }

    #[test]
    fn test_no_conflicts() {
        let detector = ConflictDetector::new(create_test_platform());
        let path_entries = vec![
            PathEntry {
                path: PathBuf::from("/usr/bin"),
                order: 0,
                exists: true,
                is_accessible: true,
                executables: vec![ExecutableInfo {
                    name: "python".to_string(),
                    full_path: PathBuf::from("/usr/bin/python"),
                    size: 1000,
                    modified: 0,
                    is_symlink: false,
                    symlink_target: None,
                    resolved_path: PathBuf::from("/usr/bin/python"),
                    version: None,
                    manager: None,
                    file_hash: None,
                    path_order: 0,
                }],
            },
        ];

        let result = detector.detect_conflicts(&path_entries).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_with_conflicts() {
        let detector = ConflictDetector::new(create_test_platform());
        let path_entries = vec![
            PathEntry {
                path: PathBuf::from("/usr/bin"),
                order: 0,
                exists: true,
                is_accessible: true,
                executables: vec![ExecutableInfo {
                    name: "python".to_string(),
                    full_path: PathBuf::from("/usr/bin/python"),
                    size: 1000,
                    modified: 0,
                    is_symlink: false,
                    symlink_target: None,
                    resolved_path: PathBuf::from("/usr/bin/python"),
                    version: None,
                    manager: None,
                    file_hash: None,
                    path_order: 0,
                }],
            },
            PathEntry {
                path: PathBuf::from("/usr/local/bin"),
                order: 1,
                exists: true,
                is_accessible: true,
                executables: vec![ExecutableInfo {
                    name: "python".to_string(),
                    full_path: PathBuf::from("/usr/local/bin/python"),
                    size: 2000,
                    modified: 0,
                    is_symlink: false,
                    symlink_target: None,
                    resolved_path: PathBuf::from("/usr/local/bin/python"),
                    version: None,
                    manager: None,
                    file_hash: None,
                    path_order: 1,
                }],
            },
        ];

        let result = detector.detect_conflicts(&path_entries).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].binary_name, "python");
        assert_eq!(result[0].instances.len(), 2);
    }
}
