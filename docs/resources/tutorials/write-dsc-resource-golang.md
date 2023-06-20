# Write a DSC Resource in Go

<!-- markdownlint-disable MD010 -->

With DSC v3, you can author [command-based DSC Resources][01] in any language. This enables you to
manage applications in the programming language you and your team prefer, or in the same language
as the application you're managing.

This tutorial describes how you can implement a DSC Resource in Go to manage an application's
configuration files. While this tutorial creates a DSC Resource to manage the fictional
[TSToy application][02], the principles apply when you author any command-based DSC Resource.

In this tutorial, you learn how to:

- Create a small Go application to use as a DSC Resource.
- Define the properties of the DSC Resource.
- Implement `get` and `set` commands for the DSC Resource.
- Write a manifest for the DSC Resource.
- Manually test the DSC Resource.

## Prerequisites

- Familiarize yourself with the structure of a [command-based DSC Resource][01].
- Read [About the TSToy application][02], install `tstoy`, and add it to your `PATH`.
- Go 1.19 or higher
- VS Code with the Go extension

## 1 - Create the DSC Resource

Create a new folder called `gotstoy` and open it in VS Code. This folder is the root folder for the
project.

```sh
mkdir ./gotstoy
code ./gotstoy
```

Open the integrated terminal in VS Code. In that terminal, initialize the folder as a Go module.

```sh
go mod init "github.com/<your_github_id>/gotstoy"
```

In this tutorial, you'll be creating a DSC Resource with [Cobra][03]. Cobra helps you create a
command line application in Go. It handles argument parsing, setting flags, shell completions, and
help.

Use the following command to install `cobra-cli`.

```sh
go install github.com/spf13/cobra-cli@latest
```

Use `cobra-cli` to scaffold the DSC Resource and add the `get` and `set` commands.

```sh
cobra-cli init
cobra-cli add get
cobra-cli add set
```

Run the following commands to get the Go modules you'll be using in addition to the standard
library.

```sh
go get github.com/thediveo/enumflag
go get github.com/TylerBrock/colorjson
go get github.com/knadh/koanf/maps
```

The `enumflag` module simplifies using enumerations as command line flags. The `colorjson` module
enables you to pretty-print output in the console. The `maps` module makes interacting with
arbitrary maps easier.

Verify that the new DSC Resource can run and has the expected commands.

```sh
go run ./main.go
```

```output
A longer description that spans multiple lines and likely contains
examples and usage of using your application. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.

Usage:
  gotstoy [command]

Available Commands:
  completion  Generate the autocompletion script for the specified shell
  get         A brief description of your command
  help        Help about any command
  set         A brief description of your command

Flags:
  -h, --help     help for gotstoy
  -t, --toggle   Help message for toggle

Use "gotstoy [command] --help" for more information about a command.
```

With the command scaffolded, you need to understand the application the DSC Resource manages before
you can implement the commands. By now, you should have read [About the TSToy application][02].

## 2 - Define the configuration settings

Create the `config` folder in the project root. Inside it, create the `config.go` file. This file
defines the configuration types and functions of the DSC Resource.

```sh
mkdir ./config
touch ./config/config.go
```

Set the package name at the top of the file to `config`:

```go
package config
```

Create a new type called `Settings` as a struct. The fields of the struct are the settings the DSC
Resource manages. In the struct, define the `Ensure`, `Scope`, `UpdateAutomatically`, and
`UpdateFrequency` fields. Set their types to `any`.

```go
type Settings struct {
	Ensure              any
	Scope               any
	UpdateAutomatically any
	UpdateFrequency     any
}
```

Defining the fields this way is convenient but also inaccurate. To make the DSC Resource more
reliable, you need to define the fields to match their purpose.

### Define Ensure

The `Ensure` field follows a common pattern in DSC for managing whether an instance of a DSC
Resource should exist. To use this pattern, the field should be an enumeration with the valid
values `absent` and `present`.

Create a new type called `Ensure` as an integer.

```go
type Ensure int
```

Define three constants of the `Ensure` type to use as enumerations: `EnsureUndefined`,
`EnsureAbsent`, and `EnsurePresent`.

```go
const (
	EnsureUndefined Ensure = iota
	EnsureAbsent
	EnsurePresent
)
```

Implement the [fmt.Stringer][04] interface for the `Ensure` type. This interface translates the
values into strings. It should return `"absent"`, `"present"`, or `"undefined"`.

```go
func (e Ensure) String() string {
	switch e {
	case EnsureAbsent:
		return "absent"
	case EnsurePresent:
		return "present"
	}

	return "undefined"
}
```

Implement a function called `ParseEnsure` that converts an input string into an `Ensure` enum and
returns an error if the input can't be parsed as `EnsurePresent` or `EnsureAbsent`.

```go
func ParseEnsure(s string) (Ensure, error) {
	switch strings.ToLower(s) {
	case "absent":
		return EnsureAbsent, nil
	case "present":
		return EnsurePresent, nil
	}

	return EnsureUndefined, fmt.Errorf(
		"unable to convert '%s' to Ensure, must be one of: absent, present",
		s,
	)
}
```

Implement the `MarshalJSON` and `UnmarshalJSON` methods for `Ensure` that convert to and from JSON
as the enum's label instead of the integer value.

```go
func (e Ensure) MarshalJSON() ([]byte, error) {
	return json.Marshal(e.String())
}

func (ensure *Ensure) UnmarshalJSON(data []byte) (err error) {
	var e string
	if err := json.Unmarshal(data, &e); err != nil {
		return err
	}
	if *ensure, err = ParseEnsure(e); err != nil {
		return err
	}

	return nil
}
```

Create a variable called `EnsureMap` to map the enumeration value to its string. This map is used
when you define the command line flags for the DSC Resource.

```go
var EnsureMap = map[Ensure][]string{
	EnsureAbsent:  {"absent"},
	EnsurePresent: {"present"},
}
```

Create a function called `EnsureFlagCompletion`. This function provides shell completion for the
command-line flags of the DSC Resource.

