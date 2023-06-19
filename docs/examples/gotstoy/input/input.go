// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package input

import (
	"encoding/json"
	"io"
	"os"
	"strings"
)

// Defines a valid PFlag that you can use to take a JSON blob as a flag value
// in your Cobra command. This is required to support sending JSON blobs to
// the resource from stdin.
type JSONFlag struct {
	Target interface{}
}

// String is used both by fmt.Print and by Cobra in help text
func (f *JSONFlag) String() string {
	b, err := json.Marshal(f.Target)
	if err != nil {
		return "failed to marshal object"
	}
	return string(b)
}

// Set must have pointer receiver so it doesn't change the value of a copy
func (f *JSONFlag) Set(v string) error {
	return json.Unmarshal([]byte(v), f.Target)
}

// Type is only used in help text
func (f *JSONFlag) Type() string {
	return "json"
}

// HandleStdIn returns an array of arguments, appending the --inputJSON flag
// with the accompanying JSON blob. It expects that the full input from stdin
// is a single JSON blob.
//
// For example, if you called gotstoy with a single-line JSON blob:
//
//	`{ "scope": "machine" }` | gotstoy get
//
// It returns the args:
//
//   - get
//   - --inputJSON
//   - { "scope": "machine" }
func HandleStdIn(args []string) []string {
	info, _ := os.Stdin.Stat()
	if (info.Mode() & os.ModeCharDevice) == os.ModeCharDevice {
		// do nothing
	} else {
		stdin, err := io.ReadAll(os.Stdin)
		if err != nil {
			panic(err)
		}

		jsonBlob := strings.Trim(string(stdin), "\n")
		jsonBlob = strings.Trim(jsonBlob, "\r")
		jsonBlob = strings.TrimSpace(jsonBlob)
		if jsonBlob != "" {
			args = append(args, "--inputJSON", jsonBlob)
		}
	}

	return args
}
