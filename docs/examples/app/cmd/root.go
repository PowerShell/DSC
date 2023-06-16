// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package cmd

import (
	"fmt"
	"os"

	"github.com/PowerShell/DSC/docs/examples/app/config"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
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

var cfgFile string
var autoUpdate bool
var checkFrequency int
var MachineConfig map[string]interface{}
var UserConfig map[string]interface{}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "tstoy",
	Short: "A fictional application to demonstrate writing a DSC Resource.",
	Long: `This is a fictional application with a handful of configuration
options defined so you can write a Desired State Configuration (DSC) Resource
against a known application.

While a real application could define its own manifest for a DSC Resource,
tstoy does not. Instead, the DSC documentation includes a number of example
DSC Resources all managing this application but written in different
programming languages.`,
	Version: version,
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	cobra.OnInitialize(initConfig)

	// Users can pass an explicit config.
	rootCmd.PersistentFlags().StringVar(
		&cfgFile,
		"config",
		"",
		"config file (default is the machine configuration, then user configuration)",
	)

	rootCmd.PersistentFlags().BoolVar(
		&autoUpdate,
		"update-automatically",
		false,
		"Specifies whether the app should update automatically if the frequency window is past.",
	)
	viper.BindPFlag("updates.automatic", rootCmd.PersistentFlags().Lookup("update-automatically"))

	rootCmd.PersistentFlags().IntVar(
		&checkFrequency,
		"update-frequency",
		config.Default.Updates.CheckFrequency,
		"Specifies the length of the frequency window for updates in days.",
	)
	viper.BindPFlag("updates.checkFrequency", rootCmd.PersistentFlags().Lookup("update-frequency"))
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	viper.SetDefault("updates.automatic", config.Default.Updates.Automatic)
	viper.SetDefault("updates.checkFrequency", config.Default.Updates.CheckFrequency)

	viper.AutomaticEnv() // read in environment variables that match

	// Check for and merge-in the config settings for the machine, then user configs.
	machineConfig := getScopedConfigFile(config.Machine)
	if nil != machineConfig {
		viper.MergeConfigMap(machineConfig)
	}
	userConfig := getScopedConfigFile(config.User)
	if nil != userConfig {
		viper.MergeConfigMap(userConfig)
	}

	if cfgFile != "" {
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
		// If a config file is found, read it in.
		if err := viper.ReadInConfig(); err == nil {
			fmt.Fprintln(os.Stderr, "Using config file:", viper.ConfigFileUsed())
		}
	}
}

func getScopedConfigFile(scope config.Scope) map[string]interface{} {
	folder := config.MachineFolder
	if scope == config.User {
		folder = config.UserFolder
	}

	v := viper.New()
	v.AddConfigPath(folder)
	v.SetConfigName(config.FileName)
	v.SetConfigType(config.FileExtension)

	// If a config file exists, read it in.
	if err := v.ReadInConfig(); err == nil {
		if scope == config.User {
			UserConfig = v.AllSettings()
		} else {
			MachineConfig = v.AllSettings()
		}
		return v.AllSettings()
	}

	return nil
}