```go
func EnsureFlagCompletion(
	cmd *cobra.Command,
	args []string,
	toComplete string,
) ([]string, cobra.ShellCompDirective) {
	completions := []string{
		"absent\tThe configuration file shouldn't exist.",
		"present\tThe configuration file should exist.",
	}
	return completions, cobra.ShellCompDirectiveNoFileComp
}
```

Update the `Ensure` field of the `Settings` type to use the newly defined `Ensure` value instead
of `any`.

```go
type Settings struct {
	Ensure              Ensure
	Scope               any
	UpdateAutomatically any
	UpdateFrequency     any
}
```

### Define Scope

The `Scope` field of the `Settings` struct defines which instance of the `tstoy` configuration file
the DSC Resource should manage. Like `Ensure`, it should be an enumeration.

Define the `Scope` as an integer. Add constant values for the enumeration as `ScopeUndefined`,
`ScopeMachine`, and `ScopeUser`.

```go
type Scope int

const (
	ScopeUndefined Scope = iota
	ScopeMachine
	ScopeUser
)
```

`Scope` needs the same functions and methods you defined for `Ensure`, but for its own enumeration
values.

```go
func (s Scope) String() string {
	switch s {
	case ScopeMachine:
		return "machine"
	case ScopeUser:
		return "user"
	}

	return "undefined"
}

func ParseScope(s string) (Scope, error) {
	switch strings.ToLower(s) {
	case "machine":
		return ScopeMachine, nil
	case "user":
		return ScopeUser, nil
	}

	return ScopeUndefined, fmt.Errorf(
		"unable to convert '%s' to Scope, must be one of: machine, user",
		s,
	)
}

func (s Scope) MarshalJSON() ([]byte, error) {
	return json.Marshal(s.String())
}

func (scope *Scope) UnmarshalJSON(data []byte) (err error) {
	var e string
	if err := json.Unmarshal(data, &e); err != nil {
		return err
	}
	if *scope, err = ParseScope(e); err != nil {
		return err
	}

	return nil
}

var ScopeMap = map[Scope][]string{
	ScopeMachine: {"machine"},
	ScopeUser:    {"user"},
}

func ScopeFlagCompletion(
	cmd *cobra.Command,
	args []string,
	toComplete string,
) ([]string, cobra.ShellCompDirective) {
	completions := []string{
		"machine\tThe configuration file should exist.",
		"user\tThe configuration file shouldn't exist.",
	}
	return completions, cobra.ShellCompDirectiveNoFileComp
}
```

When you've implemented the `Scope` type, enumerations, methods, and functions, update the
`Scope` field of the `Settings` type.

```go
type Settings struct {
	Ensure              Ensure
	Scope               Scope
	UpdateAutomatically any
	UpdateFrequency     any
}
```

### Define UpdateAutomatically

Like `Ensure` and `Scope`, whether the `tstoy` application should be configured for automatic
updates only has two options. Unlike `Ensure` and `Scope`, you can represent those options as a
boolean.

Update the `UpdateAutomatically` field of the `Settings` type to be a pointer to a boolean value.
Using a pointer for this field allows the value to be `nil`, which enables the DSC Resource to
distinguish between the setting not being specified and being specified as `false`.

```go
type Settings struct {
	Ensure              Ensure
	Scope               Scope
	UpdateAutomatically *bool
	UpdateFrequency     any
}
```

If the value wasn't a pointer, the DSC Resource would need extra handling to distinguish between
whether the value is false because the user or configuration file specified the value as `false` or
because it wasn't specified at all.

### Define UpdateFrequency

The `UpdateFrequency` field represents a count of days between `1` and `90`, inclusive. To add
validation for the field, define a new type called `Frequency` as an integer.

```go
type Frequency int
```

Next, define the `Validate` method to check whether a `Frequency` value is valid for the setting.
It should return an error when the integer value of the Frequency is out of range.

```go
func (f Frequency) Validate() error {
	v := int(f)
	if v < 1 || v > 90 {
		return fmt.Errorf(
			"invalid value %v; must be an integer between 1 and 90, inclusive",
			v,
		)
	}
	return nil
}
```

To make the new type usable as a command line flag, you need to implement the `Set`, `Type`, and
`String` methods of the [pflag.Value interface][05].

```go
func (f *Frequency) Set(s string) error {
	v, err := strconv.ParseInt(s, 0, 64)
	if err != nil {
		return err
	}

	*f = Frequency(v)

	return f.Validate()
}

func (f *Frequency) Type() string {
	return "int"
}

func (f *Frequency) String() string {
	return strconv.Itoa(int(*f))
}
```

Finally, update the `UpdateFrequency` field of `Settings` to use the defined `Frequency` type.

```go
type Settings struct {
	Ensure              Ensure
	Scope               Scope
	UpdateAutomatically *bool
	UpdateFrequency     Frequency
}
```

### Ensure Settings serializes to JSON correctly

Now that the `Settings` type is defined and has the correct value types for each field, you need
to add tags to the fields so they can be marshalled to and unmarshalled from JSON correctly.

```go
type Settings struct {
	Ensure              Ensure    `json:"ensure,omitempty"`
	Scope               Scope     `json:"scope,omitempty"`
	UpdateAutomatically *bool     `json:"updateAutomatically,omitempty"`
	UpdateFrequency     Frequency `json:"updateFrequency,omitempty"`
}
```

The tags should all be in the format `json:"<key_name>,omitempty"`. The first value defines the
name of the key that the DSC Resource expects from JSON input and uses for returning an instance
of `Settings` to DSC. The `omitempty` value indicates that if the fields value is the same as its
zero value, that key shouldn't be included in the output JSON.

### Implement validation for Settings

The DSC Resource should be able to indicate whether an instance of Settings is valid and, if it
isn't, how it's invalid.

Add the `Validate` method to return an error if the instance is invalid.

```go
func (s *Settings) Validate() error {
	if s.Scope == ScopeUndefined {
		return fmt.Errorf(
			"the Scope setting isn't defined. Must define a Scope for Settings",
		)
	}

	if s.Ensure == EnsureAbsent {
		return nil
	}

	if s.UpdateFrequency != 0 {
		return s.UpdateFrequency.Validate()
	}

	return nil
}
```

