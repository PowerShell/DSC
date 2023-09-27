---
description: JSON schema reference for the 'get' property in a DSC Resource manifest
ms.date:     09/27/2023
ms.topic:    reference
title:       DSC Resource manifest get property schema reference
---

# DSC Resource manifest get property schema reference

## Synopsis

Defines how to retrieve a DSC Resource instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.get.json
Type:          object
```

## Description

Every command-based DSC Resource must define the `get` property in its manifest. This property
defines how DSC can get the current state of a resource instance.

When defining this property, be sure to define a value for [input](#input). Even though it isn't
required, most resources need to receive input to enforce the desired state. When this property
isn't defined, DSC doesn't send any input to the resource for `get` operations. The only resources
that usually don't require any input for `get` are resources that only describe a single instance,
like the operating system information or timezone.

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

### Example 2 - Full definition

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

The `args` property defines an array of strings to pass as arguments to the command. DSC passes the
arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```

### input

The `input` property defines how to pass input to the resource. If this property isn't defined, DSC
doesn't send any input to the resource when invoking the `get` operation.

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
