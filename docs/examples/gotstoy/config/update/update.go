// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package update

import "strconv"

type Frequency int

func (f Frequency) IsValid() bool {
	return IsValid(int(f))
}

func IsValid(value int) bool {
	if value < 0 {
		return false
	}

	if value > 90 {
		return false
	}

	return true
}

func (f *Frequency) Set(s string) error {
	v, err := strconv.ParseInt(s, 0, 64)
	*f = Frequency(v)
	return err
}

func (f *Frequency) Type() string {
	return "int"
}

func (f *Frequency) String() string {
	return strconv.Itoa(int(*f))
}
