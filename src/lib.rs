//! # seavan
//!
//! A library which wraps files in a container layer for later composition.

pub mod error;
pub mod utils;

use crate::{
    error::{SeavanError, SeavanResult},
    utils::docker_safe_string,
};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{ffi::OsStr, path::Path};

use log::{debug, info};
use sha2::Digest;

/// This value is a constant prefix for the generated image; this
/// makes it harder for people to use DockerHub for storage.
const PACKAGE_ROOT: &str = "seavanpkg";

pub struct WrappedLayer {
    path: PathBuf,
    tag: String,
}

impl WrappedLayer {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S, tag: Option<&str>) -> SeavanResult<Self> {
        // Store the canonical path.
        let path = Path::new(path);
        let canonical_path = std::fs::canonicalize(path)?;
        debug!("Wrapping path {}", canonical_path.display());

        // Store the docker-safe version of the tag.
        let safe_tag = docker_safe_string(tag.unwrap_or("latest"))?;

        Ok(Self {
            path: canonical_path,
            tag: safe_tag.into(),
        })
    }

    fn filename_str(&self) -> SeavanResult<&str> {
        let os_str = self
            .path
            .file_name()
            .ok_or_else(|| SeavanError::NoFileName(self.path.clone()))?;
        os_str.to_str().ok_or(SeavanError::FailedStrConversion)
    }

    fn working_directory(&self) -> SeavanResult<&Path> {
        self.path
            .parent()
            .ok_or_else(|| SeavanError::NoDirectory(self.path.clone()))
    }

    fn hash(&self) -> SeavanResult<String> {
        let mut file = std::fs::File::open(&self.path)?;
        let mut hasher = sha2::Sha256::new();
        let _ = std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    pub fn repository_name_and_tag(&self) -> SeavanResult<String> {
        let safe_filename = docker_safe_string(self.filename_str()?)?;
        Ok(format!(
            "{}/{}--{}:{}",
            PACKAGE_ROOT,
            self.hash()?,
            safe_filename,
            self.tag
        ))
    }

    /// Create a container image containing the wrapped file.
    ///
    /// TODO: Usage of NamedTempFile
    ///
    pub fn create_image(&self) -> SeavanResult<String> {
        let mut tempdocker =
            tempfile::NamedTempFile::new().map_err(|_| SeavanError::FailedTempFileCreation)?;

        // Write the template to the temporary file.
        write!(
            tempdocker,
            "FROM scratch\nCOPY {} /\n",
            self.filename_str()?
        )?;

        // Convert the temporary named file into a TempPath.
        let tempdockerpath = tempdocker.into_temp_path();
        info!("Created dockerfile {}", tempdockerpath.display());

        // Run docker to build the image.
        let repository_name_and_tag = self.repository_name_and_tag()?;
        let args = vec![
            "build",
            "-f",
            tempdockerpath
                .to_str()
                .ok_or(SeavanError::FailedStrConversion)?,
            "-t",
            &repository_name_and_tag,
            ".",
        ];

        let output = Command::new("docker")
            .args(args)
            .current_dir(self.working_directory()?)
            .output()?;

        // Check for command success
        match output.status.success() {
            true => {
                // Best effort debug logging for stdout
                if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                    debug!("Docker output: {}", stdout);
                }

                Ok(repository_name_and_tag)
            }
            false => {
                let stderr = String::from_utf8(output.stderr)
                    .map_err(|_| SeavanError::FailedStrConversion)?;
                Err(SeavanError::DockerBuildFailure(stderr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn clean_up_docker_image(image_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("docker").args(["rmi", image_name]).output()?;
        assert!(output.status.success());
        info!("Removed {}", image_name);
        Ok(())
    }

    #[test]
    fn wrap_cargo_toml() -> Result<(), Box<dyn std::error::Error>> {
        log_init();

        // Wrap Cargo.toml - it has capital letters and a fullstop.
        let wrap = WrappedLayer::new("Cargo.toml", Some("Some r4ndom t@g with character$"))?;
        let image_tag = wrap.create_image()?;
        info!("Created image {}", image_tag);

        // Clean up.
        clean_up_docker_image(&image_tag)?;
        Ok(())
    }
}
