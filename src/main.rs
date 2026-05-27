mod config;
mod py;
mod tg;
mod util;

use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{OffLevel, Verbosity};

use config::AppConfig;

#[derive(Parser)]
#[command(name = "tyw", about = "Yixuan \"Tom\" Wang's command line utilities")]
struct Cli {
    /// Config file (default: $XDG_CONFIG_HOME/tyw.toml)
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(flatten)]
    verbose: Verbosity<OffLevel>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Python utilities
    Py {
        #[command(subcommand)]
        command: PyCommands,
    },
    /// Telegram utilities
    Tg {
        #[command(subcommand)]
        command: TgCommands,
    },
}

#[derive(Subcommand)]
enum PyCommands {
    /// List Python virtual environments
    List,
    /// Use a Python virtual environment
    Use {
        /// Environment name (omit to auto-detect from cwd)
        name: Option<String>,
    },
    /// Select and use a Python virtual environment
    Sel,
}

#[derive(Subcommand)]
enum TgCommands {
    /// Send a text message to a Telegram chat
    Text {
        /// Message to send (default: "Hello, world!")
        message: Option<String>,
    },
    /// Send a ping message and wait for response
    Ping {
        /// Message to send (default: "Heads up!")
        text: Option<String>,

        /// Duration to wait before timeout (e.g. "6h", "30m", "1h30m")
        #[arg(short, long, default_value = "6h", value_parser = parse_duration)]
        timeout: Duration,
    },
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    let mut total_secs: u64 = 0;
    let mut current_num = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            let n: u64 = current_num
                .parse()
                .map_err(|_| format!("invalid duration: {s}"))?;
            current_num.clear();
            match c {
                'h' => total_secs += n * 3600,
                'm' => total_secs += n * 60,
                's' => total_secs += n,
                _ => return Err(format!("unknown duration unit: {c}")),
            }
        }
    }

    // Handle bare number (treat as seconds)
    if !current_num.is_empty() {
        let n: u64 = current_num
            .parse()
            .map_err(|_| format!("invalid duration: {s}"))?;
        total_secs += n;
    }

    if total_secs == 0 {
        return Err(format!("invalid duration: {s}"));
    }

    Ok(Duration::from_secs(total_secs))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .target(env_logger::Target::Stderr)
        .init();

    let config = AppConfig::load(cli.config.as_deref())?;

    match cli.command {
        Commands::Py { command } => match command {
            PyCommands::List => py::list(&config.py),
            PyCommands::Use { name } => py::use_env(&config.py, name.as_deref()),
            PyCommands::Sel => py::select_env(&config.py),
        },
        Commands::Tg { command } => match command {
            TgCommands::Text { message } => {
                let msg = message.as_deref().unwrap_or("Hello, world!");
                tg::text(&config.tg, msg)
            }
            TgCommands::Ping { text, timeout } => {
                let msg = text.as_deref().unwrap_or("Heads up!");
                tg::ping(&config.tg, msg, timeout)
            }
        },
    }
}
