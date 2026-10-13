---
description: JSON schema reference for the expected stdout from the validate resource operation
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource validate operation stdout schema reference
---

# DSC resource validate operation stdout schema reference

## Synopsis

Defines the JSON DSC expects a resource to emit to stdout for the **Validate** operation.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/validate.json
Type:          object
```

## Description

Defines the JSON DSC expects a resource to emit to stdout for the **Validate** operation.

DSC expects the resource to return a JSON object that indicates whether the instance is valid and
optionally a string indicating how the resource is invalid.

## Required properties

The output object for the **Validate** operation must include these properties:

- [valid](#valid)

## Properties

### valid

Indicates whether the instance is valid for the resource. When the value is `true`, the instance is
valid for the resource.

```yaml
Type:     boolean
Required: true
```

### reason

Describes how and why the instance is invalid for the resource. Always define this property in the
output object when `valid` is `false`. Don't define this property in the output object when `valid`
is `true`.

```yaml
Type:     string
Required: false
```

<!-- Reference link definitions -->
