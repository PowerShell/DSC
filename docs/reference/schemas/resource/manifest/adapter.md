---
description: JSON schema reference for the 'adapter' property in a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource manifest adapter property schema reference
---

# DSC Resource manifest adapter property schema reference

## Synopsis

Defines a DSC Resource as a DSC Resource Adapter.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.adapter.json
Type:          object
```

## Description

DSC Resource Adapters must define the `adapter` property in their manifest. This property
identifies the resource as an adapter and defines how DSC can call the adapter to get the resources
the adapter supports and how to pass resource instances to the adapter. When a manifest defines
this property and doesn't define the [kind][01] property, DSC infers the resource kind as
`adapter`.

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
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      "./psDscAdapter/powershell.resource.ps1 List"
    ]
  },
  "inputKind": "full"
}
```

The manifest sets `inputKind` to `full`, indicating that the adapter expects a JSON blob
representing the full and unprocessed configuration from `stdin`.

It defines `list.executable` as `pwsh`. The arguments defined in `list.args` ensure that DSC runs
PowerShell:

- Without the logo banner
- In non-interactive mode
- Without loading any profile scripts
- With the execution policy set to `Bypass`
- To invoke the `powershell.resource.ps1` script in the `psDscAdapter` folder next to the manifest
  and pass the `List` argument.

With this definition, DSC calls the `list` method for this adapter by running:

```sh
pwsh -NoLogo -NonInteractive -NoProfile -ExecutionPolicy Bypass \
  -Command "./psDscAdapter/powershell.resource.ps1 List"
```

### Example 2 - Microsoft.Adapter/PowerShell

This example is from the `Microsoft.Adapter/PowerShell` DSC Resource Adapter.

```json
"adapter": {
  "list": {
    "executable": "pwsh",
    "args": [
      "-NoLogo",
      "-NonInteractive",
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      "./psDscAdapter/powershell.resource.ps1",
      "List",
      "-ResourceType",
      "Single"
    ]
  },
  "inputKind": "single"
}
```

The manifest sets `inputKind` to `single`, indicating that DSC should invoke the adapter for one
adapted resource instance at a time. The adapter's `get`, `set`, `test`, and `export` definitions
use the [resource type argument][02] and [resource path argument][03] to identify which adapted
resource to invoke.

## Required properties

The `adapter` definition must include these properties:

- [inputKind](#inputkind)

## Properties

### inputKind

The `inputKind` property defines how the adapter expects to receive resource configurations. The
value must be one of the following options:

- `full` - Indicates that the adapter expects a JSON blob containing the full and unprocessed
  configuration as a single JSON blob over `stdin`. DSC adds a `metadata` property with the
  `Microsoft.DSC.context` value set to `configuration` to the input so the adapter can distinguish
  a full configuration from a single resource instance.
- `sequence` - Indicates that the adapter expects each resource's configuration as a
  [JSON Line][04] over `stdin`.
- `single` - Indicates that the adapter expects a single adapted resource instance. DSC invokes
  the adapter's operation commands directly for each adapted resource instance and passes the
  instance properties as the input for the command. Use the [resource type argument][02],
  [resource path argument][03], [resource version argument][05], and
  [adapted content argument][06] in the operation definitions to tell the adapter which adapted
  resource to invoke.

Prior to DSC version 3.2.0, this property was named `config`. DSC still accepts the `config` name
for backward compatibility, but new manifests should use `inputKind`.

```yaml
Type:        string
Required:    true
ValidValues: [full, sequence, single]
```

### list

The `list` property defines how to call the adapter to list the resources it supports. The value
of this property must be an object and define the `executable` sub-property. For more information
about the expected output, see [DSC resource list operation stdout schema reference][07].

When this property isn't defined, DSC can't discover the adapter's resources by invoking the
adapter. Users can still define adapted resources for the adapter with adapted resource manifests
that specify the adapter with their `requireAdapter` property.

```yaml
Type:               object
Required:           false
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
the arguments to the command in the order they're specified. Unlike the `args` property for the
operation methods, this array only accepts strings.

```yaml
Type:      array
Required:  false
Default:   []
ItemsType: string
```

<!-- Link reference definitions -->
[01]: root.md#kind
[02]: get.md#resource-type-argument
[03]: get.md#resource-path-argument
[04]: https://jsonlines.org/
[05]: get.md#resource-version-argument
[06]: get.md#adapted-content-argument
[07]: ../stdout/list.md
