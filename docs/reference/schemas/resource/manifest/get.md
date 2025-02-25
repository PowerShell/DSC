---
description: JSON schema reference for the 'get' property in a DSC Resource manifest
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC Resource manifest get property schema reference
---

# DSC Resource manifest get property schema reference

## Synopsis

Defines how to retrieve a DSC Resource instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/resource/manifest.get.json
Type:          object
```

## Description

Every command-based DSC Resource must define the `get` property in its manifest. This property
defines how DSC can get the current state of a resource instance.

DSC sends data to the command in three ways:

1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
   JSON object without spaces or newlines between the object properties.
1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
   variable for each property in the input data object, using the name and value of the property.
1. When the `args` array includes a JSON input argument definition, DSC sends the data as a string
   representing the data as a compressed JSON object to the specified argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. You can only define one JSON input argument for a command.

You must define the `input` property, one JSON input argument in the `args` property array, or
both.

## Examples

### Example 1 - Minimal definition

This example is from the `Microsoft/OSInfo` DSC Resource.

```json
"get": {
  "executable": "osinfo"
}
```

It only defines the `executable` property. When a manifest doesn't define `args`, DSC passes no
arguments to the command. When a manifest doesn't define `input`, the default behavior is to send a
JSON blob to the command over `stdin`.

With this definition, DSC calls the `get` method for this resource by running:

```sh
{ ... } | osinfo
```

### Example 2 - Input from stdin

This example is from the `Microsoft.Windows/Registry` DSC Resource.

```json
"get": {
  "executable": "registry",
  "args": [
    "config",
    "get"
  ],
  "input": "stdin"
}
```

It defines `executable` as `registry`, rather than `registry.exe`. The extension isn't required
when the operating system recognizes the command as an executable. The manifest defines two
arguments, `config` and `get`. The `input` property indicates that the `get` command expects its
input as a JSON blob from `stdin`. Combined with the value for `executable`, DSC calls the get
method for this resource by running:

```sh
{ ... } | registry config get
```

### Example 3 - JSON input argument

This example uses a JSON input argument to send the data to the command.

```json
"get": {
  "executable": "tstoy",
  "args": [
    "config",
    "get",
    { "jsonInputArg": "--input", "mandatory": true }
  ],
}
```

It defines the executable as `tstoy`. It defines two [string arguments](#string-arguments) and one [JSON input argument](#json-input-argument). When DSC invokes the `get` operation for this resource, it passes the JSON data to the resource as a compressed JSON string to the `--input` argument.

The combined call for this operation is:

```sh
tstoy config get --input "{ ... }"
```

Where `{ ... }` represents the compressed JSON for the resource instance.

Because the `mandatory` option for the JSON input argument is set to `true`, DSC passes an empty
string to the argument when there's no data to send to the command. If the property wasn't defined,
or was defined as `false`, DSC would omit the argument entirely when there's no data to send.

## Required Properties

The `get` definition must include these properties:

- [executable](#executable)

## Properties

### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the command. A
file extension is only required when the command isn't recognizable by the operating system as an
executable.

```yaml
Type:     string
Required: true
```

### args

The `args` property defines the list of arguments to pass to the command. The arguments can be any
number of strings. If you want to pass the JSON object representing the property bag for the
resource to an argument, you can define a single item in the array as a [JSON object], indicating the
name of the argument with the `jsonInputArg` string property and whether the argument is mandatory
for the command with the `mandatory` boolean property.

```yaml
Type:     array
Required: false
Default:  []
Type:     [string, object(JSON Input Argument)]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `config` or `--format`.

```yaml
Type: string
```

#### JSON input argument

Defines an argument for the command that accepts the JSON input object as a string. DSC passes the
JSON input to the named argument when available. A JSON input argument is defined as a JSON object with the following properties:

- `jsonInputArg` (required) - the argument to pass the JSON data to for the command, like `--input`.
- `mandatory` (optional) - Indicate whether DSC should always pass the argument to the command,
  even when there's no JSON input for the command. In that case, DSC passes an empty string to the
  JSON input argument.

You can only define one JSON input argument per arguments array.

If you define a JSON input argument and an `input` kind for a command, DSC sends the JSON data both
ways:

- If you define `input` as `env` and a JSON input argument, DSC sets an environment variable for
  each property in the JSON input and passes the JSON input object as a string to the defined
  argument.
- If you define `input` as `stdin` and a JSON input argument, DSC passes the JSON input over stdin
  and as a string to the defined argument.
- If you define a JSON input argument without defining the `input` property, DSC only passes the
  JSON input as a string to the defined argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. This makes the manifest invalid. You must define the `input` property,
a JSON input argument in the `args` property array, or both.

```yaml
Type:                object
RequiredProperties: [jsonInputArg]
```

### input

The `input` property defines how to pass input to the resource. If this property isn't defined and
the definition doesn't define a [JSON input argument](#json-input-argument), DSC doesn't send any
input to the resource when invoking the `get` operation.

The value of this property must be one of the following strings:

- `env` - Indicates that the resource expects the properties of an instance to be specified as
  environment variables with the same names and casing.

  This option only supports the following data types for instance properties:

  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `array` of `integer` values
  - `array` of `number` values
  - `array` of `string` values

  For non-array values, DSC sets the environment variable to the specified value as-is. When the
  data type is an array of values, DSC sets the environment variable as a comma-delimited string.
  For example, the property `foo` with a value of `[1, 2, 3]` is saved in the `foo` environment
  variable as `"1,2,3"`.

  If the resource needs to support complex properties with an `object` value or multi-type arrays,
  set this to `stdin` instead.
- `stdin` - Indicates that the resource expects a JSON blob representing an instance from `stdin`.
  The JSON must adhere to the instance schema for the resource.

```yaml
Type:        string
Required:    false
ValidValues: [env, stdin]
```
