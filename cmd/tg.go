package cmd

import (
	"time"

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
			var message string
			if len(args) == 0 {
				message = "Hello, world!"
			} else {
				message = args[0]
			}

			chatId, err := tg.GetChatId()
			if err != nil {
				return err
			}
			tg.SendMessage(chatId, message)
			return nil
		},
	})

	tgPingCmd := cobra.Command{
		Use: "ping",
		Short: "Send a ping message",
		Long: `Send a ping message to a Telegram chat.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			chatId, err := tg.GetChatId()
			if err != nil {
				return err
			}

			var text string
			if len(args) > 0 {
				text = args[0]
			}

			timeout, _ := cmd.Flags().GetDuration("timeout")

			if err := tg.SendPing(chatId, text, true, timeout); err != nil {
				return util.Fail("Didn't receive a response.")
			}
			return nil
		},
	}
	tgPingCmd.Flags().DurationP("timeout", "t", 6 * time.Hour, "Duration to wait before timeout")

	tgCmd.AddCommand(&tgPingCmd)
}
