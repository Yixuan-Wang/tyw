package cmd

import (
	"github.com/spf13/cobra"
	"github.com/yixuan-wang/tyw/pkg/tg"
	"github.com/yixuan-wang/tyw/pkg/util"
)

var tgCmd = &cobra.Command{
	Use:   "tg",
	Short: "Telegram utilities.",
	Long: `Utilities for sending quick messages through Telegram.`,
	PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
		return tg.InitConfig()
	},
}

func init() {
	rootCmd.AddCommand(tgCmd)

	tgCmd.AddCommand(&cobra.Command{
		Use:   "text",
		Short: "Text to a chat",
		Long:  `Send a text message to a Telegram chat.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) == 0 {
				return util.Fail("Please provide a message to send.")
			}

			chatId := tg.TgConfig.GetString("chat_id")
			if chatId == "" {
				return util.Fail("Please provide a chat ID.")
			}
			tg.SendMessage(chatId, args[0])
			return nil
		},
	})
}
