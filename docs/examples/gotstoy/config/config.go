// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/ensure"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/scope"
	"github.com/PowerShell/DSC/docs/examples/gotstoy/config/update"
	"github.com/TylerBrock/colorjson"
	"github.com/knadh/koanf/maps"
)

// An instance of Settings defines the configurable state of the resource for
// one of the application's configuration files. It uses the Scope and Ensure
// fields to determine which configuration file to manage and whether the file
// should exist. It uses the UpdateAutomatically and UpdateFrequency fields to
// determine what the values in the actual configuration file should be.
type Settings struct {
	// Defines whether the application's configuration file for this scope
	// should exist or not. This value must be Present or Absent.
	//
	// When this value is Absent, the resource deletes the file if it
	// exists.
	//
	// When this value is Present, the resource creates the file if it
	// doesn't exist or updates it in place if it does exist.
	Ensure ensure.Value `json:"ensure,omitempty"`
	// Defines the scope for the application's configuration file that this
	// instance is managing. This value must be for the Machine or User scope.
	Scope scope.Value `json:"scope,omitempty"`
	// Defines whether the application should automatically check for updates.
	//
	// This field maps to the "automatic" key in the "updates" object of the
	// application's configuration file.
	//
	// The value of this field is a pointer instead of a boolean to allow for
	// only writing the value when marshalling to JSON when it's explicitly set.
	UpdateAutomatically *bool `json:"updateAutomatically,omitempty"`
	// Defines how frequently the application should check for updates.
	//
	// This field maps to the "checkFrequency" key in the "updates" object of
	// the application's configuration file.
	//
	// This value must be a number between 1 and 90, inclusive.
	UpdateFrequency update.Frequency `json:"updateFrequency,omitempty"`
	// A private field to reduce the need to shell out to the application
	// to determine the path to the instance's configuration file.
	configPath string
}

// The Validate method checks that the Settings instance has defined values for
// Scope and Ensure. If Ensure isn't Absent, it also validates that the
// UpdateFrequency is between 1 and 90, inclusive.
//
// If the instance is valid, it returns true and no error. If the instance is
// invalid, it returns false and an error describing how the instance is
// invalid.
func (s *Settings) Validate(quiet bool) (isValid bool, err error) {
	if s.Scope == scope.Undefined {
		return false, fmt.Errorf(
			"the Scope setting isn't defined. Must define a Scope for Settings",
		)
	}
	if s.Ensure == ensure.Absent {
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

// The NewAppConfigJson method uses the fields of a Settings instance
// to create a JSON blob representing the Settings. Because the
// application's configuration file isn't an exact match of the
// Resource's settings, you can't just marshal the Settings to JSON and
// write them to the file.
//
// This method ensures the returned JSON is valid for the application.
// For example, this settings instance:
//
//	Settings{
//		UpdateAutomatically: true,
//		UpdateFrequency: 45,
//	}
//
// Is converted into the JSON:
//
//	`
//	{
//		"updates": {
//			"automatic": true,
//			"checkFrequence": 45
//		}
//	}
//	`
func (s *Settings) NewAppConfigJson() ([]byte, error) {
	settings := make(map[string]interface{})
	updates := make(map[string]interface{})
	addUpdates := false
	if s.UpdateAutomatically != nil {
		addUpdates = true
		updates["automatic"] = *s.UpdateAutomatically
	}
	if s.UpdateFrequency != 0 {
		addUpdates = true
		updates["checkFrequency"] = s.UpdateFrequency
	}

	if addUpdates {
		settings["updates"] = updates
	}

	return json.MarshalIndent(settings, "", "  ")
}

// The GetAppConfigPath method retrieves the path to the application's
// configuration file for a specified Scope. It shells out to the
// application, running `tstoy show path <scope_name>` and returning
// the trimmed value.
//
// If the application can't be found on the path, or the command errors,
// the method returns an empty string and the error message.
func GetAppConfigPath(targetScope scope.Value) (string, error) {
	args := []string{"show", "path"}

	switch targetScope {
	case scope.Machine:
		args = append(args, "machine")
	case scope.User:
		args = append(args, "user")
	}

	output, err := exec.Command("tstoy", args...).Output()
	if err != nil {
		return "", err
	}

	// We need to trim trailing whitespace automatically emitted for the path.
	path := string(output)
	path = strings.Trim(path, "\n")
	path = strings.Trim(path, "\r")

	return path, nil
}

// The getConfigPath private method is a convenience method that
// returns the path to the configuration file for a Settings instance's
// Scope if it's already defined. If it isn't, the method retrieves it
// and stores the value.
//
// This method should only be called from methods after you've
// confirmed that the path is retrievable, because it swallows any
// errors.
func (s *Settings) getConfigPath() string {
	if s.configPath == "" {
		s.configPath, _ = GetAppConfigPath(s.Scope)
	}

	return s.configPath
}

// The GetAppConfigMap function retrieves the configuration file for a
// specific scope and returns the map for the JSON in that file. If the
// path to the configuration file for the scope can't be determined, or
// if the configuration file doesn't exist, the function passes the
// error back to the caller.
//
// This function has no specific error handling so the caller can
// distinguish between the underlying errors and make its own decisions.
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

// The GetAppConfigurationSettings function retrieves the configuration
// file for a specific scope and converts its data into an instance of
// Settings.
//
// If the file doesn't exist, the returned instance defines the Scope
// as the target scope and Ensure as Absent.
//
// If the file exists, the returned instance defines the Scope as the
// target scope, Ensure as Present, and only defines the
// UpdateAutomatically and UpdateFrequency fields if they were set in
// the configuration file. The function ignores all unmanaged settings
// in the configuration file.
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
				auto := value.(bool)
				configSettings.UpdateAutomatically = &auto
			case "checkFrequency":
				intValue := int(value.(float64))
				frequency := update.Frequency(intValue)
				configSettings.UpdateFrequency = frequency
			}
		}
	}

	return configSettings, nil
}

