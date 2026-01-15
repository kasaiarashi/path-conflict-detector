pub mod macos;
pub mod unix;
pub mod windows;
pub mod wsl;

use crate::error::{Error, Result};
use crate::output::types::PlatformInfo;
use std::path::Path;

pub fn detect_platform() -> Result<PlatformInfo> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let (is_wsl, wsl_version, wsl_distro) = if cfg!(target_os = "linux") {
        wsl::detect_wsl()?
    } else {
        (false, None, None)
    };

    Ok(PlatformInfo {
        os,
        arch,
        is_wsl,
        wsl_version,
        wsl_distro,
    })
}

pub fn get_path_separator() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

pub fn get_path_env_var() -> Result<String> {
    std::env::var("PATH").map_err(|_| Error::PathNotFound)
}

pub fn is_executable(path: &Path) -> bool {
    if cfg!(windows) {
        windows::is_executable_windows(path)
    } else {
        unix::is_executable_unix(path)
    }
}

pub fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();

    #[cfg(windows)]
    {
        result = windows::expand_windows_env_vars(&result);
    }

    #[cfg(unix)]
    {
        result = unix::expand_unix_env_vars(&result);
    }

    result
}
