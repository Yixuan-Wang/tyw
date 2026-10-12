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

        /// Delete the ping and its reply once a response is received
        #[arg(
            long,
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
            require_equals = true
        )]
        transient: bool,
    },
    /// Send a message and print the text reply
    Ask {
        /// Message to send (default: "Waiting for your reply:")
        text: Option<String>,

        /// Text format of the message
        #[arg(short, long, value_enum, default_value_t = TextFormat::PlainText)]
        format: TextFormat,

        /// Duration to wait before timeout (e.g. "6h", "30m", "1h30m")
        #[arg(short, long, default_value = "6h", value_parser = parse_duration)]
        timeout: Duration,
    },
    /// Send a message with buttons and print the option picked
    Pick {
        /// Message to send (default: "Pick one:")
        text: Option<String>,

        /// Option to pick from, shown as a button (repeatable)
        #[arg(
            short,
            long = "option",
            value_name = "OPTION",
            required_unless_present = "confirm",
            value_parser = parse_option
        )]
        options: Vec<String>,

        /// Shorthand for `--option yes --option no`
        #[arg(short = 'C', long, conflicts_with = "options")]
        confirm: bool,

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

fn parse_option(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("option must not be empty".into());
    }
    Ok(s.to_string())
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
            transient,
        } => ping(
            config,
            text.as_deref().unwrap_or("Heads up!"),
            format,
            transient,
            timeout,
        ),
        Commands::Ask {
            text,
            format,
            timeout,
        } => ask(
            config,
            text.as_deref().unwrap_or("Waiting for your reply:"),
            format,
            timeout,
        ),
        Commands::Pick {
            text,
            options,
            confirm,
            format,
            timeout,
        } => {
            let options = if confirm {
                vec!["yes".to_string(), "no".to_string()]
            } else {
                options
            };
            pick(
                config,
                text.as_deref().unwrap_or("Pick one:"),
                format,
                &options,
                timeout,
            )
        }
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
    client.send_message(message, format, None)?;
    Ok(())
}

pub fn ping(
    config: &TgConfig,
    message: &str,
    format: TextFormat,
    transient: bool,
    timeout: Duration,
) -> Result<()> {
    let client = make_client(config)?;
    client.send_ping(message, format, transient, timeout)?;
    Ok(())
}

pub fn ask(config: &TgConfig, message: &str, format: TextFormat, timeout: Duration) -> Result<()> {
    let client = make_client(config)?;
    let answer = client.send_ask(message, format, timeout)?;
    println!("{answer}");
    Ok(())
}

pub fn pick(
    config: &TgConfig,
    message: &str,
    format: TextFormat,
    options: &[String],
    timeout: Duration,
) -> Result<()> {
    let client = make_client(config)?;
    let choice = client.send_pick(message, format, options, timeout)?;
    println!("{choice}");
    Ok(())
}
