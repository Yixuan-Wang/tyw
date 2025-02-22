package cmd

import (
	"log/slog"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/yixuan-wang/tyw/pkg/py"
)

var cfgFile string

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "tyw",
	Short: "Yixuan \"Tom\" Wang's command line utilities",
	Long: `A collection of command line utilities for Yixuan "Tom" Wang.`,
	// Uncomment the following line if your bare application
	// has an action associated with it:
	// Run: func(cmd *cobra.Command, args []string) { },
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	cobra.OnInitialize(initConfig)

	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default: $XDG_CONFIG_HOME/tyw.toml)")

	// Cobra also supports local flags, which will only run
	// when this action is called directly.
	rootCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	if cfgFile != "" {
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
	} else {
		home, err := os.UserHomeDir()
		cobra.CheckErr(err)

		osDefaultConfigPath, _ := os.UserConfigDir()
		viper.AddConfigPath(osDefaultConfigPath)
		slog.Debug("Add config path location candidate", "path", osDefaultConfigPath)

		homeConfigPath := filepath.Join(home, ".config")
		viper.AddConfigPath(homeConfigPath)
		slog.Debug("Add config path location candidate", "path", homeConfigPath)

		viper.SetConfigType("toml")
		viper.SetConfigName("tyw")
	}

	viper.AutomaticEnv() // read in environment variables that match

	// If a config file is found, read it in.
	if err := viper.ReadInConfig(); err == nil {
		slog.Info("Loaded config file", "file", viper.ConfigFileUsed())
	} else {
		slog.Error("Failed to load config")
	}

	py.InitConfig()
}
