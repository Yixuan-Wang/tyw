use std::path::Path;

use crate::util::shell;

/// Generate the shell command to activate a venv.
pub fn gen_activate_cmd(env_path: &Path, shell_name: &str) -> String {
    let shell_name = if shell_name.is_empty() {
        shell::detect_shell().unwrap_or_else(|e| {
            log::warn!("Failed to detect shell, defaulting to sh-compatible: {e}");
            "sh".to_string()
        })
    } else {
        shell_name.to_string()
    };

    let bin = env_path.join("bin");
    let script = match shell_name.as_str() {
        "fish" => "activate.fish",
        "csh" | "tcsh" => "activate.csh",
        "nu" => "activate.nu",
        "powershell" | "pwsh" => "activate.ps1",
        _ => "activate",
    };

    let path = bin.join(script);
    format!("source \"{}\"", path.display())
}
