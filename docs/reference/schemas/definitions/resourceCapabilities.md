---
description: JSON schema reference for resource capabilities
ms.date:     07/03/2025
ms.topic:    reference
title:       DSC Resource capabilities schema reference
---

# DSC Resource capabilities schema reference

## Synopsis

Defines the operations you can invoke for a resource and how the resource behaves when invoked.

## Metadata

```yaml
SchemaDialect:     https://json-schema.org/draft/2020-12/schema
SchemaID:          https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/definitions/resourceKind.json
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsType:         string
ItemsValidValues: [
                    get,
                    set,
                    setHandlesExist,
                    whatIf,
                    test,
                    delete,
                    export,
                    resolve
                  ]
```

## Description

DSC resources always have at least one capability. Resource capabilities define the operations you
can invoke for a resource and how the resource behaves when invoked.

DSC resources may have the following capabilities:

- `get` - The resource can retrieve the current state of an instance.
- `set` - The resource can enforce the desired state for an instance.
- `setHandlesExist` - The resource handles deleting an instance during a **Set** operation.
- `whatIf` - The resource can report how it would change state for an instance during a **Set** operation.
- `test` - The resource implements the **Test** operation and doesn't rely on synthetic testing.
- `delete` - The resource can remove an instance.
- `export` - The resource can enumerate every instance.
- `resolve` - The resource can resolve nested instances from an external source.

For more information about resource capabilities, see [DSC resource capabilities][01]. For more
information about the operations you can invoke for a resource, see [DSC resource operations][02].

[01]: ../../../concepts/resources/capabilities.md
[02]: ../../../concepts/resources/operations.md
