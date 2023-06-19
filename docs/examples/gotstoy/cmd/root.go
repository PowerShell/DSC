// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"os"

	"github.com/PowerShell/DSC/docs/examples/gotstoy/config"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/ensure"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/scope"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/update"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/input"
	"github.com/spf13/cobra"
	"github.com/thediveo/enumflag"
)

// Build-time variables
var (
	// The version of the app - usually the tagged version
	version = "dev"
	// The most recent commit for the build
	commit = "none"
	// The date the app was built
	date = "unknown"
)

var inputJson *config.Settings
var targetScope scope.Value
var targetEnsure ensure.Value
var updateAutomatically bool
var updateFrequency update.Frequency
var pretty bool

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "gotstoy",
	Short: "The DSC Resource for managing tstoy, written in go.",
	Long: `This application is an implementation of DSCv3 Resource for
	managing the fictional tstoy application.
	
	It has two commands: get and set.
	
	You can use the get command to retrieve a JSON representation of the
	current state of the resource.
	
	You can use the set command to enforce the desired state of the
	resource.`,
	Version: version,
}

// Execute adds all child commands to the root command and sets flags
// appropriately. This is called by main.main(). It only needs to
// happen once to the rootCmd.
//
// Unlike normal cobra apps, this one sets the args explicitly from main to
// account for JSON blobs sent from stdin.
func Execute(args []string) {
	rootCmd.SetArgs(args)
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	rootCmd.PersistentFlags().Var(
		&input.JSONFlag{Target: &inputJson},
		"inputJSON",
		"Specify options as a JSON blob instead of using the scope, ensure, and update* flags.",
	)

	rootCmd.PersistentFlags().Var(
		enumflag.New(&targetScope, "scope", scope.FlagMap, enumflag.EnumCaseInsensitive),
		"scope",
		"The target scope for the configuration.",
	)
	rootCmd.RegisterFlagCompletionFunc("scope", scope.FlagCompletion)

	rootCmd.PersistentFlags().Var(
		enumflag.New(&targetEnsure, "ensure", ensure.FlagMap, enumflag.EnumCaseInsensitive),
		"ensure",
		"Whether the configuration file should exist.",
	)
	rootCmd.RegisterFlagCompletionFunc("ensure", ensure.FlagCompletion)

	rootCmd.PersistentFlags().BoolVar(
		&updateAutomatically,
		"updateAutomatically",
		false,
		"Whether the configuration should set the app to automatically update. Pass as --updateAutomatically=false to set to false.",
	)

	rootCmd.PersistentFlags().Var(
		&updateFrequency,
		"updateFrequency",
		"How frequently the configuration should update, between 1 and 90 days inclusive.",
	)

	rootCmd.PersistentFlags().BoolVar(
		&pretty,
		"pretty",
		false,
		"Whether the output should pretty print as indented and colorized JSON.",
	)
}

func validArgs(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	return nil, cobra.ShellCompDirectiveNoFileComp
}
