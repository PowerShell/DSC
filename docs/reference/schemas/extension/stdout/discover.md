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

Represents the actual state of a resource instance in DSCpath to a discovered DSC resource or
extension manifest on the system. DSC expects every JSON Line emitted to stdout for the
**Discover** operation to adhere to this schema.

The output must be a JSON object. The object must define the full path to the discovered manifest.
If an extension returns JSON that is invalid against this schema, DSC raises an error.

## Required Properties

The output for the `discover` operation must include these properties:

- [manifestPath](#manifestpath)

## Properties

### manifestPath

The value for this property must be the absolute path to a manifest file on the system. The
manifest can be for a DSC resource or extension. If the returned path doesn't exist, DSC raises an
error.

```yaml
Type:     string
Required: true
```
