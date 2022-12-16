//! Error types for seavan

use std::path::PathBuf;

/// Types of error for seavan
#[derive(thiserror::Error, Debug)]
pub enum SeavanError {
    /// The given path has no filename. Check whether the path is correct.
    #[error("{0:?} has no filename")]
    NoFileName(PathBuf),

    /// The given path has no directory. Check whether the path is correct.
    #[error("{0:?} has no directory")]
    NoDirectory(PathBuf),

    /// A string conversion operation failed.
    #[error("Failed string conversion")]
    FailedStrConversion,

    /// There was a failure while calling Docker to build the image.
    #[error("Docker build failure: {0}")]
    DockerBuildFailure(String),

    /// Standard io error.
    #[error("io error")]
    IoError(#[from] std::io::Error),

    /// Error with safe string replacement
    #[error("regex error")]
    RegexError(#[from] regex::Error),
}

/// Result wrapper for `SeavanError`
pub type SeavanResult<T> = Result<T, SeavanError>;
