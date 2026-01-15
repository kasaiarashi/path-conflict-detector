use crate::error::{Error, Result};
use crate::output::types::ExecutableInfo;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct SymlinkResolver {
    max_depth: usize,
}

impl SymlinkResolver {
    pub fn new() -> Self {
        SymlinkResolver { max_depth: 10 }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        SymlinkResolver { max_depth }
    }

    pub fn resolve_executables(&self, executables: &mut [ExecutableInfo]) -> Result<()> {
        for executable in executables.iter_mut() {
            if executable.is_symlink {
                match self.resolve(&executable.full_path) {
                    Ok(resolved) => {
                        executable.resolved_path = resolved;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to resolve symlink {}: {}",
                            executable.full_path.display(),
                            e
                        );
                        // Keep the original path as resolved_path
                        executable.resolved_path = executable.full_path.clone();
                    }
                }
            } else {
                // Not a symlink, resolved path is the same as full path
                executable.resolved_path = executable.full_path.clone();
            }
        }

        Ok(())
    }

    pub fn resolve(&self, path: &std::path::Path) -> Result<PathBuf> {
        let mut current = path.to_path_buf();
        let mut seen = HashSet::new();
        let mut depth = 0;

        while current.is_symlink() && depth < self.max_depth {
            // Check for circular symlinks
            if seen.contains(&current) {
                return Err(Error::CircularSymlink {
                    path: current.to_string_lossy().to_string(),
                });
            }

            seen.insert(current.clone());

            // Read the symlink target
            let target = fs::read_link(&current).map_err(|_| Error::SymlinkError {
                path: current.to_string_lossy().to_string(),
            })?;

            // If target is relative, resolve it relative to the symlink's directory
            current = if target.is_relative() {
                if let Some(parent) = current.parent() {
                    parent.join(target)
                } else {
                    target
                }
            } else {
                target
            };

            depth += 1;
        }

        // Try to canonicalize the final path
        if let Ok(canonical) = current.canonicalize() {
            Ok(canonical)
        } else {
            Ok(current)
        }
    }

    pub fn are_same_binary(&self, path1: &std::path::Path, path2: &std::path::Path) -> bool {
        let resolved1 = self.resolve(path1).ok();
        let resolved2 = self.resolve(path2).ok();

        if let (Some(r1), Some(r2)) = (resolved1, resolved2) {
            r1 == r2
        } else {
            false
        }
    }
}

impl Default for SymlinkResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symlink_resolver_creation() {
        let resolver = SymlinkResolver::new();
        assert_eq!(resolver.max_depth, 10);

        let resolver_custom = SymlinkResolver::with_max_depth(5);
        assert_eq!(resolver_custom.max_depth, 5);
    }
}
