---
description: JSON schema reference for the 'discover' property in a DSC extension manifest
ms.date:     09/01/2026
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

A DSC extension that can enumerate DSC resources not discoverable in the `PATH` or
`DSC_RESOURCE_PATH` environment variables should define the `discover` property in its manifest.
This property defines how DSC can get the path to, or the content of, otherwise undiscoverable
manifests. When the manifest defines this property, the extension has the `discover` capability.

When DSC performs discovery for any operation, it calls the command defined by this property with
the folder containing the extension manifest as the working directory. The extension must return
the discovered manifests as [JSON Lines][01]. Each JSON Line must be an object that validates
against the [DSC extension discover operation stdout schema reference][02].

## Examples

The following example shows the `discover` property from the manifest for the
`Microsoft.PowerShell/Discover` extension. It runs a PowerShell script and passes the list of
manifest file extensions that DSC recognizes to the script's `-extensions` parameter as a quoted
string.

```yaml
discover:
  executable: pwsh
  args:
    - -NoLogo
    - -NonInteractive
    - -ExecutionPolicy
    - Bypass
    - -NoProfile
    - -Command
    - ./powershell.discover.ps1
    - extensionsArg:  -extensions
      includeQuotes: true
```

## Required properties

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

The `args` property defines the list of arguments to pass to the command. Each item in the array
can be a string representing a static argument or an
[extensions argument](#extensions-argument) object that receives the list of file extensions DSC
recognizes for manifests.

```yaml
Type:      array
Required:  false
ItemsType: [string, object(Extensions argument)]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `discover` or `--format`.

```yaml
Type: string
```

#### Extensions argument

Defines an argument that receives the list of file extensions DSC recognizes for manifests. Use
this argument so the extension can find manifests by file name without hard-coding the naming
conventions. This argument type was added in DSC version 3.3.0.

DSC passes the value of `extensionsArg` to the command, followed by a single argument containing
the comma-separated list of recognized file extensions:

- `.dsc.adaptedresource.json`, `.dsc.adaptedresource.yaml`, and `.dsc.adaptedresource.yml`
- `.dsc.extension.json`, `.dsc.extension.yaml`, and `.dsc.extension.yml`
- `.dsc.manifests.json`, `.dsc.manifests.yaml`, and `.dsc.manifests.yml`
- `.dsc.resource.json`, `.dsc.resource.yaml`, and `.dsc.resource.yml`

An extensions argument is defined as a JSON object with the following properties:

- `extensionsArg` (required) - The argument to pass before the list of file extensions, like
  `--extensions`.
- `includeQuotes` (optional) - Indicates whether DSC should wrap the list of file extensions in
  double quotes. The default is `false`.

> [!NOTE]
> In DSC 3.2.x, the `args` array for the `discover` property accepted the same JSON input argument
> object (`jsonInputArg`) as the `get.args` property in resource manifests. Starting with DSC
> 3.3.0, that form isn't valid for the `discover` property.

```yaml
Type:               object
RequiredProperties: [extensionsArg]
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../stdout/discover.md
