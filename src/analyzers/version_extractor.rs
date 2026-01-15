use crate::error::Result;
use crate::output::types::{ExecutableInfo, VersionInfo};
use regex::Regex;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct VersionExtractor {
    timeout_secs: u64,
}

impl VersionExtractor {
    pub fn new() -> Self {
        VersionExtractor { timeout_secs: 5 }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        VersionExtractor { timeout_secs }
    }

    pub fn extract_versions(&self, executables: &mut [ExecutableInfo]) {
        for executable in executables.iter_mut() {
            if let Some(version) = self.extract(&executable.full_path, &executable.name) {
                executable.version = Some(version);
            }
        }
    }

    pub fn extract(&self, path: &std::path::Path, binary_name: &str) -> Option<VersionInfo> {
        // Try different version extraction methods
        if let Some(version) = self.try_execution_methods(path) {
            return Some(version);
        }

        if let Some(version) = self.try_path_parsing(path, binary_name) {
            return Some(version);
        }

        None
    }

    fn try_execution_methods(&self, path: &std::path::Path) -> Option<VersionInfo> {
        let version_args = vec![
            vec!["--version"],
            vec!["-v"],
            vec!["version"],
            vec!["-V"],
            vec!["--version"],
        ];

        for args in version_args {
            if let Some(output) = self.execute_with_timeout(path, &args) {
                if let Some(version) = self.parse_version_output(&output) {
                    return Some(VersionInfo {
                        raw: version.clone(),
                        parsed: Some(version),
                        extraction_method: "command execution".to_string(),
                    });
                }
            }
        }

        None
    }

    fn execute_with_timeout(&self, path: &std::path::Path, args: &[&str]) -> Option<String> {
        // Try to execute the binary with the given arguments
        match Command::new(path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => {
                // Try stdout first
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if !stdout.trim().is_empty() {
                        return Some(stdout.trim().to_string());
                    }
                }

                // Try stderr as fallback (some tools output version to stderr)
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    if !stderr.trim().is_empty() {
                        return Some(stderr.trim().to_string());
                    }
                }

                None
            }
            Err(_) => None,
        }
    }

    fn parse_version_output(&self, output: &str) -> Option<String> {
        // Common version patterns
        let patterns = vec![
            // Semantic versioning: X.Y.Z
            r"(\d+\.\d+\.\d+)",
            // With 'v' prefix: vX.Y.Z
            r"v(\d+\.\d+\.\d+)",
            // Version word followed by number: "Version 3.11.0"
            r"[Vv]ersion\s+(\d+\.\d+(?:\.\d+)?)",
            // Python style: "Python 3.11.0"
            r"[A-Za-z]+\s+(\d+\.\d+(?:\.\d+)?)",
            // Simple X.Y format
            r"(\d+\.\d+)",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(output) {
                    if let Some(version) = caps.get(1) {
                        return Some(version.as_str().to_string());
                    }
                }
            }
        }

        // If no pattern matched, try to extract first line
        if let Some(first_line) = output.lines().next() {
            if !first_line.is_empty() && first_line.len() < 100 {
                return Some(first_line.to_string());
            }
        }

        None
    }

    fn try_path_parsing(&self, path: &std::path::Path, binary_name: &str) -> Option<VersionInfo> {
        let path_str = path.to_string_lossy();

        // Try to extract version from path
        // e.g., /usr/local/lib/python3.11/bin/python -> 3.11
        // e.g., /home/user/.nvm/versions/node/v18.0.0/bin/node -> 18.0.0

        let patterns = vec![
            // Pattern like "python3.11"
            format!(r"{}\s*(\d+\.\d+(?:\.\d+)?)", binary_name),
            // Pattern like "v18.0.0" in path
            r"v(\d+\.\d+\.\d+)".to_string(),
            // Pattern like "3.11" in path
            r"/(\d+\.\d+(?:\.\d+)?)/".to_string(),
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(&pattern) {
                if let Some(caps) = re.captures(&path_str) {
                    if let Some(version) = caps.get(1) {
                        return Some(VersionInfo {
                            raw: version.as_str().to_string(),
                            parsed: Some(version.as_str().to_string()),
                            extraction_method: "path parsing".to_string(),
                        });
                    }
                }
            }
        }

        None
    }
}

impl Default for VersionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_output() {
        let extractor = VersionExtractor::new();

        assert_eq!(
            extractor.parse_version_output("Python 3.11.0"),
            Some("3.11.0".to_string())
        );

        assert_eq!(
            extractor.parse_version_output("node v18.0.0"),
            Some("18.0.0".to_string())
        );

        assert_eq!(
            extractor.parse_version_output("rustc 1.70.0"),
            Some("1.70.0".to_string())
        );

        assert_eq!(
            extractor.parse_version_output("Version 2.5"),
            Some("2.5".to_string())
        );
    }

    #[test]
    fn test_try_path_parsing() {
        let extractor = VersionExtractor::new();

        let result = extractor.try_path_parsing(
            &std::path::PathBuf::from("/usr/local/lib/python3.11/bin/python"),
            "python",
        );
        assert!(result.is_some());

        let result = extractor.try_path_parsing(
            &std::path::PathBuf::from("/home/user/.nvm/versions/node/v18.0.0/bin/node"),
            "node",
        );
        assert!(result.is_some());
    }
}
