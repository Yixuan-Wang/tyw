use std::collections::{HashSet, VecDeque};
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
    let mut visited = HashSet::new();
    queue.push_back(root.to_path_buf());

    while let Some(dir) = queue.pop_front() {
        let Ok(real) = fs::canonicalize(&dir)
            .inspect_err(|e| log::warn!("Cannot resolve directory {}: {}", dir.display(), e))
        else {
            continue;
        };
        if !visited.insert(real) {
            continue;
        }
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

fn unquote(value: &str) -> &str {
    for q in ['\'', '"'] {
        if let Some(inner) = value.strip_prefix(q).and_then(|v| v.strip_suffix(q)) {
            return inner;
        }
    }
    value
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
                "prompt" => info.prompt = unquote(value).to_string(),
                _ => {}
            }
        }
    }

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unquote_strips_matching_quotes() {
        assert_eq!(unquote("'name'"), "name");
        assert_eq!(unquote("\"name\""), "name");
        assert_eq!(unquote("name"), "name");
        assert_eq!(unquote("'name"), "'name");
    }

    #[cfg(unix)]
    #[test]
    fn walk_survives_symlink_loop() {
        let root = std::env::temp_dir().join(format!("tyw-loop-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("a/venv")).unwrap();
        fs::write(root.join("a/venv/pyvenv.cfg"), "").unwrap();
        std::os::unix::fs::symlink("..", root.join("a/loop")).unwrap();
        assert_eq!(walk_dir_for_venv(&root).len(), 1);
        let _ = fs::remove_dir_all(&root);
    }
}