The method returns an error if the `Scope` field is undefined because the resource requires a
specific scope to manage a `tstoy` configuration file. It short-circuits the validation if `Ensure`
is set to absent because all other keys are ignored. Finally, if the `UpdateFrequency` field is
invalid, it returns an error message indicating the issue.

## 3 - Add flags to the root command

Now that the valid values for settings are defined, the root command of the DSC Resource needs to
handle when users specify those values.

Open the `cmd/root.go` file in the editor.

Add variables for each of the fields of the `Settings` type so users can specify those values at
the command line.

```go
var targetScope config.Scope
var targetEnsure config.Ensure
var updateAutomatically bool
var updateFrequency config.Frequency
```

Next, find the `init` function at the bottom of the file. Inside it, define persistent flags so
users can pass the values to both `get` and `set`.

```go
func init() {
	rootCmd.PersistentFlags().Var(
		enumflag.New(&targetScope, "scope", config.ScopeMap, enumflag.EnumCaseInsensitive),
		"scope",
		"The target scope for the configuration.",
	)
	rootCmd.RegisterFlagCompletionFunc("scope", config.ScopeFlagCompletion)

	rootCmd.PersistentFlags().Var(
		enumflag.New(&targetEnsure, "ensure", config.EnsureMap, enumflag.EnumCaseInsensitive),
		"ensure",
		"Whether the configuration file should exist.",
	)
	rootCmd.RegisterFlagCompletionFunc("ensure", config.EnsureFlagCompletion)

	rootCmd.PersistentFlags().BoolVar(
		&updateAutomatically,
		"updateAutomatically",
		false,
		"Whether the configuration should set the app to automatically update.",
	)

	rootCmd.PersistentFlags().Var(
		&updateFrequency,
		"updateFrequency",
		"How frequently the configuration should update, between 1 and 90 days inclusive.",
	)
}
```

Use the `enumflag` module to for the `ensure` and `scope` flags. It handles parsing the user inputs
and converting them to the enumeration values. Use the flag completion functions you defined earlier
to ensure that users can opt into shell completions for those flags.

## 4 - Validate input flags

With the `Settings` defined and command line flags added to the root command, you can begin
validating that the settings flags work as expected.

Open the `cmd/get.go` file in the editor.

At the bottom of the file, create a new `getState` function that takes two parameters, a pointer to
`cobra.Command` and a slice of strings, and returns an error.

```go
func getState(cmd *cobra.Command, args []string) error {
	return nil
}
```

Replace the `Run` entry in the `getCmd` variable's definition with the `RunE` field set to the
`getState` function. Update the documentation for the command to be more specific to the DSC
Resource.

```go
var getCmd = &cobra.Command{
	Use:   "get",
	Short: "Gets the current state of a tstoy configuration file.",
	Long: `The get command returns the current state of a tstoy configuration
file as a JSON blob to stdout.`,
	RunE: getState,
}
```

Next, update the `getState` function to report the value of any specified flags. You'll replace
this implementation later, but it's useful for validating that the flags work as expected.

```go
func getState(cmd *cobra.Command, args []string) error {
	if targetScope != config.ScopeUndefined {
		fmt.Println("Specified --scope as", targetScope)
	}
	if targetEnsure != config.EnsureUndefined {
		fmt.Println("Specified --ensure as", targetEnsure)
	}
	if rootCmd.PersistentFlags().Lookup("updateAutomatically").Changed {
		fmt.Println("Specified --updateAutomatically as", updateAutomatically)
	}
	if updateFrequency != 0 {
		fmt.Println("Specified --updateFrequency as", updateFrequency)
	}
	return nil
}
```

Run the DSC Resource with different flags to verify the output.

```sh
go run ./main.go get --scope machine --ensure absent
go run ./main.go get --updateAutomatically --updateFrequency 45
go run ./main.go get --updateAutomatically=false --ensure Absent
```

```Output
Specified --ensure as absent
Specified --scope as machine

Specified --updateAutomatically as true
Specified --updateFrequency as 45

Specified --ensure as absent
Specified --updateAutomatically as false
```

Next, you can test that the arguments are validating correctly:

```sh
go run ./main.go get --scope 1
go run ./main.go get --scope incorrect
go run ./main.go get --updateFrequency 100
```

```Output
Error: invalid argument "1" for "--scope" flag: must be 'machine', 'user'

Error: invalid argument "incorrect" for "--scope" flag: must be 'machine', 'user'

Error: invalid argument "100" for "--updateFrequency" flag: invalid value 100;
must be an integer between 1 and 90, inclusive
```

With validation confirmed, you can begin implementing the `get` command.

## 5 - Implement get

To implement the get command, the DSC Resource needs to be able to find and marshal the settings
from a specific `tstoy` configuration file.

Recall from [About the TSToy application][02] that you can use the `tstoy show path` command to get
the full path to the applications configuration files. The DSC Resource can use those commands
instead of trying to generate the paths itself.

### Define get helper functions and methods

Open the `config/config.go` file. In it, add the `getAppConfigPath` function. It should take a
`Scope` value as input and return a string and error.

```go
func getAppConfigPath(s Scope) (string, error) {
	args := []string{"show", "path", s.String()}

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
```

The function generates the arguments to send to `tstoy` and calls the command.

Next, update the `Settings` struct to include a private field for the configuration path and
implement the private `GetConfigPath` function to retrieve the path for the instance of the
configuration file.

```go
type Settings struct {
	Ensure              Ensure    `json:"ensure,omitempty"`
	Scope               Scope     `json:"scope,omitempty"`
	UpdateAutomatically *bool     `json:"updateAutomatically,omitempty"`
	UpdateFrequency     Frequency `json:"updateFrequency,omitempty"`
	configPath          string
}

func (s *Settings) GetConfigPath() (string, error) {
	if s.configPath == "" {
		path, err := getAppConfigPath(s.Scope)
		if err != nil {
			return "", err
		}
		s.configPath = path
	}

	return s.configPath, nil
}
```

The `GetConfigPath` function reduces the number of calls the DSC Resource needs to make to the
application when you implement the `set` command.

Now that the DSC Resource can find the correct path, it needs to be able to retrieve settings from
the configuration file. You need to implement two more private functions:

1. `getAppConfigMap` to retrieve the configuration file settings as a generic `map[string]any`
   object.