// The Enforce method applies the options defined in an instance of
// Settings to the application's configuration file. It creates,
// removes, or updates the file as required. It always acts on the
// configuration file for the instance's Scope.
//
// When Ensure is Absent, the method removes the configuration file if
// it exists.
//
// When Ensure is Present, the method creates the configuration file if
// it doesn't exist. If the file exists, the method updates the values
// in the configuration file in place, only altering the settings that
// are specified in the instance. All other values, including unmanaged
// values, are left intact. However, because the ordering of the keys
// isn't guaranteed when converting the data to and from the file on
// disk, the order of the JSON keys may update, even when the value of
// those keys is untouched.
func (s *Settings) Enforce() (*Settings, error) {
	_, err := s.Validate(false)
	if err != nil {
		return nil, err
	}

	currentSettings, err := GetAppConfigSettings(s.Scope)
	if err != nil {
		return nil, err
	}

	if s.Ensure == ensure.Absent {
		return s.remove(currentSettings)
	}

	if currentSettings.Ensure == ensure.Absent {
		return s.create(currentSettings)
	}

	return s.update(currentSettings)
}

// The remove method deletes the configuration file for an instance's
// scope. This is only called when the configuration file exists and
// the instance's Ensure value is set to Absent.
func (s *Settings) remove(currentSettings Settings) (*Settings, error) {
	if currentSettings.Ensure == ensure.Absent {
		return s, nil
	}
	err := os.Remove(s.getConfigPath())
	if err != nil {
		return &currentSettings, err
	}

	return s, nil
}

