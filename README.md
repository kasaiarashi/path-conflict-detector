# Path Conflict Detector

A powerful Rust tool to detect and analyze PATH environment variable conflicts across Windows, Linux, and macOS platforms. Identifies duplicate binaries, version conflicts, and path mixing issues (especially WSL/Windows conflicts).

## Features

- **Cross-Platform Support**: Works on Windows, Linux, and macOS
- **WSL Integration**: Special detection for WSL/Windows path mixing
- **Version Detection**: Automatically extracts version information from binaries
- **Manager Detection**: Identifies version managers (nvm, pyenv, rustup) and package managers (Homebrew, Chocolatey)
- **Conflict Categorization**: Classifies conflicts by type and severity
- **Multiple Output Formats**: Human-readable colored output or JSON
- **Symlink Resolution**: Follows symbolic links to find actual binaries
- **Detailed Analysis**: Provides recommendations for resolving conflicts

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/path-conflict-detector.git
cd path-conflict-detector

# Build with Cargo
cargo build --release

# The binary will be at target/release/path-conflict-detector
```

### Using Cargo

```bash
cargo install path-conflict-detector
```

## Usage

### Basic Analysis

Run a full PATH analysis:

```bash
path-conflict-detector
```

### Output Formats

Human-readable output (default):
```bash
path-conflict-detector
```

JSON output:
```bash
path-conflict-detector --json
```

Pretty JSON:
```bash
path-conflict-detector --output json-pretty
```

### Filtering

Check specific binary:
```bash
path-conflict-detector --binary python
```

Filter by severity:
```bash
path-conflict-detector --severity high
```

Filter by category:
```bash
path-conflict-detector --category wsl-vs-windows
```

Show only conflicts:
```bash
path-conflict-detector --conflicts-only
```

### Options

```
--output <FORMAT>        Output format: human, json, json-pretty
--json                   Use JSON output (shorthand for --output json)
--binary <NAME>          Check specific binary name
--category <CATEGORY>    Filter by conflict category
--severity <LEVEL>       Filter by minimum severity level
--conflicts-only         Show only conflicts
--extract-versions       Extract version information (default: true)
--resolve-symlinks       Resolve symbolic links (default: true)
--include-hashes         Include file hashes (slower)
--custom-path <PATH>     Use custom PATH instead of system PATH
--verbose                Verbose output
--quiet                  Quiet mode (minimal output)
--recommendations        Show recommendations for resolving conflicts
```

## Conflict Categories

- **WSL vs Windows**: Conflicts between WSL and Windows binaries
- **Version Manager vs System**: Version manager (nvm, pyenv) vs system installation
- **Multiple Version Managers**: Same binary managed by different tools
- **Package Manager vs System**: Package manager (Homebrew, Chocolatey) vs system
- **Duplicate Versions**: Multiple versions of the same binary
- **Shadowed Binary**: Binary hidden by earlier PATH entry

## Severity Levels

- **Critical**: Active binary shadowed by different major version
- **High**: WSL/Windows mixing or significant version differences
- **Medium**: Multiple version managers or minor conflicts
- **Low**: Different package managers with same version
- **Info**: Symlinks pointing to same binary

## Examples

### Example Output

```
PATH Conflict Analysis Report
═════════════════════════════
Platform: Windows 11 (with WSL2)
Scan Time: 2026-01-15 14:30:45 UTC

SUMMARY
────────────────────────────
Total PATH Entries: 42
Total Executables: 156
Unique Executables: 134
Conflicts Found: 12

CONFLICTS BY CATEGORY
────────────────────────────
🔴 WSL vs Windows (4)
🟡 Version Manager vs System (5)
🔵 Duplicate Versions (3)

DETAILED CONFLICTS
══════════════════

[1] 🔴 CRITICAL: python (WSL vs Windows)
────────────────────────────────────────
Active: /usr/bin/python → Python 3.10.12
   Path: /usr/bin/python
   Type: System (apt)

Shadowed instances:
   [2] C:\Python311\python.exe → Python 3.11.7
       Type: System (Windows)

Recommendation: Consider using pyenv for version management
```

### Common Scenarios

**Finding Python conflicts:**
```bash
path-conflict-detector --binary python --recommendations
```

**Checking WSL issues:**
```bash
path-conflict-detector --category wsl-vs-windows
```

**Quick check for critical issues:**
```bash
path-conflict-detector --severity critical --conflicts-only
```

**Full analysis with JSON output:**
```bash
path-conflict-detector --json > path-analysis.json
```

## Library Usage

The tool can also be used as a Rust library:

```rust
use path_conflict_detector::{PathAnalyzer, AnalysisOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create analyzer with default options
    let analyzer = PathAnalyzer::new();

    // Run analysis
    let result = analyzer.analyze()?;

    // Access conflicts
    for conflict in &result.conflicts {
        println!("Conflict: {} ({:?})", conflict.binary_name, conflict.severity);
    }

    // Check specific binary
    let python_instances = analyzer.check_binary("python")?;
    println!("Found {} Python installations", python_instances.len());

    Ok(())
}
```

## Architecture

The tool is organized into several modules:

- **core**: PATH parsing, executable scanning, conflict detection
- **platform**: Platform-specific logic (Windows, WSL, Unix, macOS)
- **analyzers**: Version extraction, symlink resolution, manager detection
- **output**: Formatting (human-readable, JSON)
- **cli**: Command-line interface

## Development

### Running Tests

```bash
cargo test
```

### Building

Debug build:
```bash
cargo build
```

Release build (optimized):
```bash
cargo build --release
```

### Code Style

```bash
cargo fmt
cargo clippy
```

### Releasing

To create a new release:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Commit changes:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Bump version to X.Y.Z"
   ```
4. Create and push a tag:
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin master --tags
   ```
5. GitHub Actions will automatically:
   - Build binaries for Linux, Windows, macOS (Intel & ARM)
   - Create a GitHub release
   - Attach binaries to the release

## Supported Platforms

- **Windows**: Native Windows executables, WSL integration
- **Linux**: Standard Linux installations, package managers
- **macOS**: Homebrew, system installations, framework bundles

## Supported Version Managers

- **nvm**: Node Version Manager
- **pyenv**: Python Version Manager
- **rbenv**: Ruby Version Manager
- **rustup**: Rust Toolchain Manager
- **asdf**: Multiple Runtime Version Manager
- **sdkman**: Software Development Kit Manager

## Supported Package Managers

- **Homebrew**: macOS/Linux package manager
- **Chocolatey**: Windows package manager
- **Scoop**: Windows package manager

## Performance Considerations

- Version extraction can be slow for large PATHs (runs each binary with `--version`)
- Use `--include-hashes` sparingly as it adds overhead
- Consider using `--binary` to check specific binaries instead of full PATH scan

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Troubleshooting

### Tool is slow

- Try without version extraction: the tool doesn't support disabling this yet via CLI, but you can modify the defaults
- Use `--binary` to check specific binaries
- Some binaries may hang during version detection

### WSL not detected

- Ensure you're running inside WSL
- Check that `/proc/version` contains "Microsoft" or "WSL"
- Verify `WSL_DISTRO_NAME` environment variable is set

### Permissions errors

- Some PATH directories may require elevated permissions
- The tool will skip inaccessible directories and continue

## Future Enhancements

- Interactive TUI mode
- Auto-fix suggestions with PATH reorganization
- Watch mode for monitoring PATH changes
- HTML/Markdown report exports
- Configuration file support
- CI/CD integration
