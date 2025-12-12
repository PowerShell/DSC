---
description: JSON schema reference for the expected stdout from the schema resource command
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource schema command stdout schema reference
---

# DSC resource schema command stdout schema reference

## Synopsis

Represents the JSON Schema that validates instances of the resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/schema.json
Type:          object
```

## Description

Represents the JSON Schema that validates instances of the resource. DSC expects a resource that
defines the [`schema.command`][01] field in its resource manifest to return this value for that
command.

The output must be a JSON object. The object must be a valid JSON Schema. For more information
about what DSC expects for resource instance JSON Schemas, see
[DSC Resource manifest embedded schema reference][02], which describes the expectations in full.

<!-- Reference link definitions -->
[01]: ../manifest/schema/property.md
[02]: ../manifest/schema/embedded.md
