use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Information parsed from pyvenv.cfg.
pub struct VenvInfo {
    pub home: String,
    pub version: String,
    pub prompt: String,
}

/// BFS walk to find all Python virtual environments under `root`.
/// A venv is identified by the presence of a `pyvenv.cfg` file.
pub fn walk_dir_for_venv(root: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());

    while let Some(dir) = queue.pop_front() {
        let cfg = dir.join("pyvenv.cfg");
        if cfg.exists() {
            results.push(dir);
        } else {
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            queue.push_back(entry.path());
                        }
                    }
                }
                Err(e) => {
                    log::error!("Cannot read directory {}: {}", dir.display(), e);
                }
            }
        }
    }

    results
}

/// Parse pyvenv.cfg for metadata.
pub fn get_venv_info(prefix: &Path) -> Result<VenvInfo> {
    let cfg_path = prefix.join("pyvenv.cfg");
    let contents = fs::read_to_string(&cfg_path)?;

    let mut info = VenvInfo {
        home: String::new(),
        version: String::new(),
        prompt: String::new(),
    };

    for line in contents.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "home" => info.home = value.to_string(),
                "version" | "version_info" => {
                    if info.version.is_empty() {
                        info.version = value.to_string();
                    }
                }
                "prompt" => info.prompt = value.to_string(),
                _ => {}
            }
        }
    }

    Ok(info)
}
