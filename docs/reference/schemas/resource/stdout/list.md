---
description: JSON schema reference for the expected stdout from the list resource operation
ms.date:     09/01/2026
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
stdout for the **List** operation to adhere to this schema. DSC invokes the **List** operation for
a resource adapter with the command defined by the [adapter.list][01] property in the adapter's
manifest.

This schema is the same schema DSC uses to represent every discovered resource, including the
output of the [dsc resource list][02] command. DSC parses each JSON Line independently. When a line
doesn't adhere to this schema, DSC logs a warning and skips the line. When a line doesn't define
the [requireAdapter](#requireadapter) property, DSC logs a warning and skips the adapted resource.

DSC includes the following adapter resources:

- [Microsoft.DSC/PowerShell][03] run PowerShell and enables you to use PowerShell DSC (PSDSC)
  resources implemented as PowerShell classes in DSC.
- [Microsoft.Windows/WindowsPowerShell][04] runs Windows PowerShell and enables you to use any
  available PSDSC resources in DSC. This adapter is only available when you install DSC on
  Windows.
- [Microsoft.Windows/WMI][05] enables you to use WMI classes as resources in DSC. This adapter is
  only available when you install DSC on Windows.

## Required properties

The output for the **List** operation must include these properties:

- [type](#type)
- [kind](#kind)
- [version](#version)
- [capabilities](#capabilities)
- [path](#path)
- [directory](#directory)
- [requireAdapter](#requireadapter)

## Properties

### type

The `type` property represents the fully qualified type name of the adapted resource. It's used to
specify the resource in configuration documents and as the value of the `--resource` flag when
using the `dsc resource *` commands. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][06].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### kind

The `kind` property defines how DSC should handle the adapted resource. DSC supports several kinds
of resources: `resource`, `adapter`, `group`, `importer`, and `exporter`. Adapters typically
report every adapted resource with the `resource` kind.

For more information, see [DSC resource kinds][07].

```yaml
Type:        string
Required:    true
ValidValues: [adapter, exporter, group, importer, resource]
```

### version

The `version` property represents the current version of the adapted resource as a valid semantic
version (SemVer) string. The version applies to the adapted resource, not the software it manages.
DSC also accepts a deprecated date-based version in the format `YYYY-MM-DD` with an optional
prerelease suffix.

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
- `setWhatIf` - The resource can report how it would change state for an instance during a **Set**
  operation without modifying the system. This capability was added in DSC version 3.3.0. Through
  DSC version 3.2, this capability was named `whatIf`.
- `test` - The resource implements the **Test** operation and doesn't rely on synthetic testing.
- `delete` - The resource can remove an instance.
- `deleteWhatIf` - The resource can report how it would remove an instance during a **Delete**
  operation without modifying the system. This capability was added in DSC version 3.3.0.
- `export` - The resource can enumerate every instance.
- `resolve` - The resource can resolve nested instances from an external source.

For more information about resource capabilities, see [DSC resource capabilities][08]. For more
information about the operations you can invoke for a resource, see [DSC resource operations][09].

```yaml
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsType:         string
ItemsValidValues: [
                    get,
                    set,
                    setHandlesExist,
                    setWhatIf,
                    test,
                    delete,
                    deleteWhatIf,
                    export,
                    resolve
                  ]
```

### deprecationMessage

Indicates that the adapted resource is deprecated. When defined, DSC emits the message as a warning
whenever a user invokes an operation for the adapted resource and includes the message in the
output of the `dsc resource list` command.

```yaml
Type:     [string, 'null']
Required: false
```

### path

Indicates the path to the adapted resource on the file system, like the path to the module that
implements a PSDSC resource. DSC passes this value to the adapter when the adapter's operation
definitions include a [resource path argument][10].

```yaml
Type:     string
Required: true
```

### description

Defines a synopsis for the adapted resource's purpose as a short string.

```yaml
Type:     [string, 'null']
Required: false
```

### directory

Indicates the path to the folder containing the adapted resource on the file system.

```yaml
Type:     string
Required: true
```

### implementedAs

Indicates how the adapted resource is implemented. When the value is `null` or the property is
omitted, DSC treats the resource as a command-based resource. Adapters should set this property to
a string that distinguishes between the implementations of the resources they support. For
example, the PowerShell adapters report `ClassBased`, `ScriptBased`, or `Binary`.

```yaml
Type:     [string, 'null']
Required: false
```

### author

Indicates the name of the person or organization that developed and maintains the adapted resource.

```yaml
Type:     [string, 'null']
Required: false
```

### properties

Defines the adapted resource's property names.

```yaml
Type:         [array, 'null']
Required:     false
ItemsType:    string
ItemsPattern: ^\w+$
```

### requireAdapter

Defines the fully qualified type name of the adapter that the adapted resource depends on. An
adapter should always set this value to its own fully qualified resource type name. Although the
schema doesn't mark this property as required, DSC skips any adapted resource that doesn't define
it and logs a warning.

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### schema

Defines the JSON schema that validates instances of the adapted resource. When an adapter defines
this property for an adapted resource, DSC uses the schema to validate instances of the adapted
resource instead of invoking the adapter's [schema][11] command for the adapted resource.

```yaml
Type:     [object, 'null']
Required: false
```

### targetResource

Reserved for DSC. When DSC invokes an adapter for an adapted resource, DSC sets this property on
the adapter's representation to the adapted resource. Adapters shouldn't define this property in
the output for the **List** operation.

```yaml
Type:     [object, 'null']
Required: false
```

### manifest

Represents the values defined in a command-based resource's manifest. Adapted resources don't have
a manifest of their own, so adapters shouldn't define this property in the output for the **List**
operation.

```yaml
Type:     [object, 'null']
Required: false
```

### adaptedContent

Defines the inline content of the adapted resource as a JSON object. When an adapted resource
manifest defines the resource with the `content` property instead of the `path` property, DSC sets
this property to that content. DSC passes this value to the adapter when the adapter's operation
definitions include an [adapted content argument][12]. This property was added in DSC version
3.3.0.

```yaml
Type:     [object, 'null']
Required: false
```

<!-- Reference link definitions -->
[01]: ../manifest/adapter.md#list
[02]: ../../outputs/resource/list.md
[03]: ../../../resources/Microsoft/DSC/PowerShell/index.md
[04]: ../../../resources/Microsoft/Windows/WindowsPowerShell/index.md
[05]: ../../../resources/Microsoft/Windows/WMI/index.md
[06]: ../../definitions/resourceType.md
[07]: ../../../../concepts/resources/kinds.md
[08]: ../../../../concepts/resources/capabilities.md
[09]: ../../../../concepts/resources/operations.md
[10]: ../manifest/get.md#resource-path-argument
[11]: ../manifest/schema/property.md
[12]: ../manifest/get.md#adapted-content-argument
