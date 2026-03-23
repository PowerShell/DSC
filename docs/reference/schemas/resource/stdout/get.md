---
description: JSON schema reference for the expected stdout from the get resource operation
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource get operation stdout schema reference
---

# DSC resource get operation stdout schema reference

## Synopsis

Represents the actual state of a resource instance in DSC. DSC expects the JSON Line emitted to
stdout for the **Get** operation to adhere to this schema.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/get.json
Type:          object
```

## Description

Represents the actual state of a resource instance in DSC. DSC expects the JSON Line emitted to
stdout for the **Get** operation to adhere to this schema.

The output must be a JSON object. The object must be a valid representation of an instance of the
resource.

Command resources define their instance schema with the [schema.command][01] or
[schema.embedded][02] fields in their resource manifest. If a command resource returns JSON that is
invalid against the resource instance schema, DSC raises an error.

Adapted resource instances are validated by their adapter when the adapter invokes them.

<!-- Reference link definitions -->
[01]: ../manifest/schema/property.md
[02]: ../manifest/schema/embedded.md
