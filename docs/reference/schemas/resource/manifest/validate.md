# DSC Resource manifest validate property schema reference

## Synopsis

This property

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://schemas.microsoft.com/dsc/2023/07/resource/manifest.validate.json
Type           : object
```

## Description

DSC Group Resources must define the `validate` property in their DSC Resource manifest. This
property defines how DSC can call the group resource to test whether instances in the group
have valid definitions.

Always define the `validate` property for group resources in the DSC Resource manifest.

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

The `validate` definition must include these properties:

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
