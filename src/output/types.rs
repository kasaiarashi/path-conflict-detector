use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub scan_time: DateTime<Utc>,
    pub platform: PlatformInfo,
    pub path_entries: Vec<PathEntry>,
    pub conflicts: Vec<Conflict>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub is_wsl: bool,
    pub wsl_version: Option<String>,
    pub wsl_distro: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub path: PathBuf,
    pub order: usize,
    pub exists: bool,
    pub is_accessible: bool,
    pub executables: Vec<ExecutableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutableInfo {
    pub name: String,
    pub full_path: PathBuf,
    pub size: u64,
    pub modified: i64, // Unix timestamp for easier comparison
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    pub resolved_path: PathBuf,
    pub version: Option<VersionInfo>,
    pub manager: Option<ManagerInfo>,
    pub file_hash: Option<String>,
    pub path_order: usize, // Position in PATH (lower = higher priority)
}

impl std::hash::Hash for ExecutableInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.full_path.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    pub raw: String,
    pub parsed: Option<String>, // semver string
    pub extraction_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManagerInfo {
    pub manager_type: ManagerType,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ManagerType {
    VersionManager,  // nvm, pyenv, rbenv, rustup
    PackageManager,  // brew, apt, chocolatey
    SystemInstall,   // System-installed
    ManualInstall,   // User-installed manually
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub binary_name: String,
    pub instances: Vec<ExecutableInfo>,
    pub active_instance: ExecutableInfo,
    pub category: ConflictCategory,
    pub severity: Severity,
    pub description: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConflictCategory {
    WslVsWindows,
    VersionManagerVsSystem,
    MultipleVersionManagers,
    PackageManagerVsSystem,
    DuplicateVersions,
    ShadowedBinary,
    Other,
}

impl std::fmt::Display for ConflictCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictCategory::WslVsWindows => write!(f, "WSL vs Windows"),
            ConflictCategory::VersionManagerVsSystem => write!(f, "Version Manager vs System"),
            ConflictCategory::MultipleVersionManagers => write!(f, "Multiple Version Managers"),
            ConflictCategory::PackageManagerVsSystem => write!(f, "Package Manager vs System"),
            ConflictCategory::DuplicateVersions => write!(f, "Duplicate Versions"),
            ConflictCategory::ShadowedBinary => write!(f, "Shadowed Binary"),
            ConflictCategory::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_path_entries: usize,
    pub total_executables: usize,
    pub unique_executables: usize,
    pub total_conflicts: usize,
    pub conflicts_by_category: HashMap<ConflictCategory, usize>,
    pub conflicts_by_severity: HashMap<Severity, usize>,
}

impl Summary {
    pub fn new() -> Self {
        Summary {
            total_path_entries: 0,
            total_executables: 0,
            unique_executables: 0,
            total_conflicts: 0,
            conflicts_by_category: HashMap::new(),
            conflicts_by_severity: HashMap::new(),
        }
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}
