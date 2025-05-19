# `tyw tg`

Utilities for sending Telegram messages via a bot.

## Configuration

```toml
[tg]
token = "<token>" # Telegram bot token
chat_id = "<chat_id>" # Telegram chat ID
```

> [!TIP]
> You can get your chat ID by sending a message to your bot and then using the `getUpdates` method of the Telegram Bot API.
> For example, you can use the following command to get all chats associated with your bot:
> ```bash
> curl -X POST "https://api.telegram.org/bot<token>/getUpdates"
> ```
> The response will include a list of updates, including the chat ID of the message you sent.


## Messages

### `text`

Send a text message to the configured chat.

```bash
tyw tg text "<message>"
```
