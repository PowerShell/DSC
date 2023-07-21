# DSC Resource manifest set property schema reference

## Synopsis

Defines how to enforce state for a DSC Resource instance.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://schemas.microsoft.com/dsc/2023/07/resource/manifest.set.json
Type           : object
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
  "input": "stdin",
  "preTest": true,
  "return": "state"
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

Because the manifest defines `preTest` as `true`, DSC won't call the `test` method for the resource
before calling `set`. This setting indicates that the resource itself tests instances before
enforcing their desired state.

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

The `input` property defines how to pass input to the resource. The value of this property must
be one of the following strings:

- `args` - Indicates that the resource expects the properties of an instance to be specified
  with command line arguments. This option isn't implemented yet.
- `stdin` - Indicates that the resource expects a JSON blob representing an instance from
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

The `preTest` property defines whether the resource tests the instance internally before
enforcing the desired state. Set this property to `true` when the resource tests the instance.
Set this property to `false` to ensure DSC tests the instance first instead. The default value
is `false`.

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
Type: string
Required: false
Default:  state
Valid Values:
  - state
  - stateAndDiff
```
