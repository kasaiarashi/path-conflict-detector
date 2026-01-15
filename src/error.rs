use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read PATH environment variable")]
    PathNotFound,

    #[error("Failed to access directory: {path}")]
    DirectoryAccessError { path: String },

    #[error("Failed to read file metadata: {path}")]
    MetadataError { path: String },

    #[error("Failed to resolve symbolic link: {path}")]
    SymlinkError { path: String },

    #[error("Circular symbolic link detected: {path}")]
    CircularSymlink { path: String },

    #[error("Version extraction failed for {binary}: {reason}")]
    VersionExtractionError { binary: String, reason: String },

    #[error("Platform detection failed: {reason}")]
    PlatformError { reason: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid path format: {path}")]
    InvalidPath { path: String },

    #[error("Timeout while extracting version from: {binary}")]
    TimeoutError { binary: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Failed to parse version string: {version}")]
    VersionParseError { version: String },

    #[error("Unsupported platform: {platform}")]
    UnsupportedPlatform { platform: String },

    #[error("Command execution failed: {command}")]
    CommandError { command: String },

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
