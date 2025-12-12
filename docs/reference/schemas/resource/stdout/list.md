---
description: JSON schema reference for the expected stdout from the list resource operation
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource list operation stdout schema reference
---

# DSC resource list operation stdout schema reference

## Synopsis

Defines the representation of an adapted resource in DSC. DSC expects every JSON Line emitted to
stdout for the **List** operation to adhere to this schema.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/list.json
Type:          object
```

## Description

Defines the representation of an adapted resource in DSC. DSC expects every JSON Line emitted to
stdout for the **List** operation to adhere to this schema.

DSC includes the following adapter resources:

- [Microsoft.DSC/PowerShell][01] run PowerShell and enables you to use PowerShell DSC (PSDSC)
resources implemented as PowerShell classes in DSC.
- [Microsoft.Windows/WindowsPowerShell][02] runs Windows PowerShell and enables you to use any
available PSDSC resources in DSC. This adapter is only available when you install DSC on
Windows.
- [Microsoft.Windows/WMI][03] enables you to use WMI classes as resources in DSC. This adapter is
only available when you install DSC on Windows.

## Required Properties

The output for the `discover` operation must include these properties:

- [type](#type)
- [kind](#kind)
- [version](#version)
- [capabilities](#capabilities)
- [path](#path)
- [directory](#directory)
- [implementedAs](#implementedas)
- [properties](#properties-1)
- [requireAdapter](#requireadapter)

## Properties

### type

The `type` property represents the fully qualified type name of the resource. It's used to specify
the resource in configuration documents and as the value of the `--resource` flag when using the
`dsc resource *` commands. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][04].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### kind

The `kind` property defines how DSC should handle the adapted resource. DSC supports several kinds
of resources: `resource`, `group`, `adapter`, `importer`, and `exporter`.

For more information, see [DSC resource kinds][05].

```yaml
Type:        string
Required:    false
ValidValues: [resource, adapter, group, importer, exporter]
```

### version

The `version` property represents the current version of the adapted resource as a valid semantic
version (semver) string. The version applies to the adapted resource, not the software it manages.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### capabilities

Defines the list of capabilities for the adapted resource. DSC resources always have at least one
capability. Resource capabilities define the operations you can invoke for a resource and how the
resource behaves when invoked.

DSC resources may have the following capabilities:

- `get` - The resource can retrieve the current state of an instance.
- `set` - The resource can enforce the desired state for an instance.
- `setHandlesExist` - The resource handles deleting an instance during a **Set** operation.
- `whatIf` - The resource can report how it would change state for an instance during a **Set** operation.
- `test` - The resource implements the **Test** operation and doesn't rely on synthetic testing.
- `delete` - The resource can remove an instance.
- `export` - The resource can enumerate every instance.
- `resolve` - The resource can resolve nested instances from an external source.

For more information about resource capabilities, see [DSC resource capabilities][06]. For more
information about the operations you can invoke for a resource, see [DSC resource operations][07].

```yaml
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

### path

Indicates the path to the adapted resource on the file system.

```yaml
Type:     string
Required: true
```

### directory

Indicates the path to the folder containing the adapted resource on the file system.

```yaml
Type:     string
Required: true
```

### implementedAs

Indicates that the adapted resource uses a custom implementation. The name can be used to
distinguish between different implementations for the adapted resources.

```yaml
Type: string
Required: true
```

### author

Indicates the name of the person or organization that developed and maintains the adapted Resource.

```yaml
Type:     [string, 'null']
Required:  false
Pattern:   ^\w+( \w+)*
```

### properties

Defines the adapted resource's property names.

```yaml
Type:         array
Required:     false
ItemsType:    string
ItemsPattern: ^\w+$
```

### requireAdapter

Defines the fully qualified type name of the adapter that the adapted resource depends on. An
adapter should always set this value to its own fully qualified resource type name.

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

<!-- Reference link definitions -->
[01]: ../../../resources/Microsoft/DSC/PowerShell/index.md
[02]: ../../../resources/Microsoft/Windows/WindowsPowerShell/index.md
[03]: ../../../resources/Microsoft/Windows/WMI/index.md
[04]: ../../definitions/resourceType.md
[05]: ../../../../concepts/resources/kinds.md
[06]: ../../../../concepts/resources/capabilities.md
[07]: ../../../../concepts/resources/operations.md
