---
description: JSON schema reference for the 'adapter' property in a DSC Resource manifest
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC Resource manifest adapter property schema reference
---

# DSC Resource manifest adapter property schema reference

## Synopsis

Defines a DSC Resource as a DSC Resource Adapter.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/resource/manifest.adapter.json
Type:          object
```

## Description

DSC Resource Adapters must define the `adapter` property in their manifest. This property
identifies the resource as an adapter and defines how DSC can call the adapter to get the resources
the adapter supports and how to pass resource instances to the adapter.

## Examples

### Example 1 - Microsoft.DSC/PowerShell

This example is from the `Microsoft.DSC/PowerShell` DSC Resource Adapter.

```json
"adapter": {
  "list": {
    "executable": "pwsh",
    "args": [
      "-NoLogo",
      "-NonInteractive",
      "-NoProfile",
      "-Command",
      "./powershell.resource.ps1 List"
    ]
    },
  "config": "full"
},
```

The manifest sets `config` to `full`, indicating that the adapter expects a JSON blob representing
the full and unprocessed configuration from `stdin`.

It defines `list.executable` as `pwsh`. The arguments defined in `list.args` ensure that DSC runs
PowerShell:

- Without the logo banner
- In non-interactive mode
- Without loading any profile scripts
- To invoke the `powershell.resource.ps1` script in the same folder as the `dsc` command and
  pass the `List` argument.

With this definition, DSC calls the `list` method for this adapter by running:

```sh
pwsh -NoLogo -NonInteractive -NoProfile -Command "./powershellgroup.resource.ps1 List"
```

## Required Properties

The `adapter` definition must include these properties:

- [config](#config)
- [list](#list)

## Properties

### config

The `config` property defines how the adapter expects to receive resource configurations. The
value must be one of the following options:

- `full` - Indicates that the adapter expects a JSON blob containing the full and unprocessed
  configuration as a single JSON blob over `stdin`.
- `sequence` - Indicates that the adapter expects each resource's configuration as a
  [JSON Line][01] over `stdin`.

```yaml
Type:        string
ValidValues: [full, sequence]
```

### list

The `list` property defines how to call the adapter to list the resources it supports. The value
of this property must be an object and define the `executable` sub-property.

```yaml
Type:               object
Required:           true
RequiredProperties: [executable]
```

#### executable

The `executable` sub-property defines the name of the command to run. The value must be the name of
a command discoverable in the system's `PATH` environment variable or the full path to the command.
A file extension is only required when the command isn't recognizable by the operating system as an
executable.

```yaml
Type:     string
Required: true
```

#### args

The `args` sub-property defines an array of strings to pass as arguments to the command. DSC passes
the arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```

[01]: https://jsonlines.org/
