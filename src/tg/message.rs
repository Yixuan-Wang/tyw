use std::ops::ControlFlow::{self, Break, Continue};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// Upper bound of a single `getUpdates` long poll.
const LONG_POLL_TIMEOUT: Duration = Duration::from_secs(30);
/// Extra time allowed on top of the long poll before the HTTP request is abandoned.
const LONG_POLL_SLACK: Duration = Duration::from_secs(10);
/// Delay before retrying after a failed poll.
const RETRY_DELAY: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct TgResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
    error_code: Option<i64>,
}

/// An `ok: false` response from the Bot API.
#[derive(Debug)]
struct TgApiError {
    method: String,
    code: Option<i64>,
    description: String,
}

impl std::fmt::Display for TgApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Telegram {} failed: {}", self.method, self.description)
    }
}

impl std::error::Error for TgApiError {}

impl TgApiError {
    /// Whether the request may succeed if retried, i.e. rate limiting or a
    /// server-side error.
    fn is_transient(&self) -> bool {
        matches!(self.code, Some(429 | 500..))
    }
}

#[derive(Deserialize)]
struct TgChat {
    id: i64,
}

#[derive(Deserialize)]
pub struct TgMessage {
    #[serde(rename = "message_id")]
    id: i64,
    chat: TgChat,
}

/// Text formatting of a message, mapped to Telegram's `parse_mode`.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum TextFormat {
    /// Plain text, no formatting
    #[default]
    PlainText,
    /// Telegram's `MarkdownV2` format
    Markdown,
    /// Telegram's HTML format
    Html,
}

impl TextFormat {
    fn get_parse_mode(self) -> Option<&'static str> {
        match self {
            TextFormat::PlainText => None,
            TextFormat::Markdown => Some("MarkdownV2"),
            TextFormat::Html => Some("HTML"),
        }
    }

    /// Wraps `text` in a pre block, escaping it as required by the format.
    fn wrap_in_pre(self, text: &str) -> String {
        match self {
            TextFormat::PlainText => text.to_string(),
            TextFormat::Markdown => {
                let escaped = text.replace('\\', "\\\\").replace('`', "\\`");
                format!("```\n{escaped}\n```")
            }
            TextFormat::Html => {
                let escaped = text
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                format!("<pre>{escaped}</pre>")
            }
        }
    }
}

pub struct TgClient {
    token: String,
    chat_id: String,
    agent: ureq::Agent,
}

