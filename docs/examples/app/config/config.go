// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package config

import (
	"encoding/json"
	"path/filepath"

	"github.com/adrg/xdg"
)

type Options struct {
	Updates Updates `mapstructure:"updates,omitempty"`
}

type Updates struct {
	Automatic      bool `mapstructure:"automatic,omitempty"`
	CheckFrequency int  `mapstructure:"checkFrequency,omitempty"`
}

var Default = Options{
	Updates: Updates{
		Automatic:      false,
		CheckFrequency: 90,
	},
}

func (o *Options) ToJson() ([]byte, error) {
	return json.Marshal(o)
}

func (o *Options) ToMap() map[string]interface{} {
	var m map[string]interface{}

	jsonBytes, _ := o.ToJson()
	json.Unmarshal(jsonBytes, &m)

	return m
}

type Scope int

const (
	Machine Scope = iota
	User
)

var MachineFolder string = getConfigFolder(Machine)
var UserFolder string = getConfigFolder(User)
var AppFolder string = filepath.Join("TailSpinToys", "tstoy")

const FileName string = "tstoy.config"
const FileExtension string = "json"

func getConfigFolder(scope Scope) string {
	if scope == Machine {
		return filepath.Join(xdg.ConfigDirs[0], AppFolder)
	}

	return filepath.Join(xdg.ConfigHome, AppFolder)
}
