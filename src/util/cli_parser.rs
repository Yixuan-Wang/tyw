use std::time::Duration;

use jiff::SignedDuration;

/// Parses a human-readable duration string into a [`Duration`] object.
///
/// Check [`jiff::fmt::friendly`] for supported formats.
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let d: SignedDuration = s.parse().map_err(|e: jiff::Error| e.to_string())?;
    Duration::try_from(d).map_err(|e| e.to_string())
}

/// Asserts a non-empty string.
pub fn parse_non_empty_str(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("option must not be empty".into());
    }
    Ok(s.to_string())
}
