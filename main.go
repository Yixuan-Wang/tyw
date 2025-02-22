package main

import (
	"log/slog"
	"os"

	charmlog "github.com/charmbracelet/log"
	"github.com/yixuan-wang/tyw/cmd"
	"golang.org/x/term"
)

func main() {
	var logHandler slog.Handler
	if term.IsTerminal(int(os.Stdout.Fd())) {
		logHandler = charmlog.NewWithOptions(os.Stderr, charmlog.Options{ Level: charmlog.WarnLevel })
	} else {
		logHandler = slog.DiscardHandler
	}

	logger := slog.New(logHandler)
	slog.SetDefault(logger)

	cmd.Execute()
}
