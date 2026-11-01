package tg

import (
	"fmt"
	"log/slog"

	"github.com/spf13/viper"
)

var tgConfig *viper.Viper

func InitConfig() error {
	if tgConfig = viper.GetViper().Sub("tg"); tgConfig == nil {
		tgConfig = viper.New()
		tgConfig.SetConfigName("tg")
		slog.Warn("No [tg] config found, using default")
	}

	tgConfig.SetEnvPrefix("TELEGRAM")
	tgConfig.BindEnv("token")

	slog.Warn("Telegram config loaded", "file", tgConfig.ConfigFileUsed())

	return nil
}

func GetChatId() (string, error) {
	chatId := tgConfig.GetString("chat_id")
	if chatId == "" {
		slog.Warn("No chat_id found in config, using default")
		return "", fmt.Errorf("no chat_id found in config")
	}
	return chatId, nil
}
