mod message;

use std::time::Duration;

use anyhow::{Result, bail};
use clap::Subcommand;

use crate::config::TgConfig;
use message::TgClient;

#[derive(Subcommand)]
pub enum Commands {
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
    humantime::parse_duration(s).map_err(|e| e.to_string())
}

pub fn dispatch(config: &TgConfig, command: Commands) -> Result<()> {
    match command {
        Commands::Text { message } => text(config, message.as_deref().unwrap_or("Hello, world!")),
        Commands::Ping { text, timeout } => {
            ping(config, text.as_deref().unwrap_or("Heads up!"), timeout)
        }
    }
}

fn make_client(config: &TgConfig) -> Result<TgClient> {
    if config.chat_id.is_empty() {
        bail!("no chat_id found in config");
    }
    Ok(TgClient::new(&config.token, &config.chat_id))
}

pub fn text(config: &TgConfig, message: &str) -> Result<()> {
    let client = make_client(config)?;
    client.send_message(message)?;
    Ok(())
}

pub fn ping(config: &TgConfig, message: &str, timeout: Duration) -> Result<()> {
    let client = make_client(config)?;
    client.send_ping(message, true, timeout)?;
    Ok(())
}
