use crate::error::Result;
use crate::output::types::{ExecutableInfo, PathEntry};
use crate::platform;
use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct ExecutableScanner {
    max_depth: usize,
    follow_symlinks: bool,
}

impl ExecutableScanner {
    pub fn new() -> Self {
        ExecutableScanner {
            max_depth: 1, // Only scan the directory itself, not subdirectories
            follow_symlinks: false,
        }
    }

    pub fn with_options(max_depth: usize, follow_symlinks: bool) -> Self {
        ExecutableScanner {
            max_depth,
            follow_symlinks,
        }
    }

    pub fn scan_path_entries(&self, entries: &mut [PathEntry]) -> Result<()> {
        for entry in entries.iter_mut() {
            if !entry.exists || !entry.is_accessible {
                continue;
            }

            // Skip Windows system directories - they contain hundreds of system utilities
            // that aren't relevant for developer tool conflict detection
            if self.should_skip_directory(&entry.path) {
                if cfg!(debug_assertions) {
                    eprintln!("Skipping system directory: {}", entry.path.display());
                }
                continue;
            }

            match self.scan_directory(&entry.path, entry.order) {
                Ok(executables) => {
                    entry.executables = executables;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to scan {}: {}", entry.path.display(), e);
                    // Continue with other directories even if one fails
                }
            }
        }

        Ok(())
    }

    fn should_skip_directory(&self, _path: &std::path::Path) -> bool {
        // Windows system directories
        #[cfg(windows)]
        {
            let path_str = _path.to_string_lossy().to_lowercase();
            if path_str.contains("windows\\system32")
                || path_str.contains("windows\\syswow64")
                || path_str.contains("windows\\winsxs")
                || path_str.starts_with("c:\\windows\\")
            {
                return true;
            }
        }

        // Skip very large system directories on any platform
        false
    }

    pub fn scan_directory(&self, path: &PathBuf, path_order: usize) -> Result<Vec<ExecutableInfo>> {
        let mut executables = Vec::new();
        let mut seen_names = HashSet::new();

        let walker = WalkDir::new(path)
            .max_depth(self.max_depth)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden directories (but not the root)
                if e.depth() > 0 {
                    !e.file_name()
                        .to_str()
                        .map(|s| s.starts_with('.'))
                        .unwrap_or(false)
                } else {
                    true
                }
            });

        for entry_result in walker {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue, // Skip inaccessible entries
            };

            let entry_path = entry.path();

            // Skip directories
            if entry_path.is_dir() {
                continue;
            }

            // Check if it's an executable
            if !platform::is_executable(entry_path) {
                continue;
            }

            // Get the binary name (without extension on Windows)
            let binary_name = self.get_binary_name(entry_path);

            // Skip duplicates in the same directory
            if seen_names.contains(&binary_name) {
                continue;
            }

            seen_names.insert(binary_name.clone());

            // Get metadata
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let is_symlink = entry_path.is_symlink();
            let symlink_target = if is_symlink {
                std::fs::read_link(entry_path).ok()
            } else {
                None
            };

            // For now, resolved_path is the same as full_path
            // This will be updated by the symlink resolver
            let resolved_path = entry_path.to_path_buf();

            executables.push(ExecutableInfo {
                name: binary_name,
                full_path: entry_path.to_path_buf(),
                size,
                modified,
                is_symlink,
                symlink_target,
                resolved_path,
                version: None,   // Will be filled by version extractor
                manager: None,   // Will be filled by manager detector
                file_hash: None, // Optional, can be computed if needed
                path_order,
            });
        }

        Ok(executables)
    }

    fn get_binary_name(&self, path: &std::path::Path) -> String {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // On Windows, remove common executable extensions
        if cfg!(windows) {
            let name_lower = file_name.to_lowercase();
            for ext in &[".exe", ".bat", ".cmd", ".ps1", ".com"] {
                if name_lower.ends_with(ext) {
                    return file_name[..file_name.len() - ext.len()].to_string();
                }
            }
        }

        file_name.to_string()
    }
}

impl Default for ExecutableScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_binary_name() {
        let scanner = ExecutableScanner::new();

        #[cfg(windows)]
        {
            assert_eq!(
                scanner.get_binary_name(&PathBuf::from("python.exe")),
                "python"
            );
            assert_eq!(
                scanner.get_binary_name(&PathBuf::from("script.bat")),
                "script"
            );
        }

        #[cfg(unix)]
        {
            assert_eq!(scanner.get_binary_name(&PathBuf::from("python")), "python");
            assert_eq!(scanner.get_binary_name(&PathBuf::from("node")), "node");
        }
    }
}