1. `getAppConfigSettings` to convert the generic map into a `Settings` instance.

First, implement `getAppConfigMap` to read the configuration file and unmarshal the JSON.

```go
func getAppConfigMap(path string) (map[string]any, error) {
	var config map[string]any

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	err = json.Unmarshal(data, &config)
	return config, err
}
```

Next, implement `getAppConfigSettings` to convert the map into a `Settings` instance.

```go
func getAppConfigSettings(scope Scope, config map[string]any) (Settings, error) {
	// ensure the map keys are all strings.
	maps.IntfaceKeysToStrings(config)

	// Since we found the config, we know the scope and ensure state.
	settings := Settings{
		Scope:  scope,
		Ensure: EnsurePresent,
	}

	// Check for the update settings
	updates, ok := config["updates"]
	if ok {
		for key, value := range updates.(map[string]any) {
			switch key {
			case "automatic":
				auto := value.(bool)
				settings.UpdateAutomatically = &auto
			case "checkFrequency":
				intValue := int(value.(float64))
				frequency := Frequency(intValue)
				settings.UpdateFrequency = frequency
			}
		}
	}

	return settings, nil
}
```

With those private functions implemented, you can add methods to `Settings` for retrieving the map
of settings and the actual state.

```go
func (s *Settings) GetConfigMap() (map[string]any, error) {
	path, err := s.GetConfigPath()
	if err != nil {
		return nil, err
	}
	return getAppConfigMap(path)
}

func (s *Settings) GetConfigSettings() (Settings, error) {
	config, err := s.GetConfigMap()
	if errors.Is(err, os.ErrNotExist) {
		return Settings{
			Ensure: EnsureAbsent,
			Scope:  s.Scope,
		}, nil
	} else if err != nil {
		return Settings{}, err
	}

	return getAppConfigSettings(s.Scope, config)
}
```

To finish the implementation of the `get` command, add the `Print` method output the DSC Resource as
JSON.

```go
func (s *Settings) Print() error {
	configJson, err := json.Marshal(s)
	if err != nil {
		return err
	}

	fmt.Println(string(configJson))

	return nil
}
```

### Update getState to return one instance

Open the `cmd/get.go` file and return to the `getState` function. Instead of printing the inputs,
the function should:

1. Create an instance of `Settings` from the inputs.
1. Validate the instance.
1. Get the current settings from the system.
1. Print the results.

```go
func getState(cmd *cobra.Command, args []string) error {
	// Only the scope is used when retrieving current state.
	s := config.Settings{
		Scope: targetScope,
	}

	err := s.Validate()
	if err != nil {
		return fmt.Errorf("can't get settings; %s", err)
	}

	config, err := s.GetConfigSettings()
	if err != nil {
		return fmt.Errorf("failed to get settings; %s", err)
	}

	return config.Print()
}
```

Now you can run the updated command to see how it works:

```sh
go run ./main.go get
go run ./main.go get --scope machine
go run ./main.go get --scope user --ensure present
```

```Output
Error: can't get settings; the Scope setting isn't defined. Must define a Scope
for Settings

{"ensure":"absent","scope":"machine"}

{"ensure":"absent","scope":"user"}
```

### Update getState to return all instances

DSC Resources may optionally return the current state for every manageable instance. This is
convenient for users who want to get information about a resource with a single command. It's also
useful for higher-order tools that can cache current state.

To add this functionality, add the `all` variable as a boolean in `cmd/get.go`.

```go
var all bool
```

In the `init` function, add `--all` as a new flag for the command.

```go
func init() {
	rootCmd.AddCommand(getCmd)
	getCmd.Flags().BoolVar(
		&all,
		"all",
		false,
		"Get the configurations for all scopes.",
	)
}
```

Update the `getState` function to handle the new flag by making the behavior loop.

```go
func getState(cmd *cobra.Command, args []string) error {
	list := []config.Settings{}
	if all {
		list = append(
			list,
			config.Settings{Scope: config.ScopeMachine},
			config.Settings{Scope: config.ScopeUser},
		)
	} else {
		list = append(list, config.Settings{Scope: targetScope})
	}

	for _, s := range list {

		err := s.Validate()
		if err != nil {
			return fmt.Errorf("can't get settings; %s", err)
		}

		config, err := s.GetConfigSettings()
		if err != nil {
			return fmt.Errorf("failed to get settings; %s", err)
		}

		err = config.Print()
		if err != nil {
			return err
		}
	}

	return nil
}
```

Run the updated command:

```sh
go run ./main.go get --all
go run ./main.go get --scope machine
go run ./main.go get --scope user
```

```Output
{"ensure":"absent","scope":"machine"}
{"ensure":"absent","scope":"user"}

{"ensure":"absent","scope":"machine"}

{"ensure":"absent","scope":"user"}
```

## 6 - Handle JSON over `stdin`

When command-based DSC Resources are called by `dsc` itself, they may get their input as a JSON blob
over `stdin`. While specifying flags at the command line is useful for testing, it's more robust for
the DSC Resource to support sending input over `stdin`. This also makes it easier for other
integrating tools to interact with the DSC Resource.

### Add handlers for JSON and `stdin`

Create the `input` folder in the project root. Inside it, create the `input.go` file. This file
defines how you handle input from `stdin`.

```sh
mkdir ./input
touch ./input/input.go
```

Open `input/input.go` and set the package name to `input`.

```go
package input
```

Now you need to ensure that the DSC Resource can handle a JSON blob as input along with the other
flags. Implement a new type that satisfies the [pflag.Value][05] interface.

Define a type called `JSONFlag` as a struct with the `Target` field as the `any` type.

```go
type JSONFlag struct {
	Target any
}
```

Implement the `String`, `Set`, and `Type` methods for `JSONFlag`.

```go
func (f *JSONFlag) String() string {
	b, err := json.Marshal(f.Target)
	if err != nil {
		return "failed to marshal object"
	}
	return string(b)
}

func (f *JSONFlag) Set(v string) error {
	return json.Unmarshal([]byte(v), f.Target)
}

func (f *JSONFlag) Type() string {
	return "json"
}
```

