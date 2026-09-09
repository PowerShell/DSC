---
description: JSON schema reference for the 'set' property in a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource manifest set property schema reference
---

# DSC Resource manifest set property schema reference

## Synopsis

Defines how to enforce state for a DSC Resource instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.set.json
Type:          object
```

## Description

To manage an instance with DSC, a DSC Resource must define the `set` property in its manifest. This
property defines how DSC can enforce the current state of an instance. When this property is
defined, the resource has the `set` capability.

When this property isn't defined, DSC can only get the current state of instances and test whether
they're in the desired state. DSC can't enforce desired state for the resource.

DSC sends data to the command in three ways:

1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
   JSON object without spaces or newlines between the object properties.
1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
   variable for each property in the input data object, using the name and value of the property.
1. When the `args` array includes a JSON input argument definition, DSC sends the data as a string
   representing the data as a compressed JSON object to the specified argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
desired state to the resource. You can only define one JSON input argument for a command.

You must define the `input` property, one JSON input argument in the `args` property array, or
both.

When the `args` array includes a [what-if argument](#what-if-argument), the resource has the
`setWhatIf` capability. DSC calls the `set` command with the what-if argument when a user invokes
the operation in what-if mode, like with the [--what-if][01] option for the `dsc config set`
command, and the resource reports how it would change the instance without modifying the system.
When the `args` array doesn't include a what-if argument, DSC synthesizes the what-if result from
the `test` operation, unless the manifest defines the deprecated [whatIf][02] property.

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

### Example 2 - What-if argument

This example defines a what-if argument so that the resource can report the expected result of the
`set` operation without changing the system.

```json
"set": {
  "executable": "my_app",
  "args": [
    "config",
    "set",
    { "whatIfArg": "--what-if" }
  ],
  "input":  "stdin",
  "return": "state"
}
```

When a user invokes the `set` operation normally, DSC calls the command as:

```sh
{ ... } | my_app config set
```

When a user invokes the `set` operation in what-if mode, DSC calls the command as:

```sh
{ ... } | my_app config set --what-if
```

Because the manifest defines the what-if argument, the resource has the `setWhatIf` capability.

## Required properties

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

The `args` property defines the list of arguments to pass to the command. Each item in the array
must be a string or an object that defines one of the following argument kinds:

- [String arguments](#string-arguments) - A static argument, like `config` or `--format`.
- [Adapted content argument](#adapted-content-argument) - The inline content of an adapted
  resource.
- [JSON input argument](#json-input-argument) - The JSON object representing the property bag for
  the resource instance.
- [Resource path argument](#resource-path-argument) - The path to the resource being invoked.
- [Resource type argument](#resource-type-argument) - The fully qualified type name of the resource
  being invoked.
- [Resource version argument](#resource-version-argument) - The version of the resource being
  invoked.
- [What-if argument](#what-if-argument) - The argument to pass when the operation runs in what-if
  mode.

DSC passes the arguments to the command in the order they're defined. For every argument kind
except string arguments and the what-if argument, DSC passes the argument name followed by its
value as two separate arguments.

```yaml
Type:      array
Required:  false
Default:   []
ItemsType: [string, object]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `config` or `--format`.

```yaml
Type: string
```

#### Adapted content argument

Defines an argument for the command that accepts the inline content of an adapted resource as a
compressed JSON string. An adapted resource manifest can define the adapted resource inline with
its `content` property instead of pointing to a file with its `path` property. When the adapted
resource defines inline content, DSC passes the content to the named argument. When it doesn't,
DSC passes the argument name without a value.

This argument kind is only useful for [resource adapters][03]. This argument kind was added in DSC
version 3.3.0.

- `adaptedContentArg` (required) - The argument to pass the adapted content to for the command,
  like `--content`.

```yaml
Type:               object
RequiredProperties: [adaptedContentArg]
```

#### JSON input argument

Defines an argument for the command that accepts the JSON input object as a string. DSC passes the
JSON input to the named argument when available.

- `jsonInputArg` (required) - The argument to pass the JSON data to for the command, like
  `--input`.
- `mandatory` (optional) - Indicates whether DSC should always pass the argument to the command,
  even when there's no JSON input for the command. In that case, DSC passes an empty string to the
  JSON input argument. The default value is `false`.

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
input JSON to the resource. You must define the `input` property, a JSON input argument in the
`args` property array, or both.

```yaml
Type:               object
RequiredProperties: [jsonInputArg]
```

#### Resource path argument

Defines an argument for the command that accepts the path to the resource being invoked. For
resource adapters, this is the value of the [path][04] property that the adapter returned for the
adapted resource when DSC listed the adapter's resources. Use this argument kind to tell the
adapter which file defines the adapted resource, like the path to a PowerShell module.

- `resourcePathArg` (required) - The argument to pass the resource path to for the command, like
  `-ResourcePath`.
