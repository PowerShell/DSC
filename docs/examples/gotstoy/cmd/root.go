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

var inputJson *config.Settings
var targetScope scope.Value
var targetEnsure ensure.Value
var updateAutomatically bool
var updateFrequency update.Frequency

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "gotstoy",
	Short: "The DSC Resource for managing tstoy, written in go.",
	Long: `This application is an implementation of DSCv3 Resource for
	managing the fictional tstoy application.`,
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
		"input JSON, passable from stdin.",
	)
	rootCmd.PersistentFlags().Var(
		enumflag.New(&targetScope, "scope", scope.FlagMap, enumflag.EnumCaseInsensitive),
		"scope",
		"target scope for the configuration.",
	)
	rootCmd.PersistentFlags().Var(
		enumflag.New(&targetEnsure, "ensure", ensure.FlagMap, enumflag.EnumCaseInsensitive),
		"ensure",
		"whether the configuration file should exist.",
	)
	rootCmd.PersistentFlags().BoolVar(
		&updateAutomatically,
		"updateAutomatically",
		false,
		"whether the configuration should set the app to automatically update.",
	)
	rootCmd.PersistentFlags().Var(
		&updateFrequency,
		"updateFrequency",
		"how frequently the configuration should update, between 1 and 90 days inclusive.",
	)
}
