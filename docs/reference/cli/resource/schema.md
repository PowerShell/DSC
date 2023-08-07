---
description: Command line reference for the 'dsc resource schema' command
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc resource schema
---

# dsc resource schema

## Synopsis

Returns the JSON Schema for instances of a resource.

## Syntax

```sh
dsc resource schema [Options] --resource <RESOURCE>
```

## Description

The `schema` subcommand returns the JSON schema for a instances of a specific DSC Resource. DSC
uses these schemas to validate the input for the `get`, `set`, and `test` subcommands and when
validating the instances in a DSC Configuration document.

Integrating tools may use these schemas for validation or to enhance the configuration authoring
experience. A resource's instance schema defines the valid structure for an instance, including
which properties are mandatory and what their values should be. The instance schemas may also
include lightweight documentation for the properties with the `title` and `description` keywords.

## Examples

### Example 1 - Get the schema for a resource

This example returns the schema for the `OSInfo` command-based DSC Resource.

```sh
dsc resource schema --resource Microsoft/OSInfo
```

```yaml
$schema: http://json-schema.org/draft-07/schema#
title: OsInfo
type: object
required: []
properties:
  $id:
    type: string
  architecture:
    type:
    - string
    - 'null'
  bitness:
    $ref: '#/definitions/Bitness'
  codename:
    type:
    - string
    - 'null'
  edition:
    type:
    - string
    - 'null'
  family:
    $ref: '#/definitions/Family'
  version:
    type: string
additionalProperties: false
definitions:
  Bitness:
    type: string
    enum:
    - '32'
    - '64'
    - unknown
  Family:
    type: string
    enum:
    - Linux
    - MacOS
    - Windows
```

## Options

### -r, --resource

Specifies the fully qualified type name of the DSC Resource to retrieve the instance schema from,
like `Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type:      String
Mandatory: true
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns a JSON object representing the JSON schema for an instance of the specified
DSC Resource.
