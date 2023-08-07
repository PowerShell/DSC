---
description: JSON schema reference for the 'schema' property in a DSC Resource manifest
ms.date:     08/04/2023
ms.topic:    reference
title:       DSC Resource manifest schema property schema reference
---

# DSC Resource manifest schema property reference

## Synopsis

Defines how to retrieve the JSON Schema that validates a DSC Resource instance.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.schema.json
Type           : object
```

## Description

Every command-based DSC Resource must define the `schema` property in its manifest. This property
defines how DSC can get the JSON schema it needs to validate instances of the resource.

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

## Required Properties

The `schema` definition must include exactly one of these properties:

- [command](#command)
- [embedded](#embedded)

## Properties

### command

The `command` property defines how DSC must call the resource to get the JSON schema that validates
its instances. The value of this property must be an object and define the `executable` property.

When publishing a manifest with the `command` property, Microsoft recommends publishing the JSON
schema to a publicly available URI and setting the `url` property to that URI. This enables
authoring tools and other integrating applications to validate instances without running the
command locally.

```yaml
Type:     object
Required Properties:
  - executable
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

The `args` property defines an array of strings to pass as arguments to the command. DSC passes the
arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```

### embedded

The `embedded` property defines the full JSON schema for DSC to validate instances of the DSC
Resource. The value for this property must be a valid JSON schema that defines the `$schema`,
`type`, and `properties` keywords.

```yaml
Type: object
Minimum Property Count: 1
```

### url

The `url` property defines the URL to the resource's published JSON schema. It's used by
integrating tools for resources that define the `command` property instead of the `embedded`
property.

<!-- Can it resolve to a JSON schema published as YAML, or JSON only? -->

```yaml
Type:   string
Format: uri
```