impl TgClient {
    pub fn new(token: &str, chat_id: &str) -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build();
        Self {
            token: token.to_string(),
            chat_id: chat_id.to_string(),
            agent: ureq::Agent::new_with_config(config),
        }
    }

    fn build_bot_url(&self, path: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, path)
    }

    /// Calls a Bot API method, turning `ok: false` responses into
    /// [`TgApiError`]s. The request is abandoned after `timeout`, if given.
    fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        body: &Value,
        timeout: Option<Duration>,
    ) -> Result<T> {
        let resp: TgResponse<T> = self
            .agent
            .post(&self.build_bot_url(method))
            .config()
            .timeout_global(timeout)
            .build()
            .send_json(body)
            .with_context(|| format!("Failed to call {method}"))?
            .body_mut()
            .read_json()
            .with_context(|| format!("Failed to decode {method} response"))?;

        if !resp.ok {
            return Err(TgApiError {
                method: method.to_string(),
                code: resp.error_code,
                description: resp.description.unwrap_or_else(|| "unknown error".into()),
            }
            .into());
        }
        resp.result
            .with_context(|| format!("Telegram {method} returned no result"))
    }

    fn build_text_body(&self, text: &str, format: TextFormat) -> Value {
        let mut body = json!({
            "chat_id": self.chat_id,
            "text": text,
        });
        if let Some(parse_mode) = format.get_parse_mode() {
            body["parse_mode"] = json!(parse_mode);
        }
        body
    }

    pub fn send_message(
        &self,
        text: &str,
        format: TextFormat,
        reply_markup: Option<Value>,
    ) -> Result<TgMessage> {
        let mut body = self.build_text_body(text, format);
        if let Some(reply_markup) = reply_markup {
            body["reply_markup"] = reply_markup;
        }

        log::debug!("Sending message to chat {}: {text}", self.chat_id);

        self.call("sendMessage", &body, None)
            .context("Failed to send message")
    }

    pub fn send_ping(
        &self,
        text: &str,
        format: TextFormat,
        transient: bool,
        timeout: Duration,
    ) -> Result<()> {
        let allowed_updates = ["message", "message_reaction"];
        let offset = self.skip_pending_updates(&allowed_updates);
        let orig = self.send_message(text, format, None)?;

        let reply = self.wait_for_update(&allowed_updates, offset, timeout, |update| {
            let in_chat =
                |v: &Value| v.pointer("/chat/id").and_then(Value::as_i64) == Some(orig.chat.id);

            // Check for reaction on our message
            if let Some(reaction) = update.get("message_reaction")
                && in_chat(reaction)
                && let Some(mid) = reaction.get("message_id")
                && mid.as_i64() == Some(orig.id)
            {
                log::debug!("Received pong by reaction");
                return Break(None);
            }

            // Check for new message after ours in the same chat
            if let Some(msg) = update.get("message")
                && in_chat(msg)
                && let Some(mid) = msg.get("message_id").and_then(Value::as_i64)
                && mid > orig.id
            {
                log::debug!("Received pong by message");
                return Break(Some(mid));
            }

            Continue(())
        })?;

        if transient {
            let cleanup: Vec<i64> = std::iter::once(orig.id).chain(reply).collect();
            self.delete_messages(&cleanup);
        }
        Ok(())
    }

    /// Sends `text` and waits for the next text message in the chat, returning it.
    pub fn send_ask(&self, text: &str, format: TextFormat, timeout: Duration) -> Result<String> {
        let allowed_updates = ["message"];
        let offset = self.skip_pending_updates(&allowed_updates);
        let orig = self.send_message(text, format, Some(json!({ "force_reply": true })))?;

        self.wait_for_update(&allowed_updates, offset, timeout, |update| {
            // Check for a new text message after ours in the same chat
            if let Some(msg) = update.get("message")
                && msg.pointer("/chat/id").and_then(Value::as_i64) == Some(orig.chat.id)
                && let Some(mid) = msg.get("message_id").and_then(Value::as_i64)
                && mid > orig.id
                && let Some(answer) = msg.get("text").and_then(Value::as_str)
            {
                log::debug!("Received answer: {answer}");
                return Break(answer.to_string());
            }

            Continue(())
        })
    }

    /// Sends `text` with one inline button per option and waits until one is
    /// pressed, returning the chosen option.
    pub fn send_pick<'a>(
        &self,
        text: &str,
        format: TextFormat,
        options: &'a [String],
        timeout: Duration,
    ) -> Result<&'a str> {
        // Use the index as callback data, since it is limited to 64 bytes
        let keyboard: Vec<Value> = options
            .iter()
            .enumerate()
            .map(|(i, label)| json!([{ "text": label, "callback_data": i.to_string() }]))
            .collect();
        let allowed_updates = ["callback_query"];
        let offset = self.skip_pending_updates(&allowed_updates);
        let orig = self.send_message(text, format, Some(json!({ "inline_keyboard": keyboard })))?;

        let (query_id, choice) =
            self.wait_for_update(&allowed_updates, offset, timeout, |update| {
                // Check for a button press on our message, ignoring stale keyboards
                if let Some(query) = update.get("callback_query")
                    && let Some(msg) = query.get("message")
                    && msg.pointer("/chat/id").and_then(Value::as_i64) == Some(orig.chat.id)
                    && msg.get("message_id").and_then(Value::as_i64) == Some(orig.id)
                    && let Some(id) = query.get("id").and_then(Value::as_str)
                    && let Some(index) = query
                        .get("data")
                        .and_then(Value::as_str)
                        .and_then(|d| d.parse::<usize>().ok())
                    && let Some(choice) = options.get(index)
                {
                    log::debug!("Received pick: {choice}");
                    return Break((id.to_string(), choice.as_str()));
                }

                Continue(())
            })?;

        // Answer to stop the client's loading spinner
        self.answer_callback_query(&query_id);

        // Record the choice in the message, which also removes the keyboard
        let edited = format!("{text}\n{}", format.wrap_in_pre(&format!("-> {choice}")));
        if let Err(e) = self.edit_message_text(orig.id, &edited, format) {
            log::warn!("Failed to record pick: {e:#}");
            self.remove_reply_markup(orig.id);
        }
        Ok(choice)
    }

    fn answer_callback_query(&self, query_id: &str) {
        let body = json!({ "callback_query_id": query_id });

        if let Err(e) = self.call::<bool>("answerCallbackQuery", &body, None) {
            log::warn!("Failed to answer callback query: {e:#}");
        }
    }

    /// Replaces the text of a message, dropping its inline keyboard.
    fn edit_message_text(&self, message_id: i64, text: &str, format: TextFormat) -> Result<()> {
        let mut body = self.build_text_body(text, format);
        body["message_id"] = json!(message_id);

        self.call::<Value>("editMessageText", &body, None)?;
        Ok(())
    }

    /// Removes the inline keyboard of a message, keeping its text as is.
    fn remove_reply_markup(&self, message_id: i64) {
        let body = json!({
            "chat_id": self.chat_id,
            "message_id": message_id,
        });

        if let Err(e) = self.call::<Value>("editMessageReplyMarkup", &body, None) {
            log::warn!("Failed to remove keyboard: {e:#}");
        }
    }

    /// Discards pending updates and returns the offset of the first update yet
    /// to come, falling back to `-1` (only the latest update) on failure.
    ///
    /// Call this before sending the message whose responses are awaited, so
    /// that `allowed_updates` is in effect and no response can be skipped.
    fn skip_pending_updates(&self, allowed_updates: &[&str]) -> i64 {
        // Telegram only records update types from `allowed_updates` after a call sets it
        let body = json!({
            "allowed_updates": allowed_updates,
            "offset": -1,
            "timeout": 0,
        });

        match self.call::<Vec<Value>>("getUpdates", &body, Some(LONG_POLL_SLACK)) {
            Ok(updates) => updates
                .last()
                .and_then(|u| u.get("update_id"))
                .and_then(Value::as_i64)
                .map_or(0, |uid| uid + 1),
            Err(e) => {
                log::warn!("Failed to discard pending updates: {e:#}");
                -1
            }
        }
    }

    /// Long polls `getUpdates` from `offset` until `on_update` returns
    /// [`Break`] for some update, or `timeout` elapses.
    ///
    /// Network failures, rate limiting and server-side errors are retried;
    /// other API errors (e.g. a bad token or a conflicting webhook) are returned.
    fn wait_for_update<T>(
        &self,
        allowed_updates: &[&str],
        mut offset: i64,
        timeout: Duration,
        mut on_update: impl FnMut(&Value) -> ControlFlow<T>,
    ) -> Result<T> {
        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(anyhow!("Timeout after {}s", timeout.as_secs()));
            }
            // Telegram requires a positive long poll timeout
            let poll_timeout = remaining.clamp(Duration::from_secs(1), LONG_POLL_TIMEOUT);

            let update_body = json!({
                "allowed_updates": allowed_updates,
                "offset": offset,
                "timeout": poll_timeout.as_secs(),
            });

            log::debug!("Waiting for updates");
            match self.call::<Vec<Value>>(
                "getUpdates",
                &update_body,
                Some(poll_timeout + LONG_POLL_SLACK),
            ) {
                Ok(updates) => {
                    for update in &updates {
                        let uid = update.get("update_id").and_then(Value::as_i64);
                        if let Break(value) = on_update(update) {
                            if let Some(uid) = uid {
                                self.confirm_updates(uid + 1);
                            }
                            return Ok(value);
                        }

                        // Advance offset
                        if let Some(uid) = uid {
                            offset = uid + 1;
                        }
                    }
                }
                Err(e) => {
                    if let Some(api_err) = e.downcast_ref::<TgApiError>()
                        && !api_err.is_transient()
                    {
                        return Err(e.context("Failed to poll updates"));
                    }
                    log::warn!("Failed to poll updates: {e:#}");
                    thread::sleep(RETRY_DELAY);
                }
            }
        }
    }

    /// Marks all updates before `offset` as consumed so they are not
    /// redelivered.
    fn confirm_updates(&self, offset: i64) {
        // Confirmation is a side effect of `getUpdates`; `limit` only trims the response
        let body = json!({
            "offset": offset,
            "limit": 1,
            "timeout": 0,
        });

        if let Err(e) = self.call::<Vec<Value>>("getUpdates", &body, Some(LONG_POLL_SLACK)) {
            log::warn!("Failed to confirm updates: {e:#}");
        }
    }

    fn delete_messages(&self, message_ids: &[i64]) {
        let body = json!({
            "chat_id": self.chat_id,
            "message_ids": message_ids,
        });

        if let Err(e) = self.call::<bool>("deleteMessages", &body, None) {
            log::warn!("Failed to delete messages: {e:#}");
        }
    }
}
