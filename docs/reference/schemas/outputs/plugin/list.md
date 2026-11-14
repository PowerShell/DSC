---
description: JSON schema reference for the data returned by the 'dsc plugin list' command.
ms.date:     11/13/2026
ms.topic:    reference
title:       dsc plugin list result schema reference
---

# dsc plugin list result schema reference

## Synopsis

The result output from the `dsc plugin list` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.4.0/outputs/plugin/list.json
Type:          object
```

## Description

The output from the `dsc plugin list` command includes a representation of discovered DSC command
plug-ins as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
plug-in.

DSC returns the plug-ins in the order it discovered them, including the plug-ins that are
[shadowed](#shadowed).

## Examples

### Example 1 - Plug-in that provides a command

This example shows the output for a plug-in defined in the file `/opt/dsc/dsclint.dsc.plugin.json`.

```json
{
  "type": "Microsoft.DSC/Lint",
  "version": "3.4.0",
  "command": "lint",
  "description": "Check configuration documents for problems without applying them",
  "deprecationMessage": null,
  "path": "/opt/dsc/dsclint.dsc.plugin.json",
  "directory": "/opt/dsc",
  "shadowed": false,
  "manifest": {
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
    "type": "Microsoft.DSC/Lint",
    "version": "3.4.0",
    "description": "Check configuration documents for problems without applying them",
    "command": {
      "name": "lint",
      "executable": "dsclint"
    }
  }
}
```

## Required properties

Each plug-in in the output always includes these properties:

- [type](#type)
- [version](#version)
- [command](#command)
- [description](#description)
- [deprecationMessage](#deprecationmessage)
- [path](#path)
- [directory](#directory)
- [shadowed](#shadowed)
- [manifest](#manifest)

## Properties

### type

Identifies the fully qualified type name of the plug-in. Plug-in type names use the same syntax as
resource type names. For more information, see
[DSC Resource fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### version

Represents the current version of the plug-in as a valid semantic version (SemVer) string. When DSC
discovers more than one manifest with the same type, it only returns the plug-in with the highest
version.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### command

Indicates the name of the command the plug-in adds to DSC. For example, a plug-in with the command
`lint` is invoked as `dsc lint`.

```yaml
Type:     string
Required: true
Pattern:  ^[a-z][a-z0-9]*(-[a-z0-9]+)*$
```

### description

Defines a short description of the command. The value is `null` when the manifest doesn't define a
description.

```yaml
Type:     [string, 'null']
Required: true
```

### deprecationMessage

Indicates that the plug-in is deprecated. The value is `null` when the plug-in isn't deprecated.
When the value is a string, DSC writes the message as a warning each time the command is invoked.

```yaml
Type:     [string, 'null']
Required: true
```

### path

Represents the path to the file that defines the plug-in manifest. For a plug-in defined in a
manifest list, this is the path to the manifest list.

```yaml
Type:     string
Required: true
```

### directory

Represents the path to the folder that contains the plug-in manifest. DSC resolves the
manifest-relative path arguments of the plug-in against this folder and passes it to the plug-in in
the `DSC_PLUGIN_DIR` environment variable.

```yaml
Type:     string
Required: true
```

### shadowed

Indicates whether another plug-in that DSC discovered earlier provides the same command. DSC never
runs a shadowed plug-in for its command name.

Plug-ins are shadowed when plug-ins with different types define the same command name. DSC searches
the directories for manifests in a fixed order, and the plug-in in the first directory provides the
command. When both plug-ins are in the same directory, the plug-in whose type name sorts first
provides the command.

To run a shadowed plug-in, specify its type name instead of the command name, like
`dsc OpenDSC/Lint`.

```yaml
Type:     boolean
Required: true
```

### manifest

Represents the values defined in the plug-in manifest that defines the plug-in. For more
information on the value for this property, see [DSC plug-in manifest schema reference][03].

```yaml
Type:     object
Required: true
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../definitions/resourceType.md
[03]: ../../plugin/manifest/root.md
