# DSC Resource manifest test property schema reference

## Synopsis

Defines how to test whether a DSC Resource instance is in the desired state.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.test.json
Type           : object
```

## Description

If a DSC Resource implements its own logic for determining whether an instance is in the desired
state, it must define the `test` property in its manifest. This property defines how DSC can call
the resource to test whether an instance is in the desired state.

When this property isn't defined, DSC uses a synthetic test method for the resource. The synthetic
test method:

1. Gets the actual state of the instance using the resource's `get` method.
1. Compares every defined property of the instance's desired state to the actual state.
1. If the desired state of a property isn't equal to the actual state of that property, DSC reports
   that the instance isn't in the desired state.

Because the synthetic test only checks for equivalency, it can't accurately test resources with
properties that can't be evaluated with equivalency alone. For example, if a resource manages
package versions and allows setting the version to `latest`, DSC would report an instance with a
version of `3.1.0` as being out of the desired state, even if `3.1.0` is the latest version of the
package.

For resources with properties that can't be evaluated by equivalency alone, always define the
`test` property in the manifest.

## Examples

### Example 1 - Full definition

This example is from the `Microsoft.Windows/Registry` DSC Resource.

```json
"test": {
  "executable": "registry",
  "args": [
    "config",
    "test"
  ],
  "input": "stdin",
  "return": "state"
}
```

It defines `executable` as `registry`, rather than `registry.exe`. The extension isn't required
when the operating system recognizes the command as an executable.

The manifest defines two arguments, `config` and `test`. The value of the `input` property
indicates that the `test` command expects its input as a JSON blob from `stdin`.

Combined with the value for `executable`, DSC calls the `test` method for this resource by running:

```sh
{ ... } | registry config test
```

The manifest defines `return` as `state`, indicating that it only returns the actual state of the
resource when the `test` method runs.

## Required Properties

The `test` definition must include these properties:

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

### return

The `return` property defines how DSC should process the output for this method. The value of this
property must be one of the following strings:

- `state` - Indicates that the resource returns only the instance's actual state.
- `stateAndDiff` - Indicates that the resource returns the instance's actual state and an array of
  property names that are out of the desired state.

The default value is `state`.

```yaml
Type: string
Required: false
Default:  state
Valid Values:
  - state
  - stateAndDiff
```
