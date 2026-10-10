---
description: JSON schema reference for the 'whatIf' property in a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource manifest whatIf property schema reference
---

# DSC Resource manifest whatIf property schema reference

## Synopsis

Defines how to indicate whether and how the set command will modify an instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.whatIf.json
Type:          object
```

## Description

When enforcing a configuration document with the [dsc config set][01] command, users can specify
the [--what-if][02] option to see whether and how resources will change system state without
actually doing so. This property defines how DSC can invoke the resource to return that information
directly.

The `whatIf` property has the same schema and shape as the [set][03] property. Every property that
you can define for the `set` method is valid for the `whatIf` method.

> [!IMPORTANT]
> Defining a separate `whatIf` command is deprecated. Instead, define a [what-if argument][04] in
> the `args` array of the `set` property. When the `set` definition includes a what-if argument,
> DSC ignores the `whatIf` property. When the `set` definition doesn't include a what-if argument
> and the manifest defines `whatIf`, DSC calls the `whatIf` command in what-if mode and emits a
> warning.

When the manifest defines neither a what-if argument nor the `whatIf` property, DSC synthesizes
this behavior by converting the result of a test operation against the resource into a set result.
The synthetic result can only indicate how the operation will change the resource properties. It
can't indicate whether the `set` operation will fail due to invalid parameters or which read-only
properties the resource will return from the operation. The following list describes a few cases
where a synthetic what-if result won't return sufficient information to the user:

- A resource requiring a credential parameter might successfully test the instance but not have
  permissions to modify it. In this case, the user might run `dsc config set --what-if` and see an
  apparently successful prediction for the resource. Then, when they run the command without the
  `--what-if` option, the resource raises an error that the user has to investigate. If any other
  resources applied successfully before the instance that failed, the system might be left in a
  partially-configured state.
- A resource with a dependency service won't be able to report whether that service needs to be
  restarted from a synthetic result. After reviewing the impact of the configuration based on the
  what-if result, a user might then inadvertently restart a service or leave the configuration in a
  partially-configured state until that service is rebooted.

If your resource uses parameters or returns read-only properties from a `set` operation, define a
what-if argument for the `set` method to ensure your users get the best information about whether
and how the resource will modify system state in what-if mode.

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

## Examples

### Example 1 - Full definition

```json
"whatIf": {
  "executable": "my_app",
  "args": [
    "config",
    "set",
    "--what-if"
  ],
  "input":  "stdin",
  "return": "state"
}
```

It defines `executable` as `my_app`, rather than `my_app.exe`. The extension isn't required when
the operating system recognizes the command as an executable.

The manifest defines three arguments, `config`, `set`, and `--what-if`. The value of the `input`
property indicates that the `whatIf` command expects its input as a JSON blob from `stdin`.

Combined with the value for `executable`, DSC calls the what-if method for this resource by
running:

```sh
{ ... } | my_app config set --what-if
```

The manifest defines `return` as `state`, indicating that it only returns the expected final state
of the resource after the `set` method runs. DSC compares the desired state to the return data of
this resource to identify which of the resource's properties the `set` method will enforce, if any.

### Example 2 - Equivalent definition with a what-if argument

This example defines the same behavior as the previous example with a what-if argument for the
`set` method instead of a separate `whatIf` definition. This is the recommended approach.

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

DSC only passes the `--what-if` argument to the command when a user invokes the `set` operation in
what-if mode. Because the manifest defines the what-if argument, the resource has the `setWhatIf`
capability.

## Required properties

The `whatIf` definition must include these properties:

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

This argument kind is only useful for [resource adapters][05]. This argument kind was added in DSC
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
resource adapters, this is the value of the [path][06] property that the adapter returned for the
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

Defines the argument to pass to the command when the operation runs in what-if mode. DSC always
invokes the `whatIf` command in what-if mode, so DSC always passes the named argument to the
command. Define the what-if argument for the [set method][04] instead of the `whatIf` method to
use a single command for both modes.

- `whatIfArg` (required) - The argument to pass in what-if mode, like `--what-if`.

```yaml
Type:               object
RequiredProperties: [whatIfArg]
```

### input

The `input` property defines how to pass input to the resource. If this property isn't defined and
the definition doesn't define a [JSON input argument](#json-input-argument), DSC doesn't send any
input to the resource when invoking the `whatIf` operation.

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
doesn't. In most cases, this value should be set the same as the `implementsPretest` property in
the definition for the [set method][03] in the resource manifest.

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
[_exist][07] property in the `set` operation. The default value is `false`. In most cases, this
value should be set the same as the `handlesExist` property in the definition for the
[set method][03] in the resource manifest.

Set this property to `true` when the resource meets the following implementation requirements:

- The resource's [instance schema][08] defines the `_exist` property as a valid instance property.
- The resource's `set` command handles creating, updating, and deleting an instance based on the
  current state of the instance and the value of the `_exist` property in the desired state.

When this property is set to `true`, the resource indicates that it has the `setHandlesExist`
[capability][09]. When processing resources with the `setHandlesExist` capability in a
configuration, DSC calls the `set` operation for the resource when an instance defines `_exist` as
`false`. Without this capability, a resource must define the [delete][10] operation to support
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

- `state` - Indicates that the resource returns only the instance's expected final state after the
  set operation as a JSON blob.
- `stateAndDiff` - Indicates that the resource returns the instance's expected final state and an
  array of property names that the resource would modify.

When this property isn't defined, DSC doesn't expect the resource to return any output. Instead,
DSC invokes the `get` operation for the resource after the command concludes and compares the
result to the state of the instance before the operation. For more information, see
[DSC resource what-if operation stdout schema reference][11].

```yaml
Type:        string
Required:    false
ValidValues: [state, stateAndDiff]
```

### requireSecurityContext

The `requireSecurityContext` property defines the security context the resource requires for the
`whatIf` operation. Before invoking the command, DSC compares the current security context to this
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

The `whatIfReturns` property defines how DSC should process the output for this method in what-if
mode. Because DSC always invokes the `whatIf` command in what-if mode, this property overrides the
[return](#return) property whenever it's defined. The value must be one of the same strings as the
`return` property:

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
[01]: ../../../cli/config/set.md
[02]: ../../../cli/config/set.md#-w---what-if
[03]: ./set.md
[04]: ./set.md#what-if-argument
[05]: adapter.md
[06]: ../stdout/list.md#path
[07]: ../properties/exist.md
[08]: ./root.md#schema-1
[09]: ../../definitions/resourceCapabilities.md
[10]: ./delete.md
[11]: ../stdout/whatIf.md