Next, the DSC Resource needs a function that can handle reading from `stdin`. The function must
operate on the list of arguments for the DSC Resource. If there's input on `stdin`, it is added to
the list of arguments with the `--inputJSON` flag.

```go
func HandleStdIn(args []string) []string {
	info, _ := os.Stdin.Stat()
	if (info.Mode() & os.ModeCharDevice) == os.ModeCharDevice {
		// do nothing
	} else {
		stdin, err := io.ReadAll(os.Stdin)
		if err != nil {
			panic(err)
		}

		// remove surrounding whitespace
		jsonBlob := strings.Trim(string(stdin), "\n")
		jsonBlob = strings.Trim(jsonBlob, "\r")
		jsonBlob = strings.TrimSpace(jsonBlob)
		// only add to arguments if the string is non-empty.
		if jsonBlob != "" {
			args = append(args, "--inputJSON", jsonBlob)
		}
	}

	return args
}
```

The function doesn't need to validate that the input is valid JSON. Instead, the `JSONFlag` and the
command handle the validation. Implementing the function to append the JSON as an argument also
gives the user the choice to pass a JSON blob as a normal argument.

### Add inputJSON to the root command

Before you can pass a JSON blob to the commands, you must update the root command to accept the
`--inputJson` flag.

Open `cmd/root.go` and add the `inputJSON` variable with its type as a pointer to
`config.Settings`.

```go
var inputJSON *config.Settings
```

In the `init` function, add a new persistent flag for `--inputJSON`.

```go
rootCmd.PersistentFlags().Var(
	&input.JSONFlag{Target: &inputJSON},
	"inputJSON",
	"Specify options as a JSON blob instead of using the scope, ensure, and update* flags.",
)
```

The new flag uses the `JSONFlag` type defined in the `input` package and sets the `Target` field to
the `inputJSON` variable. Because that variable has the `Settings` type, when the flag automatically
unmarshals the input, it deserializes the blob to `Settings` for the DSC Resource.

Finally, you must update the `Execute` function to take a list of arguments, because argument
passing must be explicitly handled to support `stdin`.

### Update main to handle `stdin`

Now that the `HandleStdIn` function is defined and the root command is updated, `main` needs to be
updated to pass the arguments to the `Execute` function and handle JSON over `stdin`.

Open `main.go` and replace the `main` function.

```go
func main() {
	args := []string{}
	for index, arg := range os.Args {
		// skip the first index, because it's the application name
		if index > 0 {
			args = append(args, arg)
		}
	}

	// Check stdin and add any found JSON blob after an --inputJSON flag.
	args = input.HandleStdIn(args)

	// execute with the combined arguments
	cmd.Execute(args)
}
```

### Update get to handle inputJSON

Now that the DSC Resource can accept JSON as input over `stdin` or as an argument, the `get` command
needs to handle that input.

Open `cmd/get.go` and add logic to handle the input JSON. The function should check for `inputJSON`
after handling the `--all` scenario and handling an explicit `--scope`.

```go
func getState(cmd *cobra.Command, args []string) error {
	list := []config.Settings{}
	if all {
		list = append(
			list,
			config.Settings{Scope: config.ScopeMachine},
			config.Settings{Scope: config.ScopeUser},
		)
	} else if targetScope != config.ScopeUndefined {
		// explicit --scope overrides JSON
		list = append(list, config.Settings{Scope: targetScope})
	} else if inputJSON != nil {
		list = append(list, *inputJSON)
	} else {
		// fails but with consistent messaging
		list = append(list, config.Settings{Scope: targetScope})
	}

	for _, s := range list {

		err := s.Validate()
		if err != nil {
			return fmt.Errorf("can't get settings; %s", err)
		}

		config, err := s.GetConfigSettings()
		if err != nil {
			return fmt.Errorf("failed to get settings; %s", err)
		}

		err = config.Print()
		if err != nil {
			return err
		}
	}

	return nil
}
```

You can verify the behavior with a few commands:

```sh
go run ./main.go get --inputJSON '{ "scope": "machine" }'
go run ./main.go get --inputJSON '{ "scope": "machine" }' --scope user
'{ "scope": "machine" }'  | go run ./main.go get
'{ "scope": "machine" }'  | go run ./main.go get --scope user
'{ "ensure": "present" }' | go run ./main.go get
```

```Output
{"ensure":"absent","scope":"machine"}

{"ensure":"absent","scope":"user"}

{"ensure":"absent","scope":"machine"}

{"ensure":"absent","scope":"user"}

Error: can't get settings; the Scope setting isn't defined. Must define
a Scope for Settings
```

The DSC Resource is now fully implemented to get the current state of an instance.

## 7 - Add support for enforcing desired state

Up to this point, the DSC Resource has been primarily concerned with representing and getting the
current state of an instance. To be fully useful, it needs to be able to change a configuration file
to enforce the desired state.

### Minimally implement set

Open the `cmd/set.go` file.

At the bottom of the file, create a new `setState` function that takes two parameters, a pointer to
`cobra.Command` and a slice of strings, and returns an error.

```go
func setState(cmd *cobra.Command, args []string) error {
	return nil
}
```

Replace the `Run` entry in the `setCmd` variable's definition with the `RunE` field set to the
`setState` function. Update the documentation for the command to be more specific to the DSC
Resource.

```go
var setCmd = &cobra.Command{
	Use:   "set",
	Short: "Sets a tstoy configuration file to the desired state.",
	Long: `The set command ensures that the tstoy configuration file for a
specific scope has the desired settings. It returns the updated settings state
as a JSON blob to stdout.`,
	RunE: setState,
}
```

Next, update the `setState` function to convert the inputs into an instance of `Settings`. For now,
the function should validate the desired state and print it.

```go
func setState(cmd *cobra.Command, args []string) error {
	enforcing := config.Settings{}

	if inputJSON != nil {
		enforcing = *inputJSON
	}
	if targetScope != config.ScopeUndefined {
		enforcing.Scope = targetScope
	}
	if targetEnsure != config.EnsureUndefined {
		enforcing.Ensure = targetEnsure
	}
	if rootCmd.PersistentFlags().Lookup("updateAutomatically").Changed {
		enforcing.UpdateAutomatically = &updateAutomatically
	}
	if updateFrequency != 0 {
		enforcing.UpdateFrequency = updateFrequency
	}

	err := enforcing.Validate()
	if err != nil {
		return fmt.Errorf("can't enforce settings; %s", err)
	}

	return enforcing.Print()
}
```

