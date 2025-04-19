package py

import (
	"github.com/spf13/viper"
	"github.com/yixuan-wang/tyw/pkg/util"
)

var pyConfig *viper.Viper

func InitConfig() error {
	if pyConfig = viper.GetViper().Sub("py"); pyConfig == nil {
		return util.Fail("Cannot find [py] config")
	}
	return nil
}