- `includeQuotes` (optional) - Indicates whether DSC should wrap the path in double quotes before
  passing it to the command. Set this to `true` when the path might contain spaces. The default
  value is `false`.

```yaml
Type:               object
RequiredProperties: [resourcePathArg]
```

#### Resource type argument

Defines an argument for the command that accepts the fully qualified type name of the resource
being invoked. For resource adapters, this is the type name of the adapted resource. Use this
argument kind to implement an adapter that operates on a single adapted resource instance instead
of processing the full configuration.

- `resourceTypeArg` (required) - The argument to pass the type name to for the command, like
  `-ResourceType`.

```yaml
Type:               object
RequiredProperties: [resourceTypeArg]
```

#### Resource version argument

Defines an argument for the command that accepts the version of the resource being invoked. For
resource adapters, this is the version of the adapted resource. This argument kind was added in
DSC version 3.3.0.

- `resourceVersionArg` (required) - The argument to pass the version to for the command, like
  `-ResourceVersion`.

```yaml
Type:               object
RequiredProperties: [resourceVersionArg]
```

#### What-if argument

Defines the argument to pass to the command when a user invokes the `set` operation in what-if
mode, like with the [--what-if][01] option for the `dsc config set` command. DSC only passes the
named argument when the operation runs in what-if mode. When the operation runs normally, DSC omits
the argument.

When the `args` array includes a what-if argument, the resource has the `setWhatIf` capability.
In what-if mode, DSC calls the `set` command with the argument instead of synthesizing the result
from the `test` operation. The resource must return the expected result of the operation without
changing the system. For more information about the expected output, see
[DSC resource what-if operation stdout schema reference][05].

- `whatIfArg` (required) - The argument to pass in what-if mode, like `--what-if`.

```yaml
Type:               object
RequiredProperties: [whatIfArg]
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
[_exist][06] property in the `set` operation. The default value is `false`.

Set this property to `true` when the resource meets the following implementation requirements:

- The resource's [instance schema][07] defines the `_exist` property as a valid instance property.
- The resource's `set` command handles creating, updating, and deleting an instance based on the
  current state of the instance and the value of the `_exist` property in the desired state.

When this property is set to `true`, the resource indicates that it has the `setHandlesExist`
[capability][08]. When processing resources with the `setHandlesExist` capability in a
configuration, DSC calls the `set` operation for the resource when an instance defines `_exist` as
`false`. Without this capability, a resource must define the [delete][09] operation to support
removing instances of the resource.

If a resource manifest doesn't define this property as `true` and doesn't define the `delete`
operation, DSC raises an error when it encounters an instance of the resource with `_exist` set to
`false`.

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

When this property isn't defined, DSC doesn't expect the resource to return any output for the
`set` operation. Instead, DSC invokes the `get` operation for the resource after the `set`
operation concludes and compares the result to the state of the instance before the operation to
determine which properties the operation changed. For more information, see
[DSC resource set operation stdout schema reference][10].

```yaml
Type:        string
Required:    false
ValidValues: [state, stateAndDiff]
```

### requireSecurityContext

The `requireSecurityContext` property defines the security context the resource requires for the
`set` operation. Before invoking the command, DSC compares the current security context to this
value and raises an error if the context doesn't satisfy the requirement. The value must be one of
the following strings:

- `current` - DSC invokes the command in any security context. This is the default behavior.
- `elevated` - DSC only invokes the command when it's running in an elevated security context,
  like as an administrator on Windows or as `root` on Linux and macOS. Otherwise, DSC raises an
  error.
- `restricted` - DSC only invokes the command when it's running in a non-elevated security
  context. Otherwise, DSC raises an error.

```yaml
Type:        string
Required:    false
Default:     current
ValidValues: [current, elevated, restricted]
```

### whatIfReturns

The `whatIfReturns` property defines how DSC should process the output for this method when a user
invokes the `set` operation in what-if mode. When this property is defined, it overrides the
[return](#return) property during what-if execution. When this property isn't defined, DSC
processes the output in what-if mode the same way it processes the output for an actual `set`
operation.

Define this property when the resource returns differently shaped data in what-if mode than it
returns for an actual `set` operation. The value must be one of the same strings as the `return`
property:

- `state` - Indicates that the resource returns only the instance's expected final state after the
  set operation as a JSON blob.
- `stateAndDiff` - Indicates that the resource returns the instance's expected final state and an
  array of property names that the resource would modify.

```yaml
Type:        string
Required:    false
ValidValues: [state, stateAndDiff]
```

<!-- Link reference definitions -->
[01]: ../../../cli/config/set.md#-w---what-if
[02]: whatif.md
[03]: adapter.md
[04]: ../stdout/list.md#path
[05]: ../stdout/whatIf.md
[06]: ../properties/exist.md
[07]: ./root.md#schema-1
[08]: ../../definitions/resourceCapabilities.md
[09]: ./delete.md
[10]: ../stdout/set.md