Verify the behavior for the set command.

```sh
go run ./main.go set --scope machine --ensure present --updateAutomatically=false

'{
    "scope": "user",
    "ensure": "present",
    "updateAutomatically": true,
    "updateFrequency": 45
}' | go run ./main.go set

'{
    "scope": "user",
    "ensure": "present",
    "updateAutomatically": true,
    "updateFrequency": 45
}' | go run ./main.go set --ensure absent
```

```Output
{"ensure":"present","scope":"machine","updateAutomatically":false}

{"ensure":"present","scope":"user","updateAutomatically":true,"updateFrequency":45}

{"ensure":"absent","scope":"user","updateAutomatically":true,"updateFrequency":45}
```

### Implement helper functions and methods for set

At this point, the DSC Resource is able to validate the desired state. It needs to be able to
actually change the configuration files.

Open `config/config.go` and define an `Enforce` method for `Settings` that returns a pointer to an
instance of `Settings` and an error. It should:

1. Validate the settings.
1. Get the current settings for that scope.
1. Decide what it needs to do to enforce the desired state, if anything.

```go
func (s *Settings) Enforce() (*Settings, error) {
	err := s.Validate()
	if err != nil {
		return nil, err
	}

	current, err := s.GetConfigSettings()
	if err != nil {
		return nil, err
	}

	if s.Ensure == EnsureAbsent {
		// remove the config file
	}

	if current.Ensure == EnsureAbsent {
		// create the config file
	}

	// update the config file
	return s, nil
}
```

This shows that the method needs to handle three different change types for the configuration file:

1. It needs to remove the configuration file it's not supposed to exist.
1. It needs to create the configuration file when it's supposed to exist and doesn't exist.
1. It needs to update the configuration file when it's supposed to exist and does exist.

Remember that a DSC Resource should be idempotent, only making changes when required.

Implement the remove method first. It should take an instance of `Settings` as input and return
both a pointer to an instance of `Settings` and an error.

```go
func (s *Settings) remove(current Settings) (*Settings, error) {
	if current.Ensure == EnsureAbsent {
		return s, nil
	}

	// At this point, s.GetConfigPath() has already run without an error,
	// so we can rely on accessing the private field directly.
	err := os.Remove(s.configPath)
	if err != nil {
		return &current, err
	}

	return s, nil
}
```

If the file doesn't exist, the method returns the desired state and nil. If it does exist, the
method tries to delete the file. If the operation fails, it returns the current state and the error
message. If the operation succeeds, it returns the desired state and nil.

Next, implement the `create` method. It needs the same inputs and outputs as `remove`. It should
create the file and parent folders if needed, then compose the JSON for the configuration file and
write it.

```go
func (s *Settings) create(currentSettings Settings) (*Settings, error) {
	configDir := filepath.Dir(s.configPath)
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

	// Create the JSON for the tstoy configuration file.
	// Can't just marshal the Settings instance because it's a representation
	// of the settings, not a literal blob of the settings.
	settings := make(map[string]any)
	updates := make(map[string]any)
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

	configJSON, err := json.MarshalIndent(settings, "", "  ")
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
```

With `create` and `remove` implemented, the last method to implement is `update`. It needs the same
inputs and outputs as the others. It should:

1. Retrieve the actual map of settings in the configuration file.
1. Update only the settings that are out of sync.
1. Only update the configuration file if at least one setting needs enforcing.

```go
func (s *Settings) update(current Settings) (*Settings, error) {
	writeConfig := false

	currentMap, err := current.GetConfigMap()
	if err != nil {
		return nil, err
	}

	// ensure the map keys are all strings.
	maps.IntfaceKeysToStrings(currentMap)

	// Check for the update settings
	updates, ok := currentMap["updates"]
	if !ok {
		currentMap["updates"] = make(map[string]any)
		updates = currentMap["updates"]
	}

	// Only update if desired state defines UpdateAutomatically and:
	// 1. Current state doesn't define it, or
	// 2. Current state's setting doesn't match desired state.
	shouldSetUA := false
	if s.UpdateAutomatically != nil {
		if current.UpdateAutomatically == nil {
			shouldSetUA = true
		} else if *s.UpdateAutomatically != *current.UpdateAutomatically {
			shouldSetUA = true
		}
	}

	if shouldSetUA {
		writeConfig = true
		updates.(map[string]any)["automatic"] = *s.UpdateAutomatically
	} else if current.UpdateAutomatically != nil {
		updates.(map[string]any)["automatic"] = *current.UpdateAutomatically
	}

	// Only update if desired state defines UpdateFrequency and:
	// 1. Current state doesn't define it, or
	// 2. Current state's setting doesn't match desired state.
	if s.UpdateFrequency != 0 && s.UpdateFrequency != current.UpdateFrequency {
		writeConfig = true
		updates.(map[string]any)["checkFrequency"] = s.UpdateFrequency
	} else if current.UpdateFrequency != 0 {
		updates.(map[string]any)["checkFrequency"] = current.UpdateFrequency
	}

	// no changes made, leave config untouched
	if !writeConfig {
		return s, nil
	}

	currentMap["updates"] = updates.(map[string]any)

	configJson, err := json.MarshalIndent(currentMap, "", "  ")
	if err != nil {
		return &current, fmt.Errorf(
			"unable to convert updated settings to json: %s",
			err,
		)
	}

	err = os.WriteFile(s.configPath, configJson, 0750)
	if err != nil {
		return &current, fmt.Errorf(
			"unable to write updated config file: %s",
			err,
		)
	}

	return s, nil
}
```

With the `create`, `remove`, and `update` methods implemented, update the `Enforce` method to
call them as required.

```go
func (s *Settings) Enforce() (*Settings, error) {
	err := s.Validate()
	if err != nil {
		return nil, err
	}

	current, err := s.GetConfigSettings()
	if err != nil {
		return nil, err
	}

	if s.Ensure == EnsureAbsent {
		return s.remove(current)
	}

	if current.Ensure == EnsureAbsent {
		return s.create(current)
	}

	return s.update(current)
}
```

