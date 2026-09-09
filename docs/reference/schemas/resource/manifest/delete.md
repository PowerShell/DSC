---
description: JSON schema reference for the 'delete' property in a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource manifest delete property schema reference
---

# DSC Resource manifest delete property schema reference

## Synopsis

Indicates how to call the resource to delete a specific instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.delete.json
Type:          object
```

## Description

Defines how DSC must call the DSC Resource to delete an instance. Define this method for resources
as an alternative to handling the [`_exist`][01] property in a `set` operation, which can lead to
highly complex code. If the `set` method for the resource is able to handle deleting an instance
when `_exist` is `false`, set the [`handlesExist`][02] property of the set method definition to
`true` instead. When this property is defined, the resource has the `delete` capability.

If you define the delete method in a resource manifest, ensure that you also define the
[`_exist`][01] property in the [JSON schema for the resource's properties][03].

DSC sends data to the command in three ways:

1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
   JSON object without spaces or newlines between the object properties.
1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
   variable for each property in the input data object, using the name and value of the property.
1. When the `args` array includes a JSON input argument definition, DSC sends the data as a string
   representing the data as a compressed JSON object to the specified argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. You can only define one JSON input argument for a command.

You must define the `input` property, one JSON input argument in the `args` property array, or
both.

When the `args` array includes a [what-if argument](#what-if-argument), the resource has the
`deleteWhatIf` capability. DSC calls the `delete` command with the what-if argument when a user
invokes the operation in what-if mode, like with the `--what-if` option for the
[dsc resource delete][04] command, and the resource reports how it would remove the instance
without modifying the system. When the `args` array doesn't include a what-if argument, DSC
synthesizes the what-if result from the `test` operation. The `deleteWhatIf` capability was added
in DSC version 3.3.0.

## Examples

### Example 1 - Definition with a what-if argument

```json
"delete": {
  "executable": "my_app",
  "args": [
    "config",
    "delete",
    { "whatIfArg": "--what-if" }
  ],
  "input": "stdin"
}
```

It defines `executable` as `my_app`, rather than `my_app.exe`. The extension isn't required when
the operating system recognizes the command as an executable.

The manifest defines two string arguments, `config` and `delete`, and a what-if argument. The value
of the `input` property indicates that the `delete` command expects its input as a JSON blob from
`stdin`.

When a user invokes the `delete` operation normally, DSC calls the command as:

```sh
{ ... } | my_app config delete
```

When a user invokes the `delete` operation in what-if mode, DSC calls the command as:

```sh
{ ... } | my_app config delete --what-if
```

Because the manifest defines the what-if argument, the resource has the `deleteWhatIf` capability.

## Required properties

The `delete` definition must include these properties:

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

Defines the argument to pass to the command when a user invokes the `delete` operation in what-if
mode. DSC only passes the named argument when the operation runs in what-if mode. When the
operation runs normally, DSC omits the argument.

When the `args` array includes a what-if argument, the resource has the `deleteWhatIf` capability.
In what-if mode, DSC calls the `delete` command with the argument instead of synthesizing the
result from the `test` operation. The resource must not modify the system and must return a JSON
object describing the expected result. For more information about the expected output, see
[DSC resource delete operation stdout schema reference][07].

- `whatIfArg` (required) - The argument to pass in what-if mode, like `--what-if`.

```yaml
Type:               object
RequiredProperties: [whatIfArg]
```

### input

The `input` property defines how to pass input to the resource. If this property isn't defined and
the definition doesn't define a [JSON input argument](#json-input-argument), DSC doesn't send any
input to the resource when invoking the `delete` operation.

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

### requireSecurityContext

The `requireSecurityContext` property defines the security context the resource requires for the
`delete` operation. Before invoking the command, DSC compares the current security context to this
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

<!-- Link reference definitions -->
[01]: ../properties/exist.md
[02]: set.md#handlesexist
[03]: schema/property.md
[04]: ../../../cli/resource/delete.md
[05]: adapter.md
[06]: ../stdout/list.md#path
[07]: ../stdout/delete.md