// The create method creates the configuration file for an instance's
// Scope. This is only called when the configuration file doesn't
// already exist. It creates the configuration file and its parent
// folders if needed.
//
// It then converts the Settings instance into a new JSON blob for the
// app, using its own fields to determine which keys must be set.
// Finally, it writes the JSON blob to the newly created file.
func (s *Settings) create(currentSettings Settings) (*Settings, error) {
	configDir := filepath.Dir(s.getConfigPath())
	if err := os.MkdirAll(configDir, 0750); err != nil {
		return &currentSettings, fmt.Errorf(
			"failed to create folder for config file in '%s': %s",
			configDir,
			err,
		)
	}
	configFile, err := os.Create(s.configPath)
	if err != nil {
		return &currentSettings, fmt.Errorf(
			"failed to create config file '%s': %s",
			s.configPath,
			err,
		)
	}

	configJSON, err := s.NewAppConfigJson()
	if err != nil {
		return &currentSettings, fmt.Errorf(
			"unable to convert settings to json: %s",
			err,
		)
	}

	_, err = configFile.Write(configJSON)
	if err != nil {
		return &currentSettings, fmt.Errorf(
			"unable to write config file: %s",
			err,
		)
	}

	return s, nil
}

// The update method updates the TSToy application's configuration file for the
// specified Scope in place. It doesn't alter any settings in the file that
// haven't been explicitly set in the instance of Settings that's calling this
// method.
//
// First, it retrieves the current map of settings from the file system. Then
// it overrides the updates.automatic and updates.checkFrequency settings in
// that map with the values of UpdateAutomatically and UpdateFrequency if those
// values are set in the instance.
//
// Then it overwrites the existing file with the new JSON from the updated map.
func (s *Settings) update(currentSettings Settings) (*Settings, error) {
	currentMap, err := GetAppConfigMap(s.Scope)
	if err != nil {
		return nil, err
	}

	// ensure the map keys are all strings.
	maps.IntfaceKeysToStrings(currentMap)

	// Check for the update settings
	updates, ok := currentMap["updates"]
	if !ok {
		currentMap["updates"] = make(map[string]interface{})
		updates = currentMap["updates"]
	}

	if s.UpdateAutomatically != nil {
		updates.(map[string]interface{})["automatic"] = *s.UpdateAutomatically
	}
	if s.UpdateFrequency != 0 {
		updates.(map[string]interface{})["checkFrequency"] = s.UpdateFrequency
	}
	currentMap["updates"] = updates.(map[string]interface{})

	configJson, err := json.MarshalIndent(currentMap, "", "  ")
	if err != nil {
		return &currentSettings, fmt.Errorf(
			"unable to convert updated settings to json: %s",
			err,
		)
	}

	err = os.WriteFile(s.getConfigPath(), configJson, 0750)
	if err != nil {
		return &currentSettings, fmt.Errorf(
			"unable to write updated config file: %s",
			err,
		)
	}

	return s, nil
}

// The Print method prints the JSON representation of the instance. If the
// pretty parameter is true, it pretty prints the JSON with indentation and
// color. If pretty is false, it prints the condensed json as a single line.
func (s *Settings) Print(pretty bool) error {
	if pretty {
		return s.printPretty()
	}

	return s.printSimple()
}

// Pretty prints an instance of Settings as colorized JSON with indenting.
func (s *Settings) printPretty() error {
	settingsMap := make(map[string]any)
	settingsMap["scope"] = s.Scope.String()
	settingsMap["ensure"] = s.Ensure.String()
	if s.UpdateAutomatically != nil {
		settingsMap["updateAutomatically"] = *s.UpdateAutomatically
	}
	if s.UpdateFrequency != 0 {
		settingsMap["updateFrequency"] = float64(s.UpdateFrequency)
	}

	formatter := colorjson.NewFormatter()
	formatter.Indent = 2

	pretty, err := formatter.Marshal(settingsMap)
	if err != nil {
		return err
	}

	fmt.Println(string(pretty))

	return nil
}

// Prints an instance of Settings as compressed one-line JSON.
func (s *Settings) printSimple() error {
	configJson, err := json.Marshal(s)
	if err != nil {
		return err
	}

	fmt.Println(string(configJson))

	return nil
}
