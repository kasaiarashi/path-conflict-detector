use crate::error::{Error, Result};
use crate::output::types::PathEntry;
use crate::platform;
use std::path::PathBuf;

pub struct PathParser {
    separator: char,
}

impl PathParser {
    pub fn new() -> Self {
        PathParser {
            separator: platform::get_path_separator(),
        }
    }

    pub fn parse_system_path(&self) -> Result<Vec<PathEntry>> {
        let path_var = platform::get_path_env_var()?;
        self.parse_path(&path_var)
    }

    pub fn parse_path(&self, path_var: &str) -> Result<Vec<PathEntry>> {
        let paths: Vec<&str> = path_var.split(self.separator).collect();
        let mut entries = Vec::new();

        for (order, path_str) in paths.iter().enumerate() {
            if path_str.trim().is_empty() {
                continue;
            }

            let expanded = platform::expand_env_vars(path_str.trim());
            let path_buf = self.normalize_path(&expanded);

            let exists = path_buf.exists();
            let is_accessible = self.check_accessibility(&path_buf);

            entries.push(PathEntry {
                path: path_buf,
                order,
                exists,
                is_accessible,
                executables: Vec::new(), // Will be populated by scanner
            });
        }

        Ok(entries)
    }

    fn normalize_path(&self, path: &str) -> PathBuf {
        let mut path_buf = PathBuf::from(path);

        // Resolve relative paths to absolute
        if path_buf.is_relative() {
            if let Ok(current_dir) = std::env::current_dir() {
                path_buf = current_dir.join(path_buf);
            }
        }

        // Canonicalize if possible (resolves .. and .)
        if let Ok(canonical) = path_buf.canonicalize() {
            return canonical;
        }

        path_buf
    }

    fn check_accessibility(&self, path: &PathBuf) -> bool {
        if !path.exists() {
            return false;
        }

        // Try to read the directory
        match std::fs::read_dir(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl Default for PathParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let parser = PathParser::new();

        #[cfg(unix)]
        let path_var = "/usr/bin:/usr/local/bin:/home/user/bin";

        #[cfg(windows)]
        let path_var = "C:\\Windows\\System32;C:\\Windows;C:\\Program Files\\Git\\cmd";

        let result = parser.parse_path(path_var);
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].order, 0);
        assert_eq!(entries[1].order, 1);
        assert_eq!(entries[2].order, 2);
    }

    #[test]
    fn test_normalize_path() {
        let parser = PathParser::new();

        #[cfg(unix)]
        {
            let normalized = parser.normalize_path("/usr/bin");
            assert!(normalized.is_absolute());
        }

        #[cfg(windows)]
        {
            let normalized = parser.normalize_path("C:\\Windows");
            assert!(normalized.is_absolute());
        }
    }
}