Open `cmd/set.go` and edit the `setState` function to call the `Enforce` method.

```go
func setState(cmd *cobra.Command, args []string) error {
	enforcing := config.Settings{}

	if inputJSON != nil {
		enforcing = *inputJSON
	}
	if targetScope != config.ScopeUndefined {
		enforcing.Scope = targetScope
	}
	if targetEnsure != config.EnsureUndefined {
		enforcing.Ensure = targetEnsure
	}
	if rootCmd.PersistentFlags().Lookup("updateAutomatically").Changed {
		enforcing.UpdateAutomatically = &updateAutomatically
	}
	if updateFrequency != 0 {
		enforcing.UpdateFrequency = updateFrequency
	}

	final, err := enforcing.Enforce()
	if err != nil {
		return fmt.Errorf("can't enforce settings; %s", err)
	}

	return final.Print()
}
```

With the set command fully implemented, you can verify the behavior:

1. Show TSToy's configuration information before changing any state.

   ```sh
   tstoy show
   ```

   ```Output
   Default configuration: {
     "Updates": {
       "Automatic": false,
       "CheckFrequency": 90
     }
   }
   Machine configuration: {}
   User configuration: {}
   Final configuration: {
     "Updates": {
       "Automatic": false,
       "CheckFrequency": 90
     }
   }
   ```

1. Run the `get` command to see how the DSC Resource reports on current state:

   ```sh
   go run ./main.go get --scope machine
   ```

   ```json
   {"ensure":"absent","scope":"machine"}
   ```

1. Enforce the desired state with the `set` command.

   ```sh
   go run ./main.go set --scope machine --ensure present --updateAutomatically=false
   ```

   ```json
   {"ensure":"present","scope":"machine","updateAutomatically":false}
   ```

1. Verify that the output from the `set` command matches the output from `get` after enforcing the
   desired state.

   ```sh
   go run ./main.go get --scope machine
   ```

   ```json
   {"ensure":"present","scope":"machine","updateAutomatically":false}
   ```

1. Use the `tstoy show` command to see how the configuration changes affected TSToy.

   ```sh
   tstoy show --only machine,final
   ```

   ```Output
   Machine configuration: {
     "Updates": {
       "Automatic": false
     }
   }
   Final configuration: {
     "Updates": {
       "Automatic": false,
       "CheckFrequency": 90
     }
   }
   ```

1. Enforce desired state for the user-scope configuration file.

   ```sh
   '{
       "scope": "user",
       "ensure": "present",
       "updateAutomatically": true,
       "updateFrequency": 45
   }' | go run ./main.go set
   ```

   ```json
   {"ensure":"present","scope":"user","updateAutomatically":true,"updateFrequency":45}
   ```

1. Use the `tstoy show` command to see how the configuration changes affected TSToy.

   ```sh
   tstoy show
   ```

   ```Output
   Default configuration: {
     "Updates": {
       "Automatic": false,
       "CheckFrequency": 90
     }
   }
   Machine configuration: {
     "Updates": {
       "Automatic": false
     }
   }
   User configuration: {
     "Updates": {
       "Automatic": true,
       "CheckFrequency": 45
     }
   }
   Final configuration: {
     "Updates": {
       "Automatic": true,
       "CheckFrequency": 45
     }
   }
   ```

## 8 - Build the DSC Resource

The DSC Resource is now fully implemented. To use it with DSC, you need to compile it and ensure
DSC can find it in the `PATH` environment variable.

<details>
<summary>Build on Windows</summary>

```powershell
go build -o gotstoy.exe .
$env:Path = $PWD.Path + ';' + $env:Path
```

</details>

<details>
<summary>Build on Linux or macOS</summary>

```sh
go build -o gotstoy .
export PATH=$(pwd):$PATH
```

</details>

## 9 - Author the resource manifest

The DSC Resource is now fully implemented. The last required step to use it with DSC is to author a
resource manifest. Command-based DSC Resources must have a JSON file that follows the naming
convention `<resource_name>.resource.json`. That is the manifest file for the resource. It informs
DSC and other higher-order tools about how the DSC Resource is implemented.

Create a new file called `gotstoy.resource.json` in the project folder and open it.

```sh
touch ./gotstoy.resource.json
code ./gotstoy.resource.json
```

Add basic metadata for the DSC Resource.

```json
{
    "manifestVersion": "1.0",
    "type": "TSToy.Example/gotstoy",
    "version": "0.1.0",
    "description": "A DSC Resource written in go to manage TSToy."
}
```

To inform DSC about how to get the current state of an instance, add the `get` key to the manifest.

```json
{
    "manifestVersion": "1.0",
    "type": "TSToy.Example/gotstoy",
    "version": "0.1.0",
    "description": "A DSC Resource written in go to manage TSToy.",
    "get": {
        "executable": "gotstoy",
        "args": ["get"],
        "input": "stdin"
    }
}
```

The `executable` key indicates the name of the binary `dsc` should use. The `args` key indicates
that `dsc` should call `gotstoy get` to get the current state. The `input` key indicates that `dsc`
should pass the settings to the DSC Resource as a JSON blob over `stdin`. Even though the DSC
Resource can use argument flags, setting this value to JSON makes the integration more robust and
maintainable.

Next, define the `set` key in the manifest to inform DSC how to enforce the desired state of an
instance.

```json
{
    "manifestVersion": "1.0",
    "type": "TSToy.Example/gotstoy",
    "version": "0.1.0",
    "description": "A DSC Resource written in go to manage TSToy.",
    "get": {
        "executable": "gotstoy",
        "args": ["get"],
        "input": "stdin"
    },
    "set": {
        "executable": "gotstoy",
        "args": ["set"],
        "input": "stdin",
        "preTest": false,
        "return": "state"
    }
}
```

In this section of the manifest, the `preTest` option indicates that the DSC Resource doesn't do
full validation of the DSC Resource itself inside the set command. When used with `dsc resource` or
`dsc config` commands, DSC tests instances of the DSC Resource before calling set.

