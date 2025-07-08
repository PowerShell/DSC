---
description: JSON schema reference for the 'discover' property in a DSC extension manifest
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC extension manifest discover property schema reference
---

# DSC extension manifest discover property schema reference

## Synopsis

Defines how to retrieve DSC resources not available in `PATH` or `DSC_RESOURCE_PATH`.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/extension/manifest.discover.json
Type:          object
```

## Description

A DSC extension that can enumerate DSC resource not discoverable in the `PATH` or `DSC_RESOURCE_PATH` environment variables
should define the `export` property in its manifest. This property defines how DSC can get the
path to otherwise undiscoverable manifests.

When the DSC performs discovery for any operation, it calls the command defined by this property.
The extension must return the path to discovered manifests as [JSON lines][05]. Each JSON Line
should be an object representing the instance and validate against the
[DSC extension discover operation stdout schema reference][06].

## Required Properties

The `discover` definition must include these properties:

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

The `args` property defines the list of arguments to pass to the command. The arguments can be any
number of strings. If you want to pass the JSON object representing the property bag for the
extension input to an argument, you can define a single item in the array as a
[JSON object](#json-input-argument), indicating the name of the argument with the `jsonInputArg`
string property and whether the argument is mandatory for the command with the `mandatory` boolean
property.

```yaml
Type:     array
Required: false
Default:  []
Type:     [string, object(JSON Input Argument)]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `discover` or `--format`.

```yaml
Type: string
```

#### JSON input argument

Defines an argument for the command that accepts the JSON input object as a string. DSC passes the
JSON input to the named argument when available. A JSON input argument is defined as a JSON object
with the following properties:

- `jsonInputArg` (required) - the argument to pass the JSON data to for the command, like `--input`.
- `mandatory` (optional) - Indicate whether DSC should always pass the argument to the command,
  even when there's no JSON input for the command. In that case, DSC passes an empty string to the
  JSON input argument.

You can only define one JSON input argument per arguments array.

```yaml
Type:                object
RequiredProperties: [jsonInputArg]
```

[05]: https://jsonlines.org/
[06]: ../stdout/discover.md
