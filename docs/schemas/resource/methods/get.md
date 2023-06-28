# DSC Resource manifest get property schema reference

Every command-based DSC Resource must define the `get` property in its DSC Resource manifest. This
property defines how DSC can get the current state of an instance of the DSC Resource.

This document describes the schema for the property.

## Metadata

| Metadata Key | Metadata Value                                                      |
|:------------:|:--------------------------------------------------------------------|
|  `$schema`   | `https://json-schema.org/draft/2020-12/schema`                      |
|    `$id`     | `https://aka.ms/dsc/schemas/resource/manifest.yaml#/properties/get` |
|    `type`    | `object`                                                            |

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

With this definition, DSC calls the `get` method for this DSC Resource by running:

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
when the operating system recognizes the application as an executable.

The manifest defines two arguments, `config` and `get`. The `input` property indicates that the
`get` command expects its input as a JSON blob from `stdin`.

Combined with the value for `executable`, DSC calls the get method for this DSC Resource by
running:

```sh
{ ... } | registry config get
```

## Required Properties

- [executable](#executable)

## Properties

### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the
application. A file extension is only required when the executable isn't recognizable by the
operating system as an executable.

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

The `input` property defines how to pass input to the DSC Resource. The value of this property must
be one of the following strings:

- `args` - Indicates that the DSC Resource expects the properties of an instance to be specified
  with command line arguments.
- `stdin` - Indicates that the DSC Resource expects a JSON blob representing an instance from
  `stdin`.

```yaml
Type:     string
Required: false
Default:  stdin
Valid Values:
  - args
  - stdin
```
