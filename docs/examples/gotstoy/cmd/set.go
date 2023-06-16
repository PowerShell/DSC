// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"encoding/json"
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
	the values from that blob, ignoring the --scope, --ensure,
	--updateAutomatically, and --updateFrequency flags.
	
	If you're not specifying a JSON blob, you must specify the --scope flag.

	If you don't specify the --ensure flag, the default value is present.`,
	RunE: setState,
}

func init() {
	rootCmd.AddCommand(setCmd)
}

func setState(cmd *cobra.Command, args []string) error {
	enforcing := config.Settings{}
	if inputJson != nil {

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
			enforcing.UpdateAutomatically = updateAutomatically
		}
		if updateFrequency != 0 {
			enforcing.UpdateFrequency = updateFrequency
		}
	}

	_, err := enforcing.Validate(false)
	if err != nil {
		return err
	}

	enforcingJson, err := json.Marshal(enforcing)
	if err != nil {
		return err
	}

	fmt.Println("setting:", string(enforcingJson))

	return nil
}
