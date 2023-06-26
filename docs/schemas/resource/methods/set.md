# DSC Resource manifest set property schema reference

To manage an instance with DSC, a DSC Resource must define the `set` property in its DSC Resource
manifest. This property defines how DSC can enforce the current state of an instance of the DSC
Resource.

When this property isn't defined, the DSC Resource can only be used to get the current state of an
instance and test whether it's in the desired state. It can't enforce desired state.

This document describes the schema for the property.

## Metadata

| Metadata Key | Metadata Value                                                      |
|:------------:|:--------------------------------------------------------------------|
|  `$schema`   | `https://json-schema.org/draft/2020-12/schema`                      |
|    `$id`     | `https://aka.ms/dsc/schemas/resource/manifest.yaml#/properties/set` |
|    `type`    | `object`                                                            |

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
  "input": "stdin",
  "preTest": true,
  "return": "state"
}
```

It defines `executable` as `registry`, rather than `registry.exe`. The extension isn't required
when the operating system recognizes the application as an executable.

The manifest defines two arguments, `config` and `set`. The value of the `input` property indicates
that the `set` command expects its input as a JSON blob from `stdin`.

Combined with the value for `executable`, DSC calls the set method for this DSC Resource by
running:

```sh
{ ... } | registry config set
```

Because the manifest defines `preTest` as `true`, DSC won't call the `test` method for the resource
before calling `set`. This setting indicates that the DSC Resource itself tests instances before
enforcing their desired state.

The manifest defines `return` as `state`, indicating that it only returns the final state of the
DSC Resource after the `set` method runs. DSC compares the desired state to the return data of this
DSC Resource to identify which of the DSC Resource's properties the `set` method enforced, if any.

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

### preTest

The `preTest` property defines whether the DSC Resource tests the instance internally before
enforcing the desired state. Set this property to `true` when the DSC Resource tests the instance.
Set this property to `false` to ensure DSC determines tests the instance instead. The default value
is `false`.

```yaml
Type:     boolean
Required: false
Default:  false
```

### return

The `return` property defines how DSC should process the output for this method. The value of this
property must be one of the following strings:

- `state` - Indicates that the DSC Resource returns only the instance's final state after the set
  operation as a JSON blob.
- `stateAndDiff` - Indicates that the DSC Resource returns the instance's final state and an array
  of property names that the DSC Resource modified.

The default value is `state`.

```yaml
Type: string
Required: false
Default:  state
Valid Values:
  - state
  - stateAndDiff
```
