//! # seavan
//!
//! A library which wraps files in a container layer for later composition.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub mod error;
pub mod utils;

use crate::{
    error::{SeavanError, SeavanResult},
    utils::docker_safe_string,
};
use std::io::Write;
use std::process::Command;
use std::{ffi::OsStr, path::Path};
use std::{io::Seek, path::PathBuf};

use log::debug;
use sha2::Digest;
use tempfile::tempfile;

/// This value is a constant prefix for the generated image; this
/// makes it harder for people to use DockerHub for storage.
const PACKAGE_ROOT: &str = "seavanpkg";

/// A structure representing a file wrapped in a Docker container shell.
pub struct Seavan {
    path: PathBuf,
    tag: String,
}

impl Seavan {
    /// Creates a new `Seavan`. The repository name will be automatically
    /// derived from the file's name.
    ///
    /// # Arguments
    ///
    /// * `path`: The file path to be wrapped in a Docker container shell
    /// * `tag`: An optional tag. If not set, `latest` is used.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use seavan::Seavan;
    /// let wrap = Seavan::new("README.md", None)?;
    ///
    /// /// This creates the image using Docker. The user must be able to run
    /// /// Docker commands.
    /// let repo_name_and_tag = wrap.create_image()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
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

    // Helper method to get a &str version of the file's basename.
    fn filename_str(&self) -> SeavanResult<&str> {
        let os_str = self
            .path
            .file_name()
            .ok_or_else(|| SeavanError::NoFileName(self.path.clone()))?;
        os_str.to_str().ok_or(SeavanError::FailedStrConversion)
    }

    // Helper method to get a &Path version of the file's parent directory.
    fn working_directory(&self) -> SeavanResult<&Path> {
        self.path
            .parent()
            .ok_or_else(|| SeavanError::NoDirectory(self.path.clone()))
    }

    // Helper method to get a sha hash of the file contents.
    fn hash(&self) -> SeavanResult<String> {
        let mut file = std::fs::File::open(&self.path)?;
        let mut hasher = sha2::Sha256::new();
        let _ = std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Returns the generated repository name and tag for the container image.
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

    /// Creates a container image containing the wrapped file.
    /// This creates the image using a Docker command. The user must be able to
    /// run Docker commands by running `docker`.
    ///
    /// Returns the generated repository name and tag for the container image.
    ///
    pub fn create_image(&self) -> SeavanResult<String> {
        // Use the standard tempfile for security.
        let mut tempdocker = tempfile()?;

        // Write the template to the temporary file, then rewind.
        write!(
            tempdocker,
            "FROM scratch\nCOPY {} /\n",
            self.filename_str()?
        )?;
        tempdocker.rewind()?;

        // Run docker to build the image.
        //
        // Enable docker buildkit for faster builds
        // Pass in the file as stdin due to https://github.com/docker/cli/issues/2249
        // and because it doesn't require us to pass in a path.
        let repository_name_and_tag = self.repository_name_and_tag()?;

        let output = Command::new("docker")
            .stdin(tempdocker)
            .args(["build", "-f", "-", "-t", &repository_name_and_tag, "."])
            .env("DOCKER_BUILDKIT", "1")
            .current_dir(self.working_directory()?)
            .output()?;

        // Check for command success!
        match output.status.success() {
            true => {
                // Best effort debug logging for stdout
                if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                    debug!("Docker output: {}", stdout);
                }
                // Best effort debug logging for stderr - buildkit prints out
                // to stderr rather than stdin.
                if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                    debug!("Docker stderr: {}", stderr);
                }

                // Return the name of the created repository name and tag.
                Ok(repository_name_and_tag)
            }
            false => {
                let stderr = String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "No Docker stderr".to_string());

                Err(SeavanError::DockerBuildFailure(stderr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;

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
        let wrap = Seavan::new("Cargo.toml", Some("Some r4ndom t@g with character$"))?;
        let image_tag = wrap.create_image()?;
        info!("Created image {}", image_tag);

        // Clean up.
        clean_up_docker_image(&image_tag)?;
        Ok(())
    }
}
