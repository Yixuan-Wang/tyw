use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

const KNOWN_SHELLS: &[&str] = &[
    "sh", "bash", "zsh", "fish", "dash", "ksh", "csh", "tcsh", "nu",
];

/// Detect the shell of the parent process.
///
/// Checks the parent process name first, then falls back to $SHELL.
pub fn detect_shell() -> Result<String> {
    let ppid = parent_id();

    if let Ok(parent_path) = get_process_path(ppid)
        && !parent_path.is_empty()
    {
        let name = Path::new(&parent_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if KNOWN_SHELLS.contains(&name) {
            return Ok(name.to_string());
        }
    }

    // Fallback to $SHELL
    if let Ok(shell) = std::env::var("SHELL")
        && !shell.is_empty()
    {
        let name = Path::new(&shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        return Ok(name.to_string());
    }

    bail!("could not detect parent shell and $SHELL is unset")
}

fn parent_id() -> u32 {
    #[cfg(unix)]
    {
        std::os::unix::process::parent_id()
    }
    #[cfg(not(unix))]
    {
        0
    }
}

fn get_process_path(pid: u32) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        let link = format!("/proc/{pid}/exe");
        let path = std::fs::read_link(&link)?;
        Ok(path.to_string_lossy().into_owned())
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = pid;
        bail!("unsupported platform for process lookup")
    }
}
