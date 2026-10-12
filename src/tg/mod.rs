mod message;

use std::time::Duration;

use anyhow::{Result, bail};
use clap::Subcommand;

use crate::config::TgConfig;
use message::{TextFormat, TgClient};

#[derive(Subcommand)]
pub enum Commands {
    /// Send a text message to a Telegram chat
    Text {
        /// Message to send (default: "Hello, world!")
        message: Option<String>,

        /// Text format of the message
        #[arg(short, long, value_enum, default_value_t = TextFormat::PlainText)]
        format: TextFormat,
    },
    /// Send a ping message and wait for response
    Ping {
        /// Message to send (default: "Heads up!")
        text: Option<String>,

        /// Text format of the message
        #[arg(short, long, value_enum, default_value_t = TextFormat::PlainText)]
        format: TextFormat,

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
        Commands::Text { message, format } => text(
            config,
            message.as_deref().unwrap_or("Hello, world!"),
            format,
        ),
        Commands::Ping {
            text,
            format,
            timeout,
        } => ping(
            config,
            text.as_deref().unwrap_or("Heads up!"),
            format,
            timeout,
        ),
    }
}

fn make_client(config: &TgConfig) -> Result<TgClient> {
    if config.chat_id.is_empty() {
        bail!("no chat_id found in config");
    }
    Ok(TgClient::new(&config.token, &config.chat_id))
}

pub fn text(config: &TgConfig, message: &str, format: TextFormat) -> Result<()> {
    let client = make_client(config)?;
    client.send_message(message, format)?;
    Ok(())
}

pub fn ping(config: &TgConfig, message: &str, format: TextFormat, timeout: Duration) -> Result<()> {
    let client = make_client(config)?;
    client.send_ping(message, format, true, timeout)?;
    Ok(())
}
