// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"encoding/json"
	"fmt"

	"github.com/PowerShell/DSC/docs/examples/gotstoy/config"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/scope"
	"github.com/spf13/cobra"
)

var all bool

// getCmd represents the get command
var getCmd = &cobra.Command{
	Use:   "get",
	Short: "Get the current state of TSToy's configuration file or files.",
	Long: `You can use the get command to return the current state of the
	fictional TSToy application's configuration files.
	
	If you specify a JSON blob from stdin or with the --inputJSON flag, it uses
	the 'scope' value to find the correct configuration to return.
	
	If you specify the --scope flag, it uses that value instead.
	
	If you specify the --all flag, it returns the configuration settings for
	both scopes.
	
	The configuration settings are returned as JSON blobs, one per line.`,
	RunE: getState,
}

func init() {
	rootCmd.AddCommand(getCmd)
	getCmd.Flags().BoolVar(
		&all,
		"all",
		false,
		"Get the configurations for all scopes.",
	)
}

func getState(cmd *cobra.Command, args []string) error {
	getScopes := []scope.Value{}
	if all {
		getScopes = append(getScopes, scope.Machine, scope.User)
	} else if inputJson != nil {
		getScopes = append(getScopes, inputJson.Scope)
	} else if targetScope != scope.Undefined {
		getScopes = append(getScopes, targetScope)
	}

	for _, getScope := range getScopes {
		// Retrieve the configuration file's JSON value, convert to Settings.
		configSettings, err := config.GetAppConfigSettings(getScope)
		if err != nil {
			return err
		}

		// Marshal the configuration's current state to JSON, omitting the
		// empty keys.
		configJson, err := json.Marshal(configSettings)
		if err != nil {
			return err
		}
		fmt.Println(string(configJson))
	}

	return nil
}
