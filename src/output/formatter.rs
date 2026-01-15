use crate::output::types::*;
use colored::*;

pub struct HumanFormatter {
    show_recommendations: bool,
    verbose: bool,
}

impl HumanFormatter {
    pub fn new(show_recommendations: bool, verbose: bool) -> Self {
        HumanFormatter {
            show_recommendations,
            verbose,
        }
    }

    pub fn format(&self, result: &AnalysisResult) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.format_header(result));
        output.push('\n');

        // Summary
        output.push_str(&self.format_summary(&result.summary));
        output.push('\n');

        // Conflicts by category
        if !result.conflicts.is_empty() {
            output.push_str(&self.format_conflicts_by_category(&result.summary));
            output.push('\n');
        }

        // Detailed conflicts
        if !result.conflicts.is_empty() {
            output.push_str(&self.format_detailed_conflicts(&result.conflicts));
        } else {
            output.push_str(&"No conflicts detected! All executables in PATH are unique.\n".green().to_string());
        }

        output
    }

    fn format_header(&self, result: &AnalysisResult) -> String {
        let mut output = String::new();

        output.push_str(&"PATH Conflict Analysis Report\n".bold().to_string());
        output.push_str(&"═".repeat(60));
        output.push('\n');

        let platform_info = format!(
            "Platform: {} ({})",
            result.platform.os, result.platform.arch
        );
        output.push_str(&platform_info);

        if result.platform.is_wsl {
            let wsl_info = format!(
                " with {}",
                result.platform.wsl_version.as_ref().unwrap_or(&"WSL".to_string())
            );
            output.push_str(&wsl_info.cyan().to_string());
        }

        output.push('\n');
        output.push_str(&format!("Scan Time: {}\n", result.scan_time.format("%Y-%m-%d %H:%M:%S UTC")));

        output
    }

    fn format_summary(&self, summary: &Summary) -> String {
        let mut output = String::new();

        output.push('\n');
        output.push_str(&"SUMMARY\n".bold().to_string());
        output.push_str(&"─".repeat(60));
        output.push('\n');

        output.push_str(&format!("Total PATH Entries: {}\n", summary.total_path_entries));
        output.push_str(&format!("Total Executables: {}\n", summary.total_executables));
        output.push_str(&format!("Unique Executables: {}\n", summary.unique_executables));

        if summary.total_conflicts > 0 {
            output.push_str(&format!("Conflicts Found: {}\n", summary.total_conflicts).red().bold().to_string());
        } else {
            output.push_str(&format!("Conflicts Found: {}\n", summary.total_conflicts).green().to_string());
        }

        output
    }

    fn format_conflicts_by_category(&self, summary: &Summary) -> String {
        let mut output = String::new();

        output.push('\n');
        output.push_str(&"CONFLICTS BY CATEGORY\n".bold().to_string());
        output.push_str(&"─".repeat(60));
        output.push('\n');

        let categories = vec![
            (ConflictCategory::WslVsWindows, "🔴"),
            (ConflictCategory::VersionManagerVsSystem, "🟡"),
            (ConflictCategory::MultipleVersionManagers, "🟡"),
            (ConflictCategory::DuplicateVersions, "🔵"),
            (ConflictCategory::ShadowedBinary, "⚪"),
        ];

        for (category, icon) in categories {
            if let Some(count) = summary.conflicts_by_category.get(&category) {
                if *count > 0 {
                    output.push_str(&format!("{} {} ({})\n", icon, category, count));
                }
            }
        }

        output
    }

    fn format_detailed_conflicts(&self, conflicts: &[Conflict]) -> String {
        let mut output = String::new();

        output.push('\n');
        output.push_str(&"DETAILED CONFLICTS\n".bold().to_string());
        output.push_str(&"═".repeat(60));
        output.push('\n');

        for (idx, conflict) in conflicts.iter().enumerate() {
            output.push('\n');
            output.push_str(&self.format_conflict(idx + 1, conflict));
        }

        output
    }

    fn format_conflict(&self, number: usize, conflict: &Conflict) -> String {
        let mut output = String::new();

        // Conflict header
        let severity_icon = self.severity_icon(&conflict.severity);
        let header = format!(
            "[{}] {} {}: {} ({})",
            number,
            severity_icon,
            conflict.severity,
            conflict.binary_name,
            conflict.category
        );

        output.push_str(&self.colorize_by_severity(&header, &conflict.severity).bold().to_string());
        output.push('\n');
        output.push_str(&"─".repeat(60));
        output.push('\n');

        // Active instance
        output.push_str(&"Active: ".green().bold().to_string());
        output.push_str(&self.format_executable(&conflict.active_instance, true));
        output.push('\n');

        // Shadowed instances
        if conflict.instances.len() > 1 {
            output.push('\n');
            output.push_str(&"Shadowed instances:\n".yellow().to_string());
            for (idx, instance) in conflict.instances.iter().enumerate().skip(1) {
                output.push_str(&format!("   [{}] ", idx + 1));
                output.push_str(&self.format_executable(instance, false));
                output.push('\n');
            }
        }

        // Recommendation
        if self.show_recommendations {
            if let Some(recommendation) = &conflict.recommendation {
                output.push('\n');
                output.push_str(&"Recommendation: ".cyan().bold().to_string());
                output.push_str(recommendation);
                output.push('\n');
            }
        }

        output
    }

    fn format_executable(&self, exec: &ExecutableInfo, _is_active: bool) -> String {
        let mut parts = vec![];

        parts.push(exec.full_path.display().to_string());

        if let Some(version) = &exec.version {
            parts.push(format!("→ {}", version.raw));
        }

        if self.verbose {
            if let Some(manager) = &exec.manager {
                parts.push(format!("({})", manager.name));
            }
        }

        parts.join(" ")
    }

    fn severity_icon(&self, severity: &Severity) -> &str {
        match severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
            Severity::Info => "⚪",
        }
    }

    fn colorize_by_severity(&self, text: &str, severity: &Severity) -> ColoredString {
        match severity {
            Severity::Critical => text.red(),
            Severity::High => text.red(),
            Severity::Medium => text.yellow(),
            Severity::Low => text.blue(),
            Severity::Info => text.normal(),
        }
    }
}

impl Default for HumanFormatter {
    fn default() -> Self {
        Self::new(false, false)
    }
}
