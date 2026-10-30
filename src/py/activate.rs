use std::path::{Path, PathBuf};

use crate::util::shell::{ShellKind, detect_shell};

/// Generate the shell command to activate a venv.
pub fn gen_activate_cmd(env_path: &Path, shell: Option<ShellKind>) -> String {
    use ShellKind::*;

    #[cfg(target_os = "windows")]
    const DEFAULT_SHELL: ShellKind = Pwsh;
    #[cfg(not(target_os = "windows"))]
    const DEFAULT_SHELL: ShellKind = Sh;

    let shell = shell.or_else(detect_shell).unwrap_or(DEFAULT_SHELL);

    #[cfg(target_os = "windows")]
    let bin = env_path.join("Scripts");
    #[cfg(not(target_os = "windows"))]
    let bin = env_path.join("bin");

    let path_str = |p: PathBuf| p.display().to_string();

    match shell {
        Fish => format!(
            "source '{}'",
            path_str(bin.join("activate.fish"))
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
        ),
        // stdlib venv writes `Activate.ps1`, virtualenv writes `activate.ps1`.
        Pwsh => {
            let p = ["Activate.ps1", "activate.ps1"]
                .iter()
                .map(|name| bin.join(name))
                .find(|p| p.exists())
                .unwrap_or_else(|| bin.join("Activate.ps1"));
            format!(". '{}'", path_str(p).replace('\'', "''"))
        }
        Bash | Zsh | Sh => format!(
            ". '{}'",
            path_str(bin.join("activate")).replace('\'', "'\\''")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ShellKind::*;

    #[test]
    fn posix_quotes_path() {
        let p = Path::new("/a/$(x)'b\"c");
        assert_eq!(
            gen_activate_cmd(p, Some(Sh)),
            ". '/a/$(x)'\\''b\"c/bin/activate'"
        );
        assert!(gen_activate_cmd(p, Some(Bash)).starts_with(". '"));
    }

    #[test]
    fn fish_quotes_path() {
        let p = Path::new("/a/'b\\c");
        assert_eq!(
            gen_activate_cmd(p, Some(Fish)),
            "source '/a/\\'b\\\\c/bin/activate.fish'"
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn pwsh_quotes_path() {
        let p = Path::new("/nonexistent/a'b$x");
        assert_eq!(
            gen_activate_cmd(p, Some(Pwsh)),
            ". '/nonexistent/a''b$x/bin/Activate.ps1'"
        );
    }
}
