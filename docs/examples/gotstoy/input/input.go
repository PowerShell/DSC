// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package input

import (
	"encoding/json"
	"io"
	"os"
	"strings"
)

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
// with the accompanying JSON blob for every line of JSON piped to the command
// from stdin. It expects JSON blobs to be passed one-per-line.
//
// For example, if you called gotstoy with a single blob:
//
//	'{ "scope": "machine" }' | gotstoy get
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

		jsonInputs := []string{}
		jsonBlobs := strings.Split(string(stdin), "\n")
		for _, blob := range jsonBlobs {
			blob = strings.Trim(blob, "\r")
			if blob != "" {
				jsonInputs = append(jsonInputs, blob)
			}
		}
		for _, blob := range jsonInputs {
			args = append(args, "--inputJSON", blob)
		}
	}

	return args
}
