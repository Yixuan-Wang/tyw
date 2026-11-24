use humantime;
use std::time::Duration;

/// Parses a human-readable duration string into a [`Duration`] object.
///
/// Check [`humantime::parse_duration`] for supported formats.
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

/// Asserts a non-empty string.
pub fn parse_non_empty_str(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("option must not be empty".into());
    }
    Ok(s.to_string())
}
