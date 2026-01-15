use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "path-conflict-detector")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "Examples:\n  \
    path-conflict-detector\n  \
    path-conflict-detector --json\n  \
    path-conflict-detector --binary python\n  \
    path-conflict-detector --severity high\n  \
    path-conflict-detector --category wsl-vs-windows\n  \
    path-conflict-detector --conflicts-only --recommendations")]
pub struct Args {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Human)]
    pub output: OutputFormat,

    /// Use JSON output (shorthand for --output json)
    #[arg(long, conflicts_with = "output")]
    pub json: bool,

    /// Check specific binary name
    #[arg(short, long)]
    pub binary: Option<String>,

    /// Filter by conflict category
    #[arg(short, long, value_enum)]
    pub category: Option<CategoryFilter>,

    /// Filter by minimum severity level
    #[arg(short, long, value_enum)]
    pub severity: Option<SeverityFilter>,

    /// Show only conflicts (hide non-conflicting binaries)
    #[arg(long)]
    pub conflicts_only: bool,

    /// Extract version information from binaries
    #[arg(long, default_value_t = true)]
    pub extract_versions: bool,

    /// Resolve symbolic links
    #[arg(long, default_value_t = true)]
    pub resolve_symlinks: bool,

    /// Include file hash calculations (slower)
    #[arg(long)]
    pub include_hashes: bool,

    /// Use custom PATH instead of system PATH
    #[arg(long)]
    pub custom_path: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Show recommendations for resolving conflicts
    #[arg(long)]
    pub recommendations: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    JsonPretty,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CategoryFilter {
    WslVsWindows,
    VersionManagerVsSystem,
    MultipleVersionManagers,
    PackageManagerVsSystem,
    DuplicateVersions,
    ShadowedBinary,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SeverityFilter {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
