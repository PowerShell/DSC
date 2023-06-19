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

func (o *Options) ToMap() map[string]any {
	var m map[string]any

	jsonBytes, _ := o.ToJson()
	json.Unmarshal(jsonBytes, &m)

	// need to remove the CheckFrequency value if it's zero, because that's
	// functionally unset.
	updates, ok := m["Updates"].(map[string]any)
	if ok {
		if updates["CheckFrequency"].(float64) == 0 {
			delete(updates, "CheckFrequency")
			m["Updates"] = updates
		}
	}

	return m
}

// Converts the viper map of settings to an instance of Options. This is
// required for handling how viper fully downcases keys. It ensures that
// the show command can display a consistent set of values from the
// configuration options.
func FromMap(data map[string]any) Options {
	options := Options{}
	updates, ok := data["updates"].(map[string]any)
	if !ok {
		return options
	}

	auto, ok := updates["automatic"].(bool)
	if ok {
		options.Updates.Automatic = auto
	}

	// if the map came from JSON, it's a float64
	floatFreq, ok := updates["checkfrequency"].(float64)
	if ok && floatFreq != 0 {
		options.Updates.CheckFrequency = int(floatFreq)
	}

	// if the map came from viper, it's an integer
	intFreq, ok := updates["checkfrequency"].(int)
	if ok && intFreq != 0 {
		options.Updates.CheckFrequency = intFreq
	}

	return options
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
