use std::path::Path;

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ShellKind {
    Sh,
    Bash,
    Zsh,
    Fish,
    #[value(alias = "powershell")]
    Pwsh,
}

/// Detect the shell of the parent process.
///
/// Checks the parent process name first, then falls back to $SHELL.
pub fn detect_shell() -> Option<ShellKind> {
    get_process_path(get_parent_id())
        .and_then(|path| parse_shell_name(&path))
        .or_else(|| parse_shell_name(&std::env::var("SHELL").ok()?))
}

fn get_parent_id() -> u32 {
    #[cfg(unix)]
    {
        std::os::unix::process::parent_id()
    }
    #[cfg(not(unix))]
    {
        0
    }
}

fn parse_shell_name(path: &str) -> Option<ShellKind> {
    let path = path.strip_suffix(" (deleted)").unwrap_or(path);
    let name = Path::new(path).file_name().and_then(|n| n.to_str())?;
    let name = name.strip_prefix('-').unwrap_or(name);
    ShellKind::from_str(name, false).ok()
}

fn get_process_path(pid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let path = std::fs::read_link(format!("/proc/{pid}/exe")).ok()?;
        Some(path.to_string_lossy().into_owned())
    }

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .ok()?;
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_names() {
        assert_eq!(
            parse_shell_name("/usr/bin/fish (deleted)"),
            Some(ShellKind::Fish)
        );
        assert_eq!(parse_shell_name("-zsh"), Some(ShellKind::Zsh));
        assert_eq!(parse_shell_name("/bin/bash"), Some(ShellKind::Bash));
        assert_eq!(parse_shell_name("/usr/bin/python"), None);
    }
}
