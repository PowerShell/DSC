// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package ensure

import (
	"encoding/json"
	"fmt"
)

type Value int

const (
	Undefined Value = iota
	Absent
	Present
)

func (value Value) String() string {
	switch value {
	case Absent:
		return "absent"
	case Present:
		return "present"
	}

	return "unknown"
}

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

func (value Value) MarshalJSON() ([]byte, error) {
	return json.Marshal(value.String())
}

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

var FlagMap = map[Value][]string{
	Absent:  {"absent"},
	Present: {"present"},
}
