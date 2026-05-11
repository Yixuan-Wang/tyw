mod activate;
mod env;

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::config::PyConfig;
use crate::util::fzf::{self, FzfLine};

pub fn list(config: &PyConfig) -> Result<()> {
    let env_home = Path::new(&config.env.home);

    if !env_home.is_dir() {
        log::error!(
            "Path does not exist or is not a directory: {}",
            env_home.display()
        );
        return Ok(());
    }

    for dir in env::walk_dir_for_venv(env_home) {
        println!("{}", dir.display());
    }

    Ok(())
}

pub fn use_env(config: &PyConfig, name: Option<&str>) -> Result<()> {
    match name {
        Some(name) => use_named_env(config, name),
        None => try_use_env(),
    }
}

fn use_named_env(config: &PyConfig, name: &str) -> Result<()> {
    if name.is_empty() {
        log::error!("Environment name is empty");
        return Ok(());
    }

    let env_home = Path::new(&config.env.home);
    let env_path = env_home.join(name);

    if !env_path.is_dir() {
        log::error!(
            "Path does not exist or is not a directory: {}",
            env_path.display()
        );
        return Ok(());
    }

    for suffix in ["", ".venv", "venv"] {
        let variation = if suffix.is_empty() {
            env_path.clone()
        } else {
            env_path.join(suffix)
        };
        if variation.join("pyvenv.cfg").exists() {
            print!("{}", activate::gen_activate_cmd(&variation, ""));
            return Ok(());
        }
    }

    log::error!("Environment does not exist: {name}");
    Ok(())
}

fn try_use_env() -> Result<()> {
    let cwd = std::env::current_dir()?;

    if !cwd.is_dir() {
        bail!(
            "Path does not exist or is not a directory: {}",
            cwd.display()
        );
    }

    let mut dir = cwd;
    loop {
        for subdir in ["venv", ".venv"] {
            let candidate = dir.join(subdir);
            if candidate.join("pyvenv.cfg").exists() {
                println!("{}", activate::gen_activate_cmd(&candidate, ""));
                return Ok(());
            }
        }

        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => bail!("No virtual environment found in the directory tree"),
        }
    }
}

pub fn select_env(config: &PyConfig) -> Result<()> {
    let env_home = PathBuf::from(&config.env.home);

    if !env_home.is_dir() {
        log::error!(
            "Environment home does not exist or is not a directory: {}",
            env_home.display()
        );
        return Ok(());
    }

    let venvs = env::walk_dir_for_venv(&env_home);

    let choices: Vec<FzfLine<PathBuf>> = venvs
        .into_iter()
        .filter_map(|path| {
            let rel_path = path.strip_prefix(&env_home).ok()?.to_path_buf();
            let info = env::get_venv_info(&path).ok()?;
            let name = rel_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let rel_str = rel_path.to_string_lossy().to_string();

            let pretty = if !info.prompt.is_empty() && info.prompt != name {
                vec![format!("{}({})", info.prompt, rel_str), info.version]
            } else {
                vec![name, info.version]
            };

            Some(FzfLine {
                key: rel_str,
                pretty,
                raw: path,
            })
        })
        .collect();

    match fzf::fzf_select(choices, &[]) {
        Ok(selected) => {
            print!("{}", activate::gen_activate_cmd(&selected, ""));
        }
        Err(e) => {
            log::error!("Failed to select environment: {e}");
        }
    }

    Ok(())
}
