// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package ensure

import (
	"encoding/json"
	"fmt"

	"github.com/spf13/cobra"
)

// An enum defining whether the application's configuration file should
// exist in a specific scope.
type Value int

const (
	// Undefined implies that the value hasn't been explicitly set. It's
	// invalid as a Setting option.
	Undefined Value = iota
	// Indicates that the configuration file shouldn't exist.
	Absent
	// Indicates that the configuration file should exist.
	Present
)

// Returns the string representation of the enum.
func (value Value) String() string {
	switch value {
	case Absent:
		return "absent"
	case Present:
		return "present"
	}

	return "unknown"
}

// Converts an input string into the enum value.
func Parse(value string) (Value, error) {
	switch value {
	case "absent":
		return Absent, nil
	case "present":
		return Present, nil
	}

	return Undefined, fmt.Errorf(
		"unable to convert '%s' to valid scope, must be one of: 'machine', 'user'",
		value,
	)
}

// Informs the JSON parser how the enum should be marshalled. This ensures the
// value is written to JSON as the string, not as the numerical value.
func (value Value) MarshalJSON() ([]byte, error) {
	return json.Marshal(value.String())
}

// Informs the JSON parser how the enum should be read from a JSON blob. This
// ensures the value in JSON can be read as a string instead of the numerical
// value.
func (value *Value) UnmarshalJSON(data []byte) (err error) {
	var v string
	if err := json.Unmarshal(data, &v); err != nil {
		return err
	}
	if *value, err = Parse(v); err != nil {
		return err
	}

	return nil
}

// Maps the enum's value to its string representation. This's required for the
// enumflag library used to create a command line flag for the enum.
var FlagMap = map[Value][]string{
	Absent:  {"absent"},
	Present: {"present"},
}

// Handles completion for the flag at the commandline. When the user enables
// shell completion, this ensures that they're prompted for the correct values
// and that the help for each value displays.
func FlagCompletion(cmd *cobra.Command, args []string, toComplete string) ([]string, cobra.ShellCompDirective) {
	completions := []string{
		"present\tThe configuration file should exist.",
		"absent\tThe configuration file shouldn't exist.",
	}
	return completions, cobra.ShellCompDirectiveNoFileComp
}
