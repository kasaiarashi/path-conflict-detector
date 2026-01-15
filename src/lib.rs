pub mod analyzers;
pub mod cli;
pub mod core;
pub mod error;
pub mod output;
pub mod platform;

pub use error::{Error, Result};
pub use output::types::*;

use chrono::Utc;
use std::collections::HashMap;

/// Options for configuring the analysis
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub extract_versions: bool,
    pub resolve_symlinks: bool,
    pub categorize_managers: bool,
    pub include_file_hashes: bool,
    pub custom_path: Option<String>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            extract_versions: false, // Disabled by default due to Windows issues
            resolve_symlinks: true,
            categorize_managers: true,
            include_file_hashes: false,
            custom_path: None,
        }
    }
}

/// Main API for analyzing PATH conflicts
pub struct PathAnalyzer {
    options: AnalysisOptions,
}

impl PathAnalyzer {
    /// Create a new PathAnalyzer with default options
    pub fn new() -> Self {
        PathAnalyzer {
            options: AnalysisOptions::default(),
        }
    }

    /// Create a PathAnalyzer with custom options
    pub fn with_options(options: AnalysisOptions) -> Self {
        PathAnalyzer { options }
    }

    /// Run a full PATH analysis
    pub fn analyze(&self) -> Result<AnalysisResult> {
        let scan_time = Utc::now();

        // Detect platform
        let platform = platform::detect_platform()?;

        // Parse PATH
        let path_parser = core::PathParser::new();
        let mut path_entries = if let Some(custom_path) = &self.options.custom_path {
            path_parser.parse_path(custom_path)?
        } else {
            path_parser.parse_system_path()?
        };

        // Scan for executables
        let scanner = core::ExecutableScanner::new();
        scanner.scan_path_entries(&mut path_entries)?;

        // Collect all executables
        let mut all_executables: Vec<ExecutableInfo> = path_entries
            .iter()
            .flat_map(|entry| entry.executables.iter().cloned())
            .collect();

        // Resolve symlinks
        if self.options.resolve_symlinks {
            let symlink_resolver = analyzers::SymlinkResolver::new();
            symlink_resolver.resolve_executables(&mut all_executables)?;

            // Update executables in path entries
            for entry in &mut path_entries {
                for exec in &mut entry.executables {
                    if let Some(resolved) = all_executables
                        .iter()
                        .find(|e| e.full_path == exec.full_path)
                    {
                        exec.resolved_path = resolved.resolved_path.clone();
                    }
                }
            }
        }

        // Detect managers
        if self.options.categorize_managers {
            let manager_detector = analyzers::ManagerDetector::new();
            manager_detector.detect_managers(&mut all_executables);

            // Update executables in path entries
            for entry in &mut path_entries {
                for exec in &mut entry.executables {
                    if let Some(detected) = all_executables
                        .iter()
                        .find(|e| e.full_path == exec.full_path)
                    {
                        exec.manager = detected.manager.clone();
                    }
                }
            }
        }

        // Extract versions
        if self.options.extract_versions {
            let version_extractor = analyzers::VersionExtractor::new();
            version_extractor.extract_versions(&mut all_executables);

            // Update executables in path entries
            for entry in &mut path_entries {
                for exec in &mut entry.executables {
                    if let Some(versioned) = all_executables
                        .iter()
                        .find(|e| e.full_path == exec.full_path)
                    {
                        exec.version = versioned.version.clone();
                    }
                }
            }
        }

        // Compute hashes if requested
        if self.options.include_file_hashes {
            let binary_info_extractor = core::BinaryInfoExtractor::new(true);
            binary_info_extractor.enrich_executables(&mut all_executables)?;

            // Update executables in path entries
            for entry in &mut path_entries {
                for exec in &mut entry.executables {
                    if let Some(hashed) = all_executables
                        .iter()
                        .find(|e| e.full_path == exec.full_path)
                    {
                        exec.file_hash = hashed.file_hash.clone();
                    }
                }
            }
        }

        // Detect conflicts
        let conflict_detector = core::ConflictDetector::new(platform.clone());
        let conflicts = conflict_detector.detect_conflicts(&path_entries)?;

        // Build summary
        let summary = self.build_summary(&path_entries, &conflicts);

        Ok(AnalysisResult {
            scan_time,
            platform,
            path_entries,
            conflicts,
            summary,
        })
    }

    /// Find conflicts for a specific binary
    pub fn check_binary(&self, binary_name: &str) -> Result<Vec<ExecutableInfo>> {
        let result = self.analyze()?;

        let executables: Vec<ExecutableInfo> = result
            .path_entries
            .iter()
            .flat_map(|entry| &entry.executables)
            .filter(|exec| exec.name == binary_name)
            .cloned()
            .collect();

        Ok(executables)
    }

    /// Find all conflicts
    pub fn find_conflicts(&self) -> Result<Vec<Conflict>> {
        let result = self.analyze()?;
        Ok(result.conflicts)
    }

    fn build_summary(&self, path_entries: &[PathEntry], conflicts: &[Conflict]) -> Summary {
        let total_path_entries = path_entries.len();
        let total_executables: usize = path_entries.iter().map(|e| e.executables.len()).sum();

        // Count unique executables
        let unique_names: std::collections::HashSet<_> = path_entries
            .iter()
            .flat_map(|e| &e.executables)
            .map(|exec| &exec.name)
            .collect();
        let unique_executables = unique_names.len();

        let total_conflicts = conflicts.len();

        // Count conflicts by category
        let mut conflicts_by_category: HashMap<ConflictCategory, usize> = HashMap::new();
        for conflict in conflicts {
            *conflicts_by_category.entry(conflict.category).or_insert(0) += 1;
        }

        // Count conflicts by severity
        let mut conflicts_by_severity: HashMap<Severity, usize> = HashMap::new();
        for conflict in conflicts {
            *conflicts_by_severity.entry(conflict.severity).or_insert(0) += 1;
        }

        Summary {
            total_path_entries,
            total_executables,
            unique_executables,
            total_conflicts,
            conflicts_by_category,
            conflicts_by_severity,
        }
    }
}

impl Default for PathAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
