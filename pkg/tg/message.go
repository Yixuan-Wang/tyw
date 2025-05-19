package tg

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
)

type TgChat struct {
	Id int64  `json:"id"`
	Type string `json:"type"`
	Username string `json:"username"`
}

type TgMessage struct {
	Id int64  `json:"message_id"`
	Date int64  `json:"date"`
	From TgChat `json:"from"`
	Chat TgChat `json:"chat"`
	Text string `json:"text"`
}

func getBotURL(path ...string) string {
	botUrl := fmt.Sprintf("https://api.telegram.org/bot%s", TgConfig.Get("token"))
	if len(path) > 0 {
		fullUrl, _ := url.JoinPath(botUrl, path...)
		return fullUrl
	} else {
		fullUrl, _ := url.JoinPath(botUrl, "/")
		return fullUrl
	}
}

func SendMessage(
	chatID string,
	message string,
) (TgMessage, error) {
	url := getBotURL("sendMessage")
	body, err := json.Marshal(map[string]interface{} {
		"chat_id": chatID,
		"text": message,
	})
	if err != nil {
		slog.Error("Cannot serialize message", "message", message, "error", err)
		return TgMessage{}, err
	}

	slog.Debug("Sending message", "message", message, "url", url, "body", string(body))
	bufBody := bytes.NewBuffer(body)
	resp, err := http.Post(url, "application/json", bufBody)
	if err != nil {
		slog.Error("Cannot send message", "message", message, "error", err)
		return TgMessage{}, err
	}
	defer resp.Body.Close()

	slog.Debug("Received response", "status", resp.StatusCode, "message", message)
	
	var messageResponse TgMessage
	if err := json.NewDecoder(resp.Body).Decode(&messageResponse); err != nil {
		slog.Error("Cannot decode response", "message", message, "error", err)
		return TgMessage{}, err
	}

	return messageResponse, nil
}
