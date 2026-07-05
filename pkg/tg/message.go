package tg

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
	"os"
	"time"
)

type TgChat struct {
	Id       int64  `json:"id"`
	Type     string `json:"type"`
	Username string `json:"username"`
}

type TgMessage struct {
	Id   int64  `json:"message_id"`
	Date int64  `json:"date"`
	From TgChat `json:"from"`
	Chat TgChat `json:"chat"`
	Text string `json:"text"`
}

type TgResponse[T any] struct {
	Ok     bool   `json:"ok"`
	Result T      `json:"result"`
}

func getBotURL(path ...string) string {
	botUrl := fmt.Sprintf("https://api.telegram.org/bot%s", tgConfig.Get("token"))
	if len(path) > 0 {
		fullUrl, _ := url.JoinPath(botUrl, path...)
		return fullUrl
	} else {
		fullUrl, _ := url.JoinPath(botUrl, "/")
		return fullUrl
	}
}

func SendMessage(
	chatId string,
	message string,
) (TgMessage, error) {
	url := getBotURL("sendMessage")
	body, err := json.Marshal(map[string]interface{}{
		"chat_id": chatId,
		"text":    message,
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

	var messageResponse TgResponse[TgMessage]
	if err := json.NewDecoder(resp.Body).Decode(&messageResponse); err != nil {
		slog.Error("Cannot decode response", "message", message, "error", err)
		return TgMessage{}, err
	}

	return messageResponse.Result, nil
}

func SendPing(
	chatId string,
	message string,
	transient bool,
	timeout time.Duration,
) error {
	if message == "" {
		message = "Heads up!"
	}

	origMessage, err := SendMessage(chatId, message)
	if err != nil {
		slog.Error("Cannot send ping", "chatID", chatId, "error", err)
		return err
	}

	messageId := origMessage.Id
	updateUrl := getBotURL("getUpdates")

	updateParams := struct { AllowedUpdates []string `json:"allowed_updates"`; Offset int64 `json:"offset"` } {
		Offset: -1,
		AllowedUpdates: []string{ "message", "message_reaction" },
	}

	pollResult := make(chan bool)
	cleanup := []int64 { messageId }
	messageSentTime := time.Unix(origMessage.Date, 0)

	go func() {
		for {
			ok := func() bool {
				jsonUpdateParams, _ := json.Marshal(updateParams)
				bufUpdateParams := bytes.NewBuffer(jsonUpdateParams)
				resp, err := http.Post(updateUrl, "application/json", bufUpdateParams)
				if err != nil {
					slog.Warn("Failed to poll updates", "error", err)
					return false
				}
				defer resp.Body.Close()
	
				var messageResponse TgResponse[[1]map[string]any]
				json.NewDecoder(resp.Body).Decode(&messageResponse)
				
				slog.Debug("Received updates", "response", messageResponse)
				
	
				for _, message := range messageResponse.Result {
					if message["message_reaction"] != nil {
						if updateMessageId := message["message_reaction"].(map[string]any)["message_id"]; updateMessageId != nil && int64(updateMessageId.(float64)) == messageId {
							slog.Debug("Received pong by reaction", "messageId", messageId)
							return true
						}
					} else if message["message"] != nil {
						updateMessageTime := int64(message["message"].(map[string]any)["date"].(float64))
						if updateMessageTime > messageSentTime.Unix() {
							slog.Debug("Received pong by message", "messageId", messageId)

							updateMessageId := int64(message["message"].(map[string]any)["message_id"].(float64))
							cleanup = append(cleanup, updateMessageId)

							return true
						}
					}
	
					if updateId := message["update_id"]; updateId != nil {
						updateParams.Offset = int64(message["update_id"].(float64)) + 1
					}
				}
				return false
			}()
			if ok {
				pollResult <- true
				break
			}
			slog.Debug("Waiting for message update", "messageId", messageId)
			time.Sleep(5 * time.Second)
		}
	}()

	select {
	case <-pollResult:
		now := time.Now()
		elapsed := now.Sub(messageSentTime)
		slog.Debug("Received response", "elapsed_min", elapsed.Minutes())

		if transient {
			cleanupUrl := getBotURL("deleteMessages")
			cleanupBody, _ := json.Marshal(map[string]interface{}{
				"chat_id": chatId,
				"message_ids": cleanup,
			})
			cleanupBufBody := bytes.NewBuffer(cleanupBody)
			if cleanupResp, err := http.Post(cleanupUrl, "application/json", cleanupBufBody); err == nil {
				cleanupResp.Body.Close()
			}
		}

		return nil
	case <-time.After(timeout):
		slog.Warn("Ping message update timed out", "messageId", messageId)
		fmt.Fprintf(os.Stderr, "Timeout after %s\n", timeout)
		os.Exit(1)
		return nil
	}
}