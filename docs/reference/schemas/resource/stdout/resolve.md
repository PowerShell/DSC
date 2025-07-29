---
description: JSON schema reference for the expected stdout from the resolve resource operation
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource resolve operation stdout schema reference
---

# DSC resource resolve operation stdout schema reference

## Synopsis

Defines the representation of a resolved configuration document.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/resolve.json
Type:          object
```

## Description

Defines the representation of a resolved configuration document. DSC expects the JSON Line emitted
to stdout for the **Resolve** operation to adhere to this schema.

## Required Properties

The output object for the **Resolve** operation must include these properties:

- [configuration](#configuration)

## Properties

### configuration

Defines the resolved configuration document. If the configuration document defines any parameters,
values for those parameters may be retrieved from the `parameters` property of the **Resolve**
operation output.

For more information, see [DSC Configuration document schema reference][01]

```yaml
Type:     object
Required: true
```

### parameters

The `parameters` property defines the set of resolved parameter values for the resolved
configuration document. If the `parameters` property is omitted from the output for the **Resolve**
operation, DSC doesn't pass any parameters to the resolved configuration when invoking operations
on it.

Each property of this object represents a different resolved parameter. The property name
identifies the name of a parameter. The property value is the resolved value for the parameter.

```yaml
Type:     object
Required: false
```

<!-- Reference link definitions -->
[01]: ../../config/document.md
