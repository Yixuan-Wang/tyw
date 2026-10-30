use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub struct FzfLine<V> {
    pub key: String,
    pub pretty: Vec<String>,
    pub raw: V,
}

/// Spawn fzf with the given choices and return the selected raw value.
pub fn fzf_select<V>(choices: Vec<FzfLine<V>>, extra_args: &[&str]) -> Result<Option<V>> {
    let mut args = vec!["--delimiter", "\t", "--with-nth", "2.."];
    args.extend_from_slice(extra_args);

    let mut fzf = Command::new("fzf")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn fzf")?;

    let mut mapping: HashMap<String, V> = HashMap::new();

    {
        let stdin = fzf.stdin.as_mut().context("Failed to open fzf stdin")?;
        for choice in choices {
            let pretty = choice.pretty.join(" ");
            writeln!(stdin, "{}\t{}", choice.key, pretty)?;
            mapping.insert(choice.key, choice.raw);
        }
    }

    let output = fzf.wait_with_output().context("Failed to run fzf")?;
    match output.status.code() {
        Some(0) => {}
        Some(1) | Some(130) => return Ok(None),
        _ => bail!("fzf exited with non-zero status"),
    }

    let selected = String::from_utf8_lossy(&output.stdout);
    let key = selected
        .trim_end_matches(['\n', '\r'])
        .split('\t')
        .next()
        .unwrap_or("");

    mapping
        .remove(key)
        .map(Some)
        .context("Selected key not found in mapping")
}
