// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/ensure"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/scope"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/update"
	"github.com/knadh/koanf/maps"
)

type Settings struct {
	Ensure              ensure.Value     `json:"ensure,omitempty" mapstructure:"ensure,omitempty"`
	Scope               scope.Value      `json:"scope,omitempty" mapstructure:"scope,omitempty"`
	UpdateAutomatically bool             `json:"updateAutomatically,omitempty" mapstructure:"update_automatically,omitempty"`
	UpdateFrequency     update.Frequency `json:"updateFrequency,omitempty" mapstructure:"update_frequency,omitempty"`
}

func (s *Settings) Validate(quiet bool) (isValid bool, err error) {
	if s.Ensure == ensure.Absent {
		return true, nil
	}
	if !s.UpdateAutomatically {
		return true, nil
	}

	if s.UpdateFrequency.IsValid() {
		return true, nil
	}

	isValid = false
	err = fmt.Errorf(
		"invalid setting, update frequency (%v)isn't between 0-90, inclusive",
		s.UpdateFrequency,
	)
	return
}

func GetAppConfigPath(targetScope scope.Value) (string, error) {
	args := []string{"show", "path"}

	switch targetScope {
	case scope.Machine:
		args = append(args, "machine")
	case scope.User:
		args = append(args, "user")
	}

	output, err := exec.Command("tstoy.exe", args...).Output()
	if err != nil {
		return "", err
	}

	// We need to trim trailing whitespace automatically emitted for the path.
	path := string(output)
	path = strings.Trim(path, "\n")
	path = strings.Trim(path, "\r")

	return path, nil
}

func GetAppConfigMap(targetScope scope.Value) (map[string]interface{}, error) {
	var configMap map[string]interface{}

	// Retrieve the actual path to the configuration the app expects
	path, err := GetAppConfigPath(targetScope)
	if err != nil {
		return nil, err
	}

	configData, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	err = json.Unmarshal(configData, &configMap)
	return configMap, err
}

func GetAppConfigSettings(targetScope scope.Value) (Settings, error) {
	configMap, err := GetAppConfigMap(targetScope)
	if errors.Is(err, os.ErrNotExist) {
		return Settings{
			Ensure: ensure.Absent,
			Scope:  targetScope,
		}, nil
	} else if err != nil {
		return Settings{}, err
	}

	// ensure the map keys are all strings.
	maps.IntfaceKeysToStrings(configMap)

	// Since we found the config, we know the scope and ensure state.
	configSettings := Settings{
		Scope:  targetScope,
		Ensure: ensure.Present,
	}

	// Check for the update settings
	updates, ok := configMap["updates"]
	if ok {
		for key, value := range updates.(map[string]interface{}) {
			switch key {
			case "automatic":
				configSettings.UpdateAutomatically = value.(bool)
			case "checkFrequency":
				intValue := int(value.(float64))
				frequency := update.Frequency(intValue)
				configSettings.UpdateFrequency = frequency
			}
		}
	}

	return configSettings, nil
}
