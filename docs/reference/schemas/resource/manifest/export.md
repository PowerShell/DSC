---
description: JSON schema reference for the 'export' property in a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource manifest export property schema reference
---

# DSC Resource manifest export property schema reference

## Synopsis

Defines how to retrieve the current state of every instance for a DSC Resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.export.json
Type:          object
```

## Description

A command-based DSC Resource that can enumerate every instance of itself with a single command
should define the `export` property in its manifest. This property defines how DSC can get the
current state for every resource instance. When this property is defined, the resource has the
`export` capability and users can:

- Specify an instance of the resource in the input configuration for the [dsc config export][01]
  command to generate an usable configuration document.
- Specify the resource with the [dsc resource export][02] command to generate a configuration
  document that defines every instance of the resource.
- Specify the resource with the [dsc resource get][03] command and the [--all][04] option to return
  the current state for every instance of the resource.

When the DSC calls the command defined by this property, the resource must return the current state
of every instance as [JSON lines][05]. Each JSON Line should be an object representing the instance
and validate against the [defined resource instance schema][06]. For more information about the
expected output, including the output for exporter resources, see
[DSC resource export operation stdout schema reference][07].

Users can provide input for the `export` operation to filter the exported instances, like with the
[--input][08] option for the `dsc resource export` command or the properties of an instance in a
configuration document. Before sending the input to the command, DSC validates it:

1. If the manifest defines [supportsFiltering](#supportsfiltering) as `false`, DSC raises an
   error.
1. If the manifest defines the [schema](#schema) property, DSC validates the input against that
   schema.
1. Otherwise, if the manifest defines the [validate][09] property, DSC calls the `validate` command
   to validate the input.
1. Otherwise, DSC validates the input against the resource's instance schema.

DSC sends data to this command in three ways:

1. When `input` is `stdin`, DSC sends the data as a string representing the data as a compressed
   JSON object without spaces or newlines between the object properties.
1. When `input` is `env`, DSC sends the data as environment variables. It creates an environment
   variable for each property in the input data object, using the name and value of the property.
1. When the `args` array includes a JSON input argument definition, DSC sends the data as a string
   representing the data as a compressed JSON object to the specified argument.

If you don't define the `input` property and don't define a JSON input argument, DSC can't pass the
input JSON to the resource. This is only appropriate for resources that don't support filtering
the exported instances. You can only define one JSON input argument for a command.

## Required properties

The `export` definition must include these properties:

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

DSC passes the arguments to the command in the order they're defined. For every argument kind
except string arguments, DSC passes the argument name followed by its value as two separate
arguments.

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

This argument kind is only useful for [resource adapters][10]. This argument kind was added in DSC
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
input JSON to the resource.

```yaml
Type:               object
RequiredProperties: [jsonInputArg]
```

#### Resource path argument

Defines an argument for the command that accepts the path to the resource being invoked. For
resource adapters, this is the value of the [path][11] property that the adapter returned for the
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

### input

The `input` property defines how to pass input to the resource. If this property isn't defined and
the definition doesn't define a [JSON input argument](#json-input-argument), DSC doesn't send any
input to the resource when invoking the `export` operation.

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
`export` operation. Before invoking the command, DSC compares the current security context to this
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

### schema

The `schema` property defines the JSON schema that DSC uses to validate the input for the `export`
operation. Define this property when the properties the resource accepts for filtering exported
instances differ from the resource's instance schema. This property uses the same shape as the
manifest [schema][12] property. The value must be an object that defines one of the following
properties:

- `command` - When you specify the `command` property, DSC calls the defined command to get the
  JSON schema for the export input. The `command` object must define the `executable` property and
  may define the `args` property, which accepts the same argument kinds as the manifest
  [schema.command.args][13] property.
- `embedded` - When you specify the `embedded` property, DSC uses the defined value as the JSON
  schema for the export input.

When the manifest doesn't define this property, DSC validates the export input with the
[validate][09] command if the manifest defines it, or against the resource's instance schema
otherwise. A manifest can't define both this property and `supportsFiltering`.

This property was added in DSC version 3.3.0.

```yaml
Type:               object
Required:           false
RequiredProperties: [command | embedded]
```

### supportsFiltering

The `supportsFiltering` property indicates whether the resource accepts input for the `export`
operation to filter the exported instances. When this property is `false`, DSC raises an error if
a user provides input for the `export` operation. When this property is `true` or isn't defined,
DSC validates the input and sends it to the command.

A manifest can't define both this property and `schema`.

This property was added in DSC version 3.3.0.

```yaml
Type:     boolean
Required: false
Default:  true
```

<!-- Link reference definitions -->
[01]: ../../../cli/config/export.md
[02]: ../../../cli/resource/export.md
[03]: ../../../cli/resource/get.md
[04]: ../../../cli/resource/get.md#-a---all
[05]: https://jsonlines.org/
[06]: schema/property.md
[07]: ../stdout/export.md
[08]: ../../../cli/resource/export.md#-i---input
[09]: validate.md
[10]: adapter.md
[11]: ../stdout/list.md#path
[12]: root.md#schema-1
[13]: schema/property.md#args
