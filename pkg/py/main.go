package py

import (
	"log/slog"
	"os"

	"github.com/spf13/viper"
)

var pyConfig *viper.Viper

func InitConfig() {
	if pyConfig = viper.GetViper().Sub("py"); pyConfig == nil {
		slog.Error("Could not find [py] configuration")
		os.Exit(1)
	}
}
