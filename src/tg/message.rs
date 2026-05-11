use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct TgResponse<T> {
    #[allow(dead_code)]
    pub ok: bool,
    pub result: T,
}

#[derive(Deserialize)]
pub struct TgMessage {
    #[serde(rename = "message_id")]
    pub id: i64,
    pub date: i64,
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

    pub fn send_message(&self, text: &str) -> Result<TgMessage> {
        let url = self.bot_url("sendMessage");
        let body = json!({
            "chat_id": self.chat_id,
            "text": text,
        });

        log::debug!("Sending message: {text} to {url}");

        let resp: TgResponse<TgMessage> = ureq::post(&url)
            .send_json(&body)
            .context("Failed to send message")?
            .body_mut()
            .read_json()
            .context("Failed to decode sendMessage response")?;

        Ok(resp.result)
    }

    pub fn send_ping(&self, text: &str, transient: bool, timeout: Duration) -> Result<()> {
        let orig = self.send_message(text)?;
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

            let updates: Result<TgResponse<Vec<Value>>> = (|| {
                let resp = ureq::post(&self.bot_url("getUpdates"))
                    .send_json(&update_body)
                    .context("Failed to poll updates")?;
                let parsed = resp
                    .into_body()
                    .read_json()
                    .context("Failed to decode getUpdates response")?;
                Ok(parsed)
            })();

            match updates {
                Ok(resp) => {
                    for update in &resp.result {
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
                    log::warn!("Failed to poll updates: {e}");
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

        if let Err(e) = ureq::post(&self.bot_url("deleteMessages")).send_json(&body) {
            log::warn!("Failed to delete messages: {e}");
        }
    }
}