This section also defines the `return` key as `state`, which indicates that the DSC Resource
returns the current state of the instance when the command finishes.

The last section of the manifest that needs to be defined is the `schema`.

### Define the resource schema

For this DSC Resource, add the JSON Schema representing valid settings in the `embedded` key. An
instance of the resource must meet these criteria:

1. The instance must be an object.
1. The instance must define the `scope` property.
1. The `scope` property must be a string and set to either `machine` or `user`.
1. If the `ensure` property is specified, must be a string and set to either `present` or `absent`.
   If `ensure` isn't specified, it should default to `present`.
1. If the `updateAutomatically` property is specified, it must be a boolean value.
1. If the `updateFrequency` property is specified, it must be an integer between `1` and `90`,
   inclusive.

```json
{
    "manifestVersion": "1.0",
    "type": "TSToy.Example/gotstoy",
    "version": "0.1.0",
    "description": "A DSC Resource written in go to manage TSToy.",
    "get": {
        "executable": "gotstoy",
        "args": ["get"],
        "input": "stdin"
    },
    "set": {
        "executable": "gotstoy",
        "args": ["set"],
        "input": "stdin",
        "preTest": false,
        "return": "state"
    },
    "schema": {
        "embedded": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Golang TSToy Resource",
            "type": "object",
            "required": [
                "scope"
            ],
            "properties": {
                "scope": {
                    "title": "Target configuration scope",
                    "description": "Defines which of TSToy's config files to manage.",
                    "type": "string",
                    "enum": [
                        "machine",
                        "user"
                    ]
                },
                "ensure": {
                    "title": "Ensure configuration file existence",
                    "description": "Defines whether the config file should exist.",
                    "type": "string",
                    "enum": [
                        "present",
                        "absent"
                    ],
                    "default": "present"
                },
                "updateAutomatically": {
                    "title": "Should update automatically",
                    "description": "Indicates whether TSToy should check for updates when it starts.",
                    "type": "boolean"
                },
                "updateFrequency": {
                    "title": "Update check frequency",
                    "description": "Indicates how many days TSToy should wait before checking for updates.",
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 90
                }
            }
        }
    }
}
```

When authoring a JSON Schema, always include the `title` and `description` keys for every property.
Authoring tools, like VS Code, use those keys to give users context.

## 10 - Validate the DSC Resource with DSC

You now have a functional DSC Resource built and added to the PATH with its manifest. You can use
it with DSC instead of calling it directly.

First, verify that DSC recognizes the DSC Resource.

```sh
dsc resource list TSToy.Example/gotstoy
```

```yaml
type: TSToy.Example/gotstoy
version: ''
path: C:\code\dsc\gotstoy\gotstoy.resource.json
directory: C:\code\dsc\gotstoy
implementedAs: Command
author: null
properties: []
requires: null
manifest:
  manifestVersion: '1.0'
  type: TSToy.Example/gotstoy
  version: 0.1.0
  description: A DSC Resource written in go to manage TSToy.
  get:
    executable: gotstoy
    args:
    - get
    input: stdin
  set:
    executable: gotstoy
    args:
    - set
    input: stdin
    preTest: false
    return: state
  schema:
    embedded:
      $schema: https://json-schema.org/draft/2020-12/schema
      title: Golang TSToy Resource
      type: object
      required:
      - scope
      properties:
        scope:
          title: Target configuration scope
          description: Defines which of TSToy's config files to manage.
          type: string
          enum:
          - machine
          - user
        ensure:
          title: Ensure configuration file existence
          description: Defines whether the config file should exist.
          type: string
          enum:
          - present
          - absent
          default: present
        updateAutomatically:
          title: Should update automatically
          description: Indicates whether TSToy should check for updates when it starts.
          type: boolean
        updateFrequency:
          title: Update check frequency
          description: Indicates how many days TSToy should wait before checking for updates.
          type: integer
          minimum: 1
          maximum: 90
```

Next, get the current state of the machine-scope configuration file.

```sh
'{ "scope": "machine" }' | dsc resource get --resource TSToy.Example/gotstoy
```

```yaml
actual_state:
  ensure: present
  scope: machine
  updateAutomatically: false
```

Remove the machine-scope configuration file.

```sh
'{
    "scope": "machine",
    "ensure": "absent"
}' | dsc resource set --resource TSToy.Example/gotstoy
```

```yaml
before_state:
  ensure: present
  scope: machine
  updateAutomatically: false
after_state:
  ensure: absent
  scope: machine
changed_properties:
- ensure
```

<!-- Need to add a section on calling dsc config when it's implemented -->

## Review

In this tutorial, you:

1. Scaffolded a new Go app as a DSC Resource.
1. Defined the configurable settings to manage the TSToy application's configuration files and
   update behavior.
1. Added flags to enable users to configure TSToy in the terminal with validation and completion
   suggestions.
1. Added handling so the DSC Resource can use JSON input with a flag or from `stdin`.
1. Implemented the `get` command to return the current state of a TSToy configuration file as an
   instance of the DSC Resource.
1. Added handling so the `get` command can retrieve every instance of the DSC Resource.
1. Implemented the `set` command to idempotently enforce the desired state for TSToy's
   configuration files.
1. Tested the DSC Resource as a standalone application.
1. Authored a DSC Resource manifest and defined a JSON Schema for instances of the DSC Resource.
1. Tested the integration of the DSC Resource with DSC itself.

At the end of this implementation, you have a functional command-based DSC Resource written in Go.

## Clean up

If you're not going to continue to work with this DSC Resource, delete the `gotstoy` folder and the
files in it.

## Next steps

1. Read about command-based DSC Resources, learn how they work, and consider why the DSC Resource
   in this tutorial is implemented this way.
1. Add a new `test` command to the DSC Resource and implement it.
1. Consider how this DSC Resource can be improved. Are there any edge cases or features it doesn't
   handle? Can you make the user experience in the terminal more delightful? Update the
   implementation with your improvements.

<!-- Fictional link for now -->
[01]: command-based-dsc-resources.md
[02]: about-tstoy.md
[03]: https://cobra.dev/
[04]: https://pkg.go.dev/fmt#Stringer
[05]: https://pkg.go.dev/github.com/spf13/pflag#Value
