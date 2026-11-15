---
description: JSON schema reference for a DSC plug-in manifest
ms.date:     11/13/2026
ms.topic:    reference
title:       DSC plug-in manifest schema reference
---

# DSC plug-in manifest schema reference

## Synopsis

The data file that defines a DSC command plug-in.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.4.0/plugin/manifest.json
Type:          object
```

## Description

A command plug-in adds a top-level command to DSC without being part of the DSC executable. Every
plug-in must have a manifest that defines:

- The type name and version that identify the plug-in.
- The name of the command the plug-in adds to DSC.
- The executable DSC runs for the command, and any arguments DSC passes to it before the arguments
  the user specified.

When the first argument to `dsc` isn't a built-in command, DSC runs the plug-in that defines a
command with that name. For more information about how DSC runs a plug-in command, see
[Plug-in commands][01].

DSC discovers plug-in manifests in the same directories as resource and extension manifests. The
file name of a plug-in manifest must end with one of the following suffixes:

- `.dsc.plugin.json`
- `.dsc.plugin.yaml`
- `.dsc.plugin.yml`

You can also define plug-ins in the `plugins` property of a manifest list, a file whose name ends
with `.dsc.manifests.json`, `.dsc.manifests.yaml`, or `.dsc.manifests.yml`. The value of the
property is an array of plug-in manifests.

DSC doesn't load a plug-in manifest that isn't valid for this schema. It writes a warning that names
the file when it discovers plug-ins.

The rest of this document describes the manifest's schema.

## Examples

### Example 1 - Plug-in implemented as an executable

This manifest defines the `lint` command, implemented by the `dsclint` executable. DSC finds the
executable in the `PATH` environment variable or in the same directory as the manifest.

```json
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
  "type": "Microsoft.DSC/Lint",
  "version": "3.4.0",
  "description": "Check configuration documents for problems without applying them",
  "command": {
    "name": "lint",
    "executable": "dsclint"
  }
}
```

### Example 2 - Plug-in implemented as a script

This manifest defines the `graph` command, implemented by a PowerShell script in the same directory
as the manifest. The `condition` makes the plug-in available only when `pwsh` is installed. The
`manifestRelativePath` argument passes the full path to the script, because DSC runs a plug-in in
the working directory of the user, not in the directory that contains the manifest.

```json
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
  "type": "Contoso.DSC/Graph",
  "version": "0.1.0",
  "description": "Show the dependency graph of a configuration document",
  "condition": "[not(equals(tryWhich('pwsh'), null()))]",
  "command": {
    "name": "graph",
    "executable": "pwsh",
    "args": [
      "-NoLogo",
      "-NoProfile",
      "-NonInteractive",
      "-File",
      { "manifestRelativePath": "graph.ps1" }
    ]
  }
}
```

## Required properties

The manifest must include these properties:

- [$schema](#schema)
- [type](#type)
- [version](#version)
- [command](#command)

## Properties

### $schema

The `$schema` property indicates the canonical URI of this schema that the manifest validates
against. This property is mandatory. DSC uses this value to validate the manifest against the
correct JSON schema and doesn't load a manifest with a URI it doesn't recognize.

For every version of the schema, there are three valid urls:

- `.../plugin/manifest.json`

  The URL to the canonical non-bundled schema. When it's used for validation, the validating client
  needs to retrieve this schema and every schema it references.

- `.../bundled/plugin/manifest.json`

  The URL to the bundled schema. When it's used for validation, the validating client only needs to
  retrieve this schema.

- `.../bundled/plugin/manifest.vscode.json`

  The URL to the enhanced authoring schema. This schema is much larger than the other schemas, as
  it includes additional definitions that provide contextual help and snippets that the others
  don't include.

```yaml
Type:     string
Required: true
Format:   URI
```

### type

The `type` property represents the fully qualified type name of the plug-in. It identifies the
plug-in. When DSC discovers more than one manifest with the same type, it uses the manifest with
the highest [version](#version).

The type name of a plug-in uses the same syntax as the type name of a resource. For more
information, see [DSC Resource fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### version

The `version` property must be the current version of the plug-in as a valid semantic version
(semver) string.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### condition

The `condition` property defines a [DSC expression][03] that must evaluate to a boolean. When the
expression evaluates to `false`, DSC treats the plug-in as not installed. The command isn't listed
in the help or by the `dsc plugin list` command and can't be invoked.

Use a condition for a plug-in that depends on a tool that might not be installed, like in
[Example 2](#example-2---plug-in-implemented-as-a-script).

```yaml
Type:     string
Required: false
```

### deprecationMessage

The `deprecationMessage` property defines a message that indicates the plug-in is deprecated. When
this property is defined, DSC writes the message as a warning each time the command is invoked.

```yaml
Type:     string
Required: false
```

### description

The `description` property defines a one-line description of the command. DSC displays the
description next to the command in the output of `dsc --help` and `dsc plugin list`, and in the
completion scripts that `dsc completer` generates.

```yaml
Type:     string
Required: false
```

### tags

The `tags` property defines a list of searchable terms for the plug-in.

```yaml
Type:              array
Required:          false
ItemsMustBeUnique: true
ItemsType:         string
ItemsPattern:      ^\w+$
```

### command

The `command` property defines the command the plug-in adds to DSC and how DSC runs it. For more
information, see [DSC plug-in manifest command property schema reference][04].

```yaml
Type:     object
Required: true
```

### metadata

The `metadata` property defines arbitrary data for the plug-in. DSC doesn't interpret the metadata.
It's returned with the manifest in the output of the `dsc plugin list` command.

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: ../../../cli/index.md#plug-in-commands
[02]: ../../definitions/resourceType.md
[03]: ../../config/functions/overview.md
[04]: ./command.md
