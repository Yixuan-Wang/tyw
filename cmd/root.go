package cmd

import (
	"log/slog"
	"os"
	"path/filepath"

	charmlog "github.com/charmbracelet/log"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/term"
)

var cfgFile string
var Verbose int
var Debug bool

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "tyw",
	Short: "Yixuan \"Tom\" Wang's command line utilities",
	Long:  `A collection of command line utilities for Yixuan "Tom" Wang.`,
	// Uncomment the following line if your bare application
	// has an action associated with it:
	// Run: func(cmd *cobra.Command, args []string) { },
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the root
func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	cobra.OnInitialize(initLogger, initConfig)

	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "Config file (default: $XDG_CONFIG_HOME/tyw.toml)")
	rootCmd.PersistentFlags().CountVarP(&Verbose, "verbose", "v", "Verbosity level, max at `-vvv` (default: 0)")
	rootCmd.PersistentFlags().BoolVarP(&Debug, "debug", "D", false, "Print debug information")
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

		homeConfigPath := filepath.Join(home, ".config")
		viper.AddConfigPath(homeConfigPath)

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
}

func initLogger() {
	var level charmlog.Level
	switch {
	case Verbose <= 0:
		level = charmlog.FatalLevel
	case Verbose == 1:
		level = charmlog.ErrorLevel
	case Verbose == 2:
		level = charmlog.WarnLevel
	case Verbose >= 3:
		level = charmlog.InfoLevel
	}

	if Debug {
		level = charmlog.DebugLevel
	}

	var logHandler slog.Handler

	if term.IsTerminal(int(os.Stdout.Fd())) {
		logHandler = charmlog.NewWithOptions(os.Stderr, charmlog.Options{Level: level})
	} else {
		logHandler = slog.DiscardHandler
	}

	logger := slog.New(logHandler)
	slog.SetDefault(logger)
}
