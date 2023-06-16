// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"fmt"

	"github.com/PowerShell/DSC/docs/examples/app/config"
	"github.com/TylerBrock/colorjson"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/exp/slices"
)

var only []string

// showCmd represents the show command
var showCmd = &cobra.Command{
	Use:   "show",
	Short: "Shows the merged configuration options for the application.",
	Long: `This command shows the merged configuration options for the
	application. The settings are applied first from the machine-level,
	then the user-level, then any environment variables, and finally any
	flags passed to the command.`,
	Run:  run,
	Args: cobra.MatchAll(cobra.NoArgs),
}

func init() {
	rootCmd.AddCommand(showCmd)

	showCmd.Flags().StringSliceVar(
		&only,
		"only",
		[]string{"default", "machine", "user", "final"},
		"The list of configuration values to retrieve. Valid values are 'default', 'machine', 'user', and 'final'.",
	)
	showCmd.RegisterFlagCompletionFunc("only", configTargetCompletion)
}

func run(cmd *cobra.Command, args []string) {
	if len(only) == 1 {
		printConfigOnly(only[0])
		return
	}
	if slices.Contains(only, "default") {
		printConfig(config.Default.ToMap(), "Default", defaultConfig)
	}
	if slices.Contains(only, "machine") {
		printConfig(MachineConfig, "Machine", configFile)
	}
	if slices.Contains(only, "user") {
		printConfig(UserConfig, "User", configFile)
	}
	if slices.Contains(only, "final") {
		printConfig(viper.AllSettings(), "Final", final)
	}
}

func configTargetCompletion(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	return []string{
		"default\tReturns the default configuration settings.",
		"machine\tReturns the machine-scope configuration settings.",
		"user\tReturns the user-scope configuration settings.",
		"final\tReturns the final configuration after all mergings.",
	}, cobra.ShellCompDirectiveDefault
}

var baseStyle = lipgloss.NewStyle().Bold(true)
var defaultStyle = baseStyle.Copy().Foreground(lipgloss.Color("#698F3F"))
var configFileStyle = baseStyle.Copy().Foreground(lipgloss.Color("#1F5673"))
var finalStyle = baseStyle.Copy().Foreground(lipgloss.Color("#DE4D86"))

type configGroup int

const (
	defaultConfig configGroup = iota
	configFile
	final
)

func printConfigOnly(target string) {
	var targetConfig any
	switch target {
	case "machine":
		targetConfig = MachineConfig
	case "user":
		targetConfig = UserConfig
	case "final":
		targetConfig = viper.AllSettings()
	default:
		targetConfig = config.Default.ToMap()
	}

	getJsonFormatter()
	formatted, _ := formatter.Marshal(targetConfig)
	fmt.Println(string(formatted))
}

func printConfig(value any, name string, group configGroup) {
	getJsonFormatter()
	formatted, _ := formatter.Marshal(value)
	var style lipgloss.Style
	switch group {
	case defaultConfig:
		style = defaultStyle
	case final:
		style = finalStyle
	case configFile:
		style = configFileStyle
	default:
		style = defaultStyle
	}
	prefix := fmt.Sprintf("%s configuration:", name)
	prefix = style.Render(prefix)
	fmt.Println(prefix, string(formatted))
}

var formatter *colorjson.Formatter

func getJsonFormatter() {
	if nil == formatter {
		formatter = colorjson.NewFormatter()
		formatter.Indent = 2
	}
}
