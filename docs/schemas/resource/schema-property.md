# DSC Resource manifest schema property reference

Every command-based DSC Resource must define the `schema` property in its DSC Resource manifest.
This property defines how DSC can get the JSON schema it needs to validate instances of the DSC
Resource.

This document describes the schema for the property.

## Metadata

| Metadata Key | Metadata Value                                                         |
|:------------:|:-----------------------------------------------------------------------|
|  `$schema`   | `https://json-schema.org/draft/2020-12/schema`                         |
|    `$id`     | `https://aka.ms/dsc/schemas/resource/manifest.yaml#/properties/schema` |
|    `type`    | `object`                                                               |

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

With the `command` property defined, DSC gets the JSON schema to validate instances of this DSC
Resource with the following command:

```sh
registry schema
```

### Example 2 - Embedded JSON schema

This example is from the `Microsoft/OSInfo` DSC Resource. It defines an embedded JSON schema that
DSC uses to validate an instance of the DSC Resource.

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

The `provider` definition must include exactly one of these properties:

- [command](#command)
- [embedded](#embedded)

## Properties

### command

The `command` property defines how DSC must call the DSC Resource to get the JSON schema that
validates its instances. The value of this property must be an object and define the `executable`
property.

When you specify the `command` property, you should also publish the JSON schema to a publicly
available URI so authoring tools and other integrating applications can validate instances without
running the command locally. When you do, use the `url` property to indicate the public URI of the
schema for integrating tools.

```yaml
Type:     object
Required Properties:
  - executable
```

#### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the
application. A file extension is only required when the executable isn't recognizable by the
operating system as an executable.

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
Resource. The value for this property must be a valid JSON schema.

```yaml
Type: object
Minimum Property Count: 1
```

### url

The `url` property defines the URL to the DSC Resource's JSON schema. It's used by integrating
tools for DSC Resources that define the `command` property instead of the `embedded` property.

<!-- Can it resolve to a JSON schema published as YAML, or JSON only? -->

```yaml
Type:   string
Format: uri
```
