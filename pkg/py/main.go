package py

import (
	"log/slog"

	"github.com/spf13/viper"
)

var pyConfig *viper.Viper

func InitConfig() error {
	if pyConfig = viper.GetViper().Sub("py"); pyConfig == nil {
		pyConfig = viper.New()
		pyConfig.SetConfigName("py")
		slog.Warn("No [py] config found, using default")

		return nil
	}
	return nil
}
