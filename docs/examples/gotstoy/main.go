// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package main

import (
	"os"

	"github.com/PowerShell/DSC/docs/examples/gotstoy/cmd"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/input"
)

func main() {
	// Normally, we let cobra handle everything around args, but because we
	// need to support JSON over stdin, we have to do some munging.
	//
	// First, grab the args for the command itself.
	args := []string{}
	for index, arg := range os.Args {
		// skip first index, because it's the application name
		if index > 0 {
			args = append(args, arg)
		}
	}

	// Check stdin and add each found JSON blob after an --inputJSON flag.
	args = input.HandleStdIn(args)

	cmd.Execute(args)
}
