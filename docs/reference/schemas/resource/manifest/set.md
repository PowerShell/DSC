---
description: JSON schema reference for the 'set' property in a DSC Resource manifest
ms.date:     09/27/2023
ms.topic:    reference
title:       DSC Resource manifest set property schema reference
---

# DSC Resource manifest set property schema reference

## Synopsis

Defines how to enforce state for a DSC Resource instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/resource/manifest.set.json
Type:          object
```

## Description

To manage an instance with DSC, a DSC Resource must define the `set` property in its manifest. This
property defines how DSC can enforce the current state of an instance.

When this property isn't defined, DSC can only get the current state of instances and test whether
they're in the desired state. DSC can't enforce desired state for the resource.

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

The `args` property defines an array of strings to pass as arguments to the command. DSC passes the
arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```

### input

The `input` property defines how to pass input to the resource. If this property isn't defined, DSC
doesn't send any input to the resource when invoking the `set` operation.

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
