// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package update

import "strconv"

// Defines the value for the application's checkFrequency configuration. This
// is a custom type instead of a normal integer so it can implement interfaces
// and include validation methods.
type Frequency int

// The IsValid method returns true if the Frequency's value is between 1 and
// 90, inclusive. Otherwise, it returns false.
func (f Frequency) IsValid() bool {
	return isValid(int(f))
}

// The isValid private function's used to verify if an integer is between 1 and
// 90, inclusive.
func isValid(value int) bool {
	if value < 0 {
		return false
	}

	if value > 90 {
		return false
	}

	return true
}

// The Set method's required to implement the Frequency as a PFlag so you
// can use it as a command line flag argument. It converts an argument's string
// value into an integer and then to a Frequency.
//
// This method errors if the string is not an integer.
func (f *Frequency) Set(s string) error {
	v, err := strconv.ParseInt(s, 0, 64)
	*f = Frequency(v)
	return err
}

// Returns the type of the flag.
func (f *Frequency) Type() string {
	return "int"
}

// Converts the Frequency's value to a string for messaging.
func (f *Frequency) String() string {
	return strconv.Itoa(int(*f))
}
