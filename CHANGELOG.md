# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-15

### Added
- Initial release of path-conflict-detector
- Cross-platform support for Windows, Linux, and macOS
- PATH environment variable parsing and analysis
- Executable conflict detection with categorization:
  - WSL vs Windows conflicts
  - Version Manager vs System installations
  - Multiple Version Managers
  - Package Manager vs System
  - Duplicate Versions
  - Shadowed Binaries
- Severity assessment (Critical, High, Medium, Low, Info)
- Multiple output formats:
  - Human-readable colored terminal output
  - JSON (compact and pretty-printed)
- CLI options:
  - `--binary` - Check specific binary
  - `--category` - Filter by conflict category
  - `--severity` - Filter by severity level
  - `--conflicts-only` - Show only conflicts
  - `--extract-versions` - Extract version information (opt-in)
  - `--resolve-symlinks` - Resolve symbolic links
  - `--include-hashes` - Include file hashes
  - `--custom-path` - Analyze custom PATH
  - `--recommendations` - Show conflict resolution recommendations
  - `--verbose` / `--quiet` - Control output verbosity
  - `--json` - JSON output format
- Platform-specific features:
  - WSL detection and integration
  - Windows system directory filtering
  - Homebrew path detection (macOS)
  - Version manager detection (nvm, pyenv, rbenv, rustup, asdf, sdkman)
  - Package manager detection (Homebrew, Chocolatey, Scoop)
- Symlink resolution with cycle detection
- Manager and package detection
- Comprehensive executable blacklist (150+ Windows system utilities)
- GitHub Actions automated release workflow
- Cross-platform binary releases (Linux, Windows, macOS Intel & ARM)

### Features
- Works as both CLI tool and Rust library
- Intelligent conflict categorization
- Actionable recommendations for resolving conflicts
- Graceful error handling
- Fast scanning with system directory skipping
- Version extraction disabled by default for safety (opt-in with --extract-versions)

### Fixed
- GUI window prevention on Windows using CREATE_NO_WINDOW flag
- Stdin closing to prevent process hanging
- Usage message filtering to avoid noise
- System directory skipping to improve performance and prevent issues

### Security
- Comprehensive blacklist of problematic executables
- Safe process execution with proper flags
- Input validation and path sanitization
- Symlink cycle detection

[0.1.0]: https://github.com/kasaiarashi/path-conflict-detector/releases/tag/v0.1.0
