package tg

import (
	"log/slog"

	"github.com/spf13/viper"
)

var TgConfig *viper.Viper

func InitConfig() error {
	if TgConfig = viper.GetViper().Sub("tg"); TgConfig == nil {
		TgConfig = viper.New()
		TgConfig.SetConfigName("tg")
		slog.Warn("No [tg] config found, using default")
	}

	TgConfig.SetEnvPrefix("TELEGRAM")
	TgConfig.BindEnv("token")

	slog.Warn("Telegram config loaded", "file", TgConfig.ConfigFileUsed())

	return nil
}
