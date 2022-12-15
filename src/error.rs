use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum SeavanError {
    #[error("{0:?} has no filename")]
    NoFileName(PathBuf),

    #[error("{0:?} has no directory")]
    NoDirectory(PathBuf),

    #[error("Failed string conversion")]
    FailedStrConversion,

    #[error("Failed to create temporary file")]
    FailedTempFileCreation,

    #[error("Docker build failure: {0}")]
    DockerBuildFailure(String),

    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("regex error")]
    RegexError(#[from] regex::Error),
}

pub type SeavanResult<T> = Result<T, SeavanError>;
