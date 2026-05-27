mod config;
mod py;
mod tg;
mod util;

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
        command: py::Commands,
    },
    /// Telegram utilities
    Tg {
        #[command(subcommand)]
        command: tg::Commands,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .target(env_logger::Target::Stderr)
        .init();

    let config = AppConfig::load(cli.config.as_deref())?;

    match cli.command {
        Commands::Py { command } => py::dispatch(&config.py, command),
        Commands::Tg { command } => tg::dispatch(&config.tg, command),
    }
}
