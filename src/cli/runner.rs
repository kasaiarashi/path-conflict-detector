use crate::cli::args::{Args, OutputFormat};
use crate::error::Result;
use crate::output::{formatter::HumanFormatter, json_output};
use crate::{AnalysisOptions, PathAnalyzer};

pub fn run(args: Args) -> Result<()> {
    // Determine output format
    let output_format = if args.json {
        OutputFormat::Json
    } else {
        args.output
    };

    // Build analysis options from CLI args
    let options = AnalysisOptions {
        extract_versions: args.extract_versions,
        resolve_symlinks: args.resolve_symlinks,
        categorize_managers: true,
        include_file_hashes: args.include_hashes,
        custom_path: args.custom_path,
    };

    // Create analyzer and run analysis
    let analyzer = PathAnalyzer::with_options(options);
    let mut result = analyzer.analyze()?;

    // Filter conflicts if needed
    if let Some(binary_name) = &args.binary {
        result.conflicts.retain(|c| c.binary_name == *binary_name);
    }

    if let Some(category_filter) = args.category {
        result.conflicts.retain(|c| {
            matches!(
                (category_filter, c.category),
                (
                    crate::cli::args::CategoryFilter::WslVsWindows,
                    crate::output::types::ConflictCategory::WslVsWindows
                ) | (
                    crate::cli::args::CategoryFilter::VersionManagerVsSystem,
                    crate::output::types::ConflictCategory::VersionManagerVsSystem
                ) | (
                    crate::cli::args::CategoryFilter::MultipleVersionManagers,
                    crate::output::types::ConflictCategory::MultipleVersionManagers
                ) | (
                    crate::cli::args::CategoryFilter::PackageManagerVsSystem,
                    crate::output::types::ConflictCategory::PackageManagerVsSystem
                ) | (
                    crate::cli::args::CategoryFilter::DuplicateVersions,
                    crate::output::types::ConflictCategory::DuplicateVersions
                ) | (
                    crate::cli::args::CategoryFilter::ShadowedBinary,
                    crate::output::types::ConflictCategory::ShadowedBinary
                )
            )
        });
    }

    if let Some(severity_filter) = args.severity {
        let min_severity = match severity_filter {
            crate::cli::args::SeverityFilter::Info => crate::output::types::Severity::Info,
            crate::cli::args::SeverityFilter::Low => crate::output::types::Severity::Low,
            crate::cli::args::SeverityFilter::Medium => crate::output::types::Severity::Medium,
            crate::cli::args::SeverityFilter::High => crate::output::types::Severity::High,
            crate::cli::args::SeverityFilter::Critical => crate::output::types::Severity::Critical,
        };

        result.conflicts.retain(|c| c.severity >= min_severity);
    }

    // Update summary after filtering
    result.summary.total_conflicts = result.conflicts.len();

    // Format and output
    match output_format {
        OutputFormat::Human => {
            let formatter = HumanFormatter::new(args.recommendations, args.verbose);
            let output = formatter.format(&result);
            if !args.quiet {
                println!("{}", output);
            }
        }
        OutputFormat::Json => {
            let json = json_output::format_json(&result, false)?;
            println!("{}", json);
        }
        OutputFormat::JsonPretty => {
            let json = json_output::format_json(&result, true)?;
            println!("{}", json);
        }
    }

    // Exit with non-zero code if conflicts found (unless quiet mode)
    if !result.conflicts.is_empty() && !args.quiet {
        std::process::exit(1);
    }

    Ok(())
}
