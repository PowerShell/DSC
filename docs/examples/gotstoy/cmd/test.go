// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// testCmd represents the test command
var testCmd = &cobra.Command{
	Use:   "test",
	Short: "Test the current state of a TSToy configuration file.",
	Long: `You can use the test command to validate the state of the
	fictional TSToy application's configuration files.
	
	If you specify a JSON blob from stdin or with the --inputJSON flag, it uses
	the values from that blob, ignoring the --scope, --ensure,
	--updateAutomatically, and --updateFrequency flags.
	
	If you're not specifying a JSON blob, you must specify the --scope flag.

	If you don't specify the --ensure flag, the default value is present.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("test called")
	},
}

func init() {
	rootCmd.AddCommand(testCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// testCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// testCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
