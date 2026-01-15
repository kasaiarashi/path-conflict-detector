use crate::error::{Error, Result};
use crate::output::types::AnalysisResult;

pub fn format_json(result: &AnalysisResult, pretty: bool) -> Result<String> {
    if pretty {
        serde_json::to_string_pretty(result).map_err(|e| Error::SerializationError(e.to_string()))
    } else {
        serde_json::to_string(result).map_err(|e| Error::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::types::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_result() -> AnalysisResult {
        AnalysisResult {
            scan_time: Utc::now(),
            platform: PlatformInfo {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                is_wsl: false,
                wsl_version: None,
                wsl_distro: None,
            },
            path_entries: vec![],
            conflicts: vec![],
            summary: Summary {
                total_path_entries: 0,
                total_executables: 0,
                unique_executables: 0,
                total_conflicts: 0,
                conflicts_by_category: HashMap::new(),
                conflicts_by_severity: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_format_json() {
        let result = create_test_result();
        let json = format_json(&result, false);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("scan_time"));
        assert!(json_str.contains("platform"));
    }

    #[test]
    fn test_format_json_pretty() {
        let result = create_test_result();
        let json = format_json(&result, true);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains('\n')); // Pretty format should have newlines
    }
}
