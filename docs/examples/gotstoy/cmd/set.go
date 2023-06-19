// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"fmt"

	"github.com/PowerShell/DSC/docs/examples/gotstoy/config"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/ensure"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/scope"
	"github.com/spf13/cobra"
)

// setCmd represents the set command
var setCmd = &cobra.Command{
	Use:   "set",
	Short: "Set the current state of a TSToy configuration file.",
	Long: `You can use the set command to enforce the state of the
	fictional TSToy application's configuration files.
	
	If you specify a JSON blob from stdin or with the --inputJSON flag, it uses
	the values from that blob. You must specify the scope key in the JSON. All
	other keys are optional. If you don't specify the ensure key, the command
	defaults to ensure as present. If you don't specify the updateAutomatically
	or updateFrequency keys, the command ignores those settings. When you
	specify a JSON blob, you can't use the --scope, --ensure,
	--updateAutomatically, or --updateFrequency flags.
	
	If you're not specifying a JSON blob, you must specify the --scope flag.

	If you're not specifying a JSON blob and don't specify the --ensure flag,
	the default value is present.
	
	The set command enforces the specified settings on the application's
	configuration file for the specified scope. When ensure is absent, the
	command deletes the configuration file for that scope if it exists. When
	ensure is present, the command creates the file if it doesn't exist and
	sets the specified values as needed if it does.
	
	The command returns the JSON representation of the enforced state as a
	single-line string.`,
	RunE:              setState,
	Args:              cobra.NoArgs,
	ValidArgsFunction: validArgs,
}

func init() {
	rootCmd.AddCommand(setCmd)
}

func setState(cmd *cobra.Command, args []string) error {
	enforcing := config.Settings{}
	if inputJson != nil {
		if inputJson.Scope != scope.Undefined {
			enforcing.Scope = inputJson.Scope
		} else {
			return fmt.Errorf("no target scope specified")
		}
		if inputJson.Ensure != ensure.Undefined {
			enforcing.Ensure = inputJson.Ensure
		} else {
			enforcing.Ensure = ensure.Present
		}
		if inputJson.UpdateAutomatically != nil {
			enforcing.UpdateAutomatically = inputJson.UpdateAutomatically
		}
		if inputJson.UpdateFrequency != 0 {
			enforcing.UpdateFrequency = inputJson.UpdateFrequency
		}
	} else {
		if targetScope != scope.Undefined {
			enforcing.Scope = targetScope
		} else {
			return fmt.Errorf("no target scope specified")
		}
		if targetEnsure != ensure.Undefined {
			enforcing.Ensure = targetEnsure
		} else {
			enforcing.Ensure = ensure.Present
		}
		if rootCmd.PersistentFlags().Lookup("updateAutomatically").Changed {
			enforcing.UpdateAutomatically = &updateAutomatically
		}
		if updateFrequency != 0 {
			enforcing.UpdateFrequency = updateFrequency
		}
	}

	_, err := enforcing.Validate(false)
	if err != nil {
		return err
	}

	enforced, err := enforcing.Enforce()
	if err != nil {
		return err
	}

	return enforced.Print(pretty)
}
