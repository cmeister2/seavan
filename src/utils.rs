//! Useful utilities for seavan

use crate::error::SeavanError;
use regex::Captures;
use std::borrow::Cow;

// Converts a string into a "docker-safe" string; replacing all upper-case with
// lower-case, and all other bad values with -.
pub(crate) fn docker_safe_string(input: &str) -> Result<Cow<str>, SeavanError> {
    let re = regex::Regex::new("([^a-z0-9-_]+)")?;
    Ok(re.replace_all(input, |caps: &Captures| {
        let cap = &caps[0];
        cap.chars()
            .map(|c| match c {
                'A'..='Z' => c.to_ascii_lowercase(),
                _ => '-',
            })
            .collect::<String>()
    }))
}
