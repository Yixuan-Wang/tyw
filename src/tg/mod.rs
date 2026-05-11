mod message;

use std::time::Duration;

use anyhow::{Result, bail};

use crate::config::TgConfig;
use message::TgClient;

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
