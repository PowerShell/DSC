---
description: JSON schema reference for the 'schema' property in a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource manifest schema property schema reference
---

# DSC Resource manifest schema property reference

## Synopsis

Defines how to retrieve the JSON Schema that validates a DSC Resource instance.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.schema.json
Type:          object
```

## Description

Every command-based DSC Resource that doesn't define the [validate][01] property should define the
`schema` property in its manifest. This property defines how DSC can get the JSON schema it needs
to validate instances of the resource. DSC validates the input for an operation against the schema
before invoking the resource and, for resources with the `resource` [kind][02], validates the
output the resource returns. When a manifest defines neither `schema` nor `validate`, DSC raises an
error when it needs to validate an instance of the resource.

The JSON schema can be defined dynamically with the [command](#command) property or statically with
the [embedded](#embedded) property.

For development purposes, it can be more convenient to use the `command` property and avoid needing
to adjust both the code and the schema.

Microsoft recommends using the `embedded` property when publishing a resource publicly. When the
manifest declares the schema with the `command` property, DSC calls the command at the beginning of
any operation using the resource, possibly impacting performance. The schema is also unavailable to
integrating tools when the resource isn't installed locally. When the schema is embedded in the
manifest, DSC and integrating tools only need the manifest itself.

## Examples

### Example 1 - Get JSON schema with a command

This example is from the `Microsoft.Windows/Registry` DSC Resource.

```json
"schema": {
  "command": {
    "executable": "registry",
    "args": ["schema"]
  }
}
```

With the `command` property defined, DSC gets the JSON schema to validate instances of this
resource with the following command:

```sh
registry schema
```

### Example 2 - Embedded JSON schema

This example is from the `Microsoft/OSInfo` DSC Resource. It defines an embedded JSON schema that
DSC uses to validate an instance of the resource.

```json
"schema": {
  "embedded": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "OSInfo",
    "type": "object",
    "required": [],
    "properties": {
      "$id": { "type": "string" },
      "architecture": { "type": ["string","null"] },
      "bitness": { "$ref": "#/definitions/Bitness" },
      "codename": { "type": ["string","null"] },
      "edition": { "type": ["string","null"] },
      "family": { "$ref": "#/definitions/Family" },
      "version": { "type": "string" }
    },
    "additionalProperties": false,
    "definitions": {
      "Bitness": { "type": "string", "enum": ["32","64","unknown"] },
      "Family": { "type": "string", "enum": ["Linux","macOS","Windows"] }
    }
  }
}
```

### Example 3 - Get JSON schema for an adapted resource

This example defines a schema command for a resource adapter that operates on a single adapted
resource at a time. DSC passes the type name and version of the adapted resource to the command.

```json
"schema": {
  "command": {
    "executable": "my_adapter",
    "args": [
      "schema",
      { "resourceTypeArg": "--type" },
      { "resourceVersionArg": "--version" }
    ]
  }
}
```

When DSC needs the schema for the adapted resource `Contoso/Example` version `1.0.0`, it runs:

```sh
my_adapter schema --type Contoso/Example --version 1.0.0
```

## Required properties

The `schema` definition must include exactly one of these properties:

- [command](#command)
- [embedded](#embedded)

## Properties

### command

The `command` property defines how DSC must call the resource to get the JSON schema that validates
its instances. The value of this property must be an object and define the `executable` property.

When publishing a manifest with the `command` property, Microsoft recommends publishing the JSON
schema to a publicly available URI that matches the `$id` property of the instance schema. This
enables authoring tools and other integrating applications to validate instances without running
the command locally.

For more information about the expected output, see
[DSC resource schema command stdout schema reference][03].

```yaml
Type:               object
RequiredProperties: [executable]
```

#### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the command. A
file extension is only required when the command isn't recognizable by the operating system as an
executable.

```yaml
Type:     string
Required: true
```

#### args

The `args` property defines the list of arguments to pass to the command. DSC passes the arguments
to the command in the order they're specified. Each item in the array must be a string or an object
that defines one of the following argument kinds:

- [String arguments](#string-arguments) - A static argument, like `schema`.
- [Resource type argument](#resource-type-argument) - The fully qualified type name of the resource
  being invoked.
- [Resource version argument](#resource-version-argument) - The version of the resource being
  invoked.

For every argument kind except string arguments, DSC passes the argument name followed by its value
as two separate arguments.

```yaml
Type:      array
Required:  false
Default:   []
ItemsType: [string, object]
```

##### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `schema` or `--format`.

```yaml
Type: string
```

##### Resource type argument

Defines an argument for the command that accepts the fully qualified type name of the resource
being invoked. For resource adapters, this is the type name of the adapted resource. Use this
argument kind to implement an adapter that returns the schema for a single adapted resource.

- `resourceTypeArg` (required) - The argument to pass the type name to for the command, like
  `--type`.

```yaml
Type:               object
RequiredProperties: [resourceTypeArg]
```

##### Resource version argument

Defines an argument for the command that accepts the version of the resource being invoked. For
resource adapters, this is the version of the adapted resource. This argument kind was added in
DSC version 3.3.0.

- `resourceVersionArg` (required) - The argument to pass the version to for the command, like
  `--version`.

```yaml
Type:               object
RequiredProperties: [resourceVersionArg]
```

### embedded

The `embedded` property defines the full JSON schema for DSC to validate instances of the DSC
Resource. The value for this property must be a valid JSON schema that defines the `$schema`,
`type`, and `properties` keywords. For more information, see
[DSC Resource manifest embedded schema reference][04].

```yaml
Type:                 object
MinimumPropertyCount: 1
```

<!-- Link reference definitions -->
[01]: ../validate.md
[02]: ../root.md#kind
[03]: ../../stdout/schema.md
[04]: embedded.md
