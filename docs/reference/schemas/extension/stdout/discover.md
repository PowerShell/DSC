---
description: JSON schema reference for the 'discover' property in a DSC extension manifest
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC extension discover operation stdout schema reference
---

# DSC extension discover operation stdout schema reference

## Synopsis

Represents the path to a manifest not discoverable in the `PATH` or `DSC_RESOURCE_PATH` environment
variables.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/extension/stdout/discover.json
Type:          object
```

## Description

Represents the actual state of a resource instance in DSC path to a discovered DSC resource or
extension manifest on the system. DSC expects every JSON Line emitted to stdout for the
**Discover** operation to adhere to this schema.

The output must be a JSON object. The object must define the full path to the discovered manifest.
If an extension returns JSON that is invalid against this schema, DSC raises an error.

If the extension doesn't discover any manifests, it must return nothing to stdout and exit with
code `0`. An empty output with a zero exit code indicates no resources were found. A non-zero exit
code indicates an error, even if stdout is empty.

## Required Properties

The output for the `discover` operation must include these properties:

- [manifestPath](#manifestpath)

## Properties

### manifestPath

The value for this property must be the absolute path to a manifest file on the system. The
manifest can be for a DSC resource or extension. If the returned path doesn't exist, DSC raises an
error.

Each discovered manifest must be emitted as a separate JSON Line to stdout. If no manifests are
discovered, the extension must not emit any output to stdout.

```yaml
Type:     string
Required: true
```

## Exit codes

The extension must return one of the following exit codes:

- `0` - Success. The extension completed discovery. If no manifests were found, stdout is empty.
- Non-zero - Error. DSC treats any non-zero exit code as a failure and surfaces the extension's
  stderr output as an error message.
