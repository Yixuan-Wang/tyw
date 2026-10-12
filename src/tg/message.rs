use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct TgResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct TgMessage {
    #[serde(rename = "message_id")]
    pub id: i64,
    pub date: i64,
}

/// Text formatting of a message, mapped to Telegram's `parse_mode`.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum TextFormat {
    /// Plain text, no formatting
    #[default]
    PlainText,
    /// Telegram's MarkdownV2 format 
    Markdown,
    /// Telegram's HTML format
    Html,
}

impl TextFormat {
    fn parse_mode(self) -> Option<&'static str> {
        match self {
            TextFormat::PlainText => None,
            TextFormat::Markdown => Some("MarkdownV2"),
            TextFormat::Html => Some("HTML"),
        }
    }
}

pub struct TgClient {
    token: String,
    pub chat_id: String,
}

impl TgClient {
    pub fn new(token: &str, chat_id: &str) -> Self {
        Self {
            token: token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }

    fn bot_url(&self, path: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, path)
    }

    /// Calls a Bot API method, turning `ok: false` responses into errors with
    /// Telegram's `description`.
    fn call<T: DeserializeOwned>(&self, method: &str, body: &Value) -> Result<T> {
        let resp: TgResponse<T> = ureq::post(&self.bot_url(method))
            .config()
            .http_status_as_error(false)
            .build()
            .send_json(body)
            .with_context(|| format!("Failed to call {method}"))?
            .body_mut()
            .read_json()
            .with_context(|| format!("Failed to decode {method} response"))?;

        if !resp.ok {
            let description = resp.description.as_deref().unwrap_or("unknown error");
            return Err(anyhow!("Telegram {method} failed: {description}"));
        }
        resp.result
            .with_context(|| format!("Telegram {method} returned no result"))
    }

    pub fn send_message(&self, text: &str, format: TextFormat) -> Result<TgMessage> {
        let url = self.bot_url("sendMessage");
        let mut body = json!({
            "chat_id": self.chat_id,
            "text": text,
        });
        if let Some(parse_mode) = format.parse_mode() {
            body["parse_mode"] = json!(parse_mode);
        }

        log::debug!("Sending message: {text} to {url}");

        self.call("sendMessage", &body)
            .context("Failed to send message")
    }

    pub fn send_ping(
        &self,
        text: &str,
        format: TextFormat,
        transient: bool,
        timeout: Duration,
    ) -> Result<()> {
        let orig = self.send_message(text, format)?;
        let message_id = orig.id;
        let message_sent_time = orig.date;

        let mut offset: i64 = -1;
        let mut cleanup = vec![message_id];
        let deadline = Instant::now() + timeout;

        loop {
            if Instant::now() > deadline {
                eprintln!("Timeout after {}s", timeout.as_secs());
                std::process::exit(1);
            }

            let update_body = json!({
                "allowed_updates": ["message", "message_reaction"],
                "offset": offset,
            });

            match self.call::<Vec<Value>>("getUpdates", &update_body) {
                Ok(updates) => {
                    for update in &updates {
                        // Check for reaction on our message
                        if let Some(reaction) = update.get("message_reaction")
                            && let Some(mid) = reaction.get("message_id")
                            && mid.as_i64() == Some(message_id)
                        {
                            log::debug!("Received pong by reaction");
                            if transient {
                                self.delete_messages(&cleanup);
                            }
                            return Ok(());
                        }

                        // Check for new message after our sent time
                        if let Some(msg) = update.get("message")
                            && let Some(date) = msg.get("date").and_then(|d| d.as_i64())
                            && date > message_sent_time
                        {
                            log::debug!("Received pong by message");
                            if let Some(mid) = msg.get("message_id").and_then(|m| m.as_i64()) {
                                cleanup.push(mid);
                            }
                            if transient {
                                self.delete_messages(&cleanup);
                            }
                            return Ok(());
                        }

                        // Advance offset
                        if let Some(uid) = update.get("update_id").and_then(|u| u.as_i64()) {
                            offset = uid + 1;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to poll updates: {e:#}");
                }
            }

            log::debug!("Waiting for message update for message_id={message_id}");
            thread::sleep(Duration::from_secs(5));
        }
    }

    fn delete_messages(&self, message_ids: &[i64]) {
        let body = json!({
            "chat_id": self.chat_id,
            "message_ids": message_ids,
        });

        if let Err(e) = self.call::<bool>("deleteMessages", &body) {
            log::warn!("Failed to delete messages: {e:#}");
        }
    }
}
