package cmd

import (
	"github.com/spf13/cobra"
	"github.com/yixuan-wang/tyw/pkg/py"
)

var pyCmd = &cobra.Command{
	Use:   "py",
	Short: "Python utilities.",
	Long: `Utilities for managing Python installations, environments and other stuff.`,
}

func init() {
	rootCmd.AddCommand(pyCmd)

	pyCmd.AddCommand(&cobra.Command{
		Use:   "list",
		Short: "List venvs.",
		Long: `List Python virtual environments.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			return py.ListEnv()
		},
	})

	pyCmd.AddCommand(&cobra.Command{
		Use:   "use",
		Short: "Use a Python virtual environment",
		Long: `Use a Python virtual environment.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) == 0 {
				return py.TryUseEnv()
			} else {
				return py.UseEnv(args[0])
			}
		},
	})

	pyCmd.AddCommand(&cobra.Command{
		Use:  "sel",
		Short: "Select and use a Python virtual environment",
		Long: `Select and use a Python virtual environment.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			return py.SelectEnv()
		},
	})
}
