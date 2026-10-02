---
description: JSON schema reference for the 'import' operation output in a DSC extension
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC extension import operation stdout schema reference
---

# DSC extension import operation stdout schema reference

## Synopsis

Represents a configuration document defined in an alternate format or language and imported by an extension.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/extension/stdout/discover.json
Type:          object
$ref:          https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/document.json
```

## Description

Represents a DSC configuration document that the extension imported from an alternate format. DSC
expects every JSON Line emitted to stdout for the **Import** operation to adhere to this schema.

Currently, the definition for this schema consists of a single `$ref` keyword, which points to the
[DSC Configuration document schema][01]. DSC expects the extension to emit a valid configuration
document as a single JSON Line.

[01]: ../../config/document.md
