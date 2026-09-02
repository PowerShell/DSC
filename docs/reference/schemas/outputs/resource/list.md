---
description: JSON schema reference for the data returned by the 'dsc resource list' command.
ms.date:     09/01/2026
ms.topic:    reference
title:       dsc resource list result schema reference
---

# dsc resource list result schema reference

## Synopsis

The result output from the `dsc resource list` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/outputs/resource/list.json
Type:          object
```

## Description

The output from the `dsc resource list` command includes a representation of discovered DSC
resources as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
resource. DSC uses the same schema to represent the adapted resources a resource adapter returns
for the **List** operation. For more information, see
[DSC resource list operation stdout schema reference][02].

The output object for a resource always includes every property described in this document. When a
property isn't defined for a resource, DSC emits the property with the value `null`.

## Required properties

Each resource in the output always defines these properties:

- [type](#type)
- [kind](#kind)
- [version](#version)
- [capabilities](#capabilities)
- [path](#path)
- [directory](#directory)

## Properties

### type

Identifies the fully qualified type name of the resource. It's used to specify the resource in
configuration documents and as the value of the `--resource` flag when using the `dsc resource *`
commands. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][03].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### kind

Identifies how DSC handles the resource. DSC supports several kinds of resources: `resource`,
`adapter`, `group`, `importer`, and `exporter`. This value is either defined in the
[resource manifest][04] or inferred by DSC. For more information about resource kinds, see
[DSC Resource kind schema reference][05].

```yaml
Type:        string
Required:    true
ValidValues: [adapter, exporter, group, importer, resource]
```

### version

Represents the current version of the resource as a valid semantic version (SemVer) string. The
version applies to the resource, not the software it manages. DSC also accepts a deprecated
date-based version in the format `YYYY-MM-DD` with an optional prerelease suffix.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### capabilities

Defines the list of capabilities for the resource. DSC resources always have at least one
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
                    setWhatIf,
                    test,
                    delete,
                    deleteWhatIf,
                    export,
                    resolve
                  ]
```

### deprecationMessage

Indicates that the resource is deprecated. When a resource manifest defines the
`deprecationMessage` property, DSC emits the message as a warning whenever a user invokes an
operation for the resource and includes the message in this output. For resources that aren't
deprecated, this property is `null`.

```yaml
Type:     [string, 'null']
Required: false
```

### path

Represents the path to the resource's manifest on the machine. For adapted resources, this
property identifies the path to the file that defines the resource instead.

```yaml
Type:     string
Required: true
```

### description

Defines a synopsis for the resource's purpose as a short string. If the resource doesn't have a
description, this property is `null`.

```yaml
Type:     [string, 'null']
Required: false
```

### directory

Represents the path to the folder containing the resource's manifest on the machine. For adapted
resources, this property identifies the path to the folder containing the file that defines the
resource instead.

```yaml
Type:     string
Required: true
```

### implementedAs

Indicates how the resource is implemented. For command-based resources, this value is `null`.
Resource adapters set this property to a string that distinguishes between the implementations of
the resources they support. For example, the PowerShell adapters report `ClassBased`,
`ScriptBased`, or `Binary` for adapted resources.

```yaml
Type:     [string, 'null']
Required: false
```

### author

Indicates the name of the person or organization that developed and maintains the resource. If
this property is `null`, the author is unknown.

```yaml
Type:     [string, 'null']
Required: false
```

### properties

Defines the property names for adapted resources. For non-adapted resources, this property is
`null`.

```yaml
Type:         [array, 'null']
Required:     false
ItemsType:    string
ItemsPattern: ^\w+$
```

### requireAdapter

Defines the fully qualified type name of the resource adapter that this resource is made available
through. This value is only defined for adapted resources. For non-adapted resources, this value
is always `null`.

```yaml
Type:     [string, 'null']
Required: false
```

### schema

Defines the JSON schema that validates instances of an adapted resource. When an adapter defines
this property for an adapted resource, DSC uses the schema to validate instances of the adapted
resource instead of invoking the adapter's [schema][08] command. For non-adapted resources, this
property is `null`.

```yaml
Type:     [object, 'null']
Required: false
```

### targetResource

Reserved for DSC. When DSC invokes an adapter for an adapted resource, DSC sets this property on
the adapter's representation to the adapted resource. In the output of the `dsc resource list`
command, this property is always `null`.

```yaml
Type:     [object, 'null']
Required: false
```

### manifest

Represents the values defined in the resource's manifest. This value is `null` for resources that
aren't command-based, like adapted resources. For more information on the value for this property,
see [Command-based DSC Resource manifest schema reference][09].

```yaml
Type:     [object, 'null']
Required: false
```

### adaptedContent

Defines the inline content of an adapted resource as a JSON object. When an adapted resource is
defined with inline content instead of a path, DSC sets this property to that content. For other
resources, this property is `null`. This property was added in DSC version 3.3.0.

```yaml
Type:     [object, 'null']
Required: false
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../resource/stdout/list.md
[03]: ../../definitions/resourceType.md
[04]: ../../resource/manifest/root.md#kind
[05]: ../../definitions/resourceKind.md
[06]: ../../../../concepts/resources/capabilities.md
[07]: ../../../../concepts/resources/operations.md
[08]: ../../resource/manifest/schema/property.md
[09]: ../../resource/manifest/root.md
