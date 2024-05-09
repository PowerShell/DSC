---
description: JSON schema reference for the 'set' property in a DSC Resource manifest
ms.date:     01/17/2024
ms.topic:    reference
title:       DSC Resource manifest set property schema reference
---

# DSC Resource manifest set property schema reference

## Synopsis

Defines how to enforce state for a DSC Resource instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/resource/manifest.set.json
Type:          object
```

## Description

To manage an instance with DSC, a DSC Resource must define the `set` property in its manifest. This
property defines how DSC can enforce the current state of an instance.

When this property isn't defined, DSC can only get the current state of instances and test whether
they're in the desired state. DSC can't enforce desired state for the resource.

DSC sends data to the command in three ways:

  1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
     JSON object without spaces or newlines between the object properties.
  1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
     variable for each property in the input data object, using the name and value of the property.
  1. When the `args` array includes a JSON input argument definition, DSC sends the data as a
     string representing the data as a compressed JSON object to the specified argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. You can only define one JSON input argument for a command.

You must define the `input` property, one JSON input argument in the `args` property array, or
both.

## Examples

### Example 1 - Full definition

This example is from the `Microsoft.Windows/Registry` DSC Resource.

```json
"set": {
  "executable": "registry",
  "args": [
    "config",
    "set"
  ],
  "input":            "stdin",
  "implementsPretest": true,
  "return":           "state"
}
```

It defines `executable` as `registry`, rather than `registry.exe`. The extension isn't required
when the operating system recognizes the command as an executable.

The manifest defines two arguments, `config` and `set`. The value of the `input` property indicates
that the `set` command expects its input as a JSON blob from `stdin`.

Combined with the value for `executable`, DSC calls the set method for this resource by
running:

```sh
{ ... } | registry config set
```

Because the manifest defines `implementsPretest` as `true`, DSC won't call the `test` method for
the resource before calling `set`. This setting indicates that the resource itself tests instances
before enforcing their desired state.

The manifest defines `return` as `state`, indicating that it only returns the final state of the
resource after the `set` method runs. DSC compares the desired state to the return data of this
resource to identify which of the resource's properties the `set` method enforced, if any.

## Required Properties

The `set` definition must include these properties:

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
input to the resource when invoking the `set` operation.

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

### implementsPretest

The `implementsPretest` property defines whether the resource tests whether the instance is in the
desired state internally before enforcing the desired state. Set this property to `true` when the
resource tests the instance as part of the `set` operation. Set this property to `false` when it
doesn't.

When this value is `false`, it indicates that users should always call `dsc resource test` against
the instance before invoking the `dsc resource set` command against the resource.

The default value is `false`.

```yaml
Type:     boolean
Required: false
Default:  false
```

### handlesExist

The `handlesExist` property defines whether the resource has built-in handling for the
[_exist][01] property in the `set` operation. The default value is `false`.

Set this property to `true` when the resource meets the following implementation requirements:

- The resource's [instance schema][02] defines the `_exist` property as a valid instance property.
- The resource's `set` command handles creating, updating, and deleting an instance based on the
  current state of the instance and the value of the `_exist` property in the desired state.

When this property is set to `true`, the resource indicates that it has the [SetHandlesExist][03]
[capability][04]. When processing resources with the `SetHandlesExist` capability in a
configuration, DSC calls the `set` operation for the resource when an instance defines `_exist` as
`false`. Without this capability, a resource must define the [delete][05] operation to support
removing instances of the resource.

If a resource manifest doesn't define this property as `true` and doesn't define the `delete`
operation, DSC raises an error when it encounters an instance of the resource with `_exist` set to
`false`.

### return

The `return` property defines how DSC should process the output for this method. The value of this
property must be one of the following strings:

- `state` - Indicates that the resource returns only the instance's final state after the set
  operation as a JSON blob.
- `stateAndDiff` - Indicates that the resource returns the instance's final state and an array of
  property names that the resource modified.

The default value is `state`.

```yaml
Type:        string
Required:    false
Default:     state
ValidValues: [state, stateAndDiff]
```

<!-- Reference link definitions -->
[01]: ../properties/exist.md
[02]: ./root.md#schema-1
[03]: ../../outputs/resource/list.md#capability-sethandlesexist
[04]: ../../outputs/resource/list.md#capabilities
[05]: ./delete.md
