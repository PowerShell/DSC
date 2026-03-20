---
description: JSON schema reference for the expected stdout from the delete resource operation
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC resource delete operation stdout schema reference
---

# DSC resource delete operation stdout schema reference

## Synopsis

DSC doesn't expect the **Delete** operation for a resource to return any JSON to stdout.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/delete.json
Type:          'null'
```

## Description

DSC resources that implement the **Delete** operation shouldn't emit any data to stdout. DSC
doesn't expect any output for the **Delete** operation and ignores any data emitted to stdout when
invoking the operation.
