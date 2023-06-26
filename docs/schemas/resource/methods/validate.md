# DSC Resource manifest validate property schema reference

DSC Group Resources must define the `validate` property in their DSC Resource manifest. This
property defines how DSC can call the DSC Group Resource to test whether instances in the group
have valid definitions.

Always define the `validate` property for DSC Group Resources in the DSC Resource manifest.

This document describes the schema for the `validate` property.

## Metadata

| Metadata Key | Metadata Value                                                           |
|:------------:|:-------------------------------------------------------------------------|
|  `$schema`   | `https://json-schema.org/draft/2020-12/schema`                           |
|    `$id`     | `https://aka.ms/dsc/schemas/resource/manifest.yaml#/properties/validate` |
|    `type`    | `object`                                                                 |

## Examples

### Example 1 - Full definition

This example is from the `DSC/AssertionGroup` DSC Group Resource.

```json
"validate": {
  "executable": "dsc",
  "args": [
    "config",
    "validate"
  ]
}
```

It defines the executable as `dsc` with the arguments `config` and `validate`. The `validate`
method always sends the method's input as a JSON blob over `stdin`.

With this definition, DSC calls the `validate` method for this DSC Group Resource by running:

```sh
{ ... } | dsc config validate
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
