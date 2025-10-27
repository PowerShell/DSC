---
description: JSON schema reference for the data returned by the 'dsc resource list' command.
ms.date:     07/03/2025
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
Resources as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
resource.

## Required properties

Each resource in the output always includes these properties:

- [type](#type)
- [version](#version)
- [path](#path)
- [directory](#directory)
- [implementedAs](#implementedas)
- [author](#author)
- [properties](#properties)
- [requireAdapter](#requireadapter)
- [manifest](#manifest)

## Properties

### type

Identifies the fully qualified type name of the resource. It's used to specify the resource in
configuration documents and as the value of the `--resource` flag when using the `dsc resource *`
commands. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### kind

Identifies whether a resource is an [adapter resource][03], a [group resource][04], or neither.
This value is either defined in the [resource manifest][05] or inferred by DSC. For more
information about resource kinds, see [DSC Resource kind schema reference][06].

```yaml
Type:          string
Required:      true
ValidValues:  [Resource, Adapter, Group]
```

### version

Represents the current version of the resource as a valid semantic version (semver) string. The
version applies to the resource, not the software it manages.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### capabilities

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

For more information about resource capabilities, see [DSC resource capabilities][07]. For more
information about the operations you can invoke for a resource, see [DSC resource operations][08].

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

### description

Defines a synopsis for the resource's purpose as a short string. If the resource doesn't have a
description, this property is `null`.

```yaml
Type:     [string, 'null']
Required: true
```

### path

Represents the path to the resource's manifest on the machine. For adapted resources, this property
identifies the path to the file that defines the resource instead.

```yaml
Type:     string
Required: true
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

Indicates how the DSC Resource was implemented. For command-based resources, this value is always
`Command`.

<!--
    Resources currently return this a null except for the test resources. Not
    sure how to document this.
-->

### author

Indicates the name of the person or organization that developed and maintains the DSC Resource. If
this property is `null`, the author is unknown.

```yaml
Type:     [string, 'null']
Required: true
```

### properties

Defines the property names for adapted resources. For non-adapted resources, this property is an
empty array.

```yaml
Type:         array
Required:     true
ItemsType:    string
ItemsPattern: ^\w+$
```

### requireAdapter

Defines the fully qualified type name of the DSC Resource Adapter that this resource is made
available through. This value is only defined for adapted resources. For non-adapted resources,
this value is always `null`.

```yaml
Type:     [string, 'null']
Required: true
```

### manifest

Represents the values defined in the resource's manifest. This value is `null` for resources that
aren't command-based. For more information on the value for this property, see
[Command-based DSC Resource manifest schema reference][09].

```yaml
Type:     [object, 'null']
Required: true
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../definitions/resourceType.md
[03]: ../../definitions/resourceKind.md#adapter-resources
[04]: ../../definitions/resourceKind.md#group-resources
[05]: ../../resource/manifest/root.md#kind
[06]: ../../definitions/resourceKind.md
[07]: ../../../../concepts/resources/capabilities.md
[08]: ../../../../concepts/resources/operations.md
[09]: ../../resource/manifest/root.md
