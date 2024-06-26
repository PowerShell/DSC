---
description: JSON schema reference for the data returned by the 'dsc resource list' command.
ms.date:     06/24/2024
ms.topic:    reference
title:       dsc resource list result schema reference
---

# dsc resource list result schema reference

## Synopsis

The result output from the `dsc resource list` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/outputs/resource/list.json
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

Defines the operations and behaviors the resource is implemented to support. This property is an
array of capabilities. Resources always have the `Get` capability, but the other capabilities are
optional and depend on the resource.

The following list describes the available capabilities for a resource:

- <a id="capability-get" ></a> `Get` - The resource supports retrieving the current state of an
  instance. All DSC Resources must have this capability. A resource has this capability when it
  defines the mandatory [get][07] property in its resource manifest.
- <a id="capability-set" ></a> `Set` - The resource supports enforcing the desired state of an
  instance. A resource has this capability when it defines the [set][08] property in its resource
  manifest. Resources without this capability can't be used with the [dsc resource set][09] or
  [dsc config set][10] commands unless they're in a Microsoft.DSC/Assertion group as a nested
  instance.
- <a id="capability-sethandlesexist" ></a> `SetHandlesExist` - The resource supports the
  [_exist property][11] directly. A resource has this capability when it defines the
  [handlesExist][12] property as `true` in the definition of the [set][08] command property in its
  resource manifest.

  When a resource has this capability, the `_exist` property is part of the resource's instance
  schema and the resource handles deleting instances of the resource in its `set` command.

  When a resource doesn't have this capability, when DSC finds an instance of the resource with
  `_exist` set to `false`, it handles calling the [delete][13] operation for the resource.

  If the resource doesn't have this capability or the `Delete` capability, DSC raises an error when
  an instance defines `_exist` as `false`.
- <a id="capability-whatif" ></a> `WhatIf` - The resource supports returning explicit information
  about how it will modify state when a user calls [dsc config set][10] with the [--what-if][14]
  option. A resource has this capability when it defines the [What-if method][15] in its resource
  manifest.

  When a resource has this capability, DSC calls the defined command with its arguments when a
  user executes the `dsc config set` command with the `--what-if` option.

  When a resource doesn't have this capability, DSC synthesizes how the resource will change and
  instance by converting the `Test` result for the instance into a `Set` result. The synthetic
  operation can't indicate potential issues or changes that can't be determined by comparing the
  result of the `Test` operation against the resource's desired state. For example, the credentials
  used to test a resource might be valid for that operation, but not have permissions to actually
  modify the system state. Only a resource with this capability can fully report whether and how
  the resource will change system state.
- <a id="capability-test" ></a> `Test` - The resource supports validating the desired state of an
  instance against the current state of the instance. A resource has this capability when it
  defines the [test][16] property in its resource manifest.

  If a resource doesn't have the `Test` capability, DSC uses a synthetic test for instances of the
  resource. The synthetic test compares each property for the desired state of an instance against
  the actual state. The synthetic test uses strict, case-sensitive equivalence. If the desired
  state for a property and the actual state aren't the same, DSC marks the property as out of the
  desired state.
- <a id="capability-delete" ></a> `Delete` - The resource supports removing an instance. A resource
  has this capability when it defines the [delete][13] property in its resource manifest. This
  capability isn't mutually exclusive with the `SetHandlesExist` property. A resource can handle
  the `_exist` property in set operations and be called directly with [dsc resource delete][17] to
  remove an instance.
- <a id="capability-export" ></a> `Export` - The resource supports enumerating every instance of
  the resource. A resource has this capability when it defines the [export][18] property in its
  resource manifest. Only resources with this capability are usable with the
  [dsc resource export][19] and [dsc config export][20] commands.
- <a id="capability-resolve" ></a> `Resolve` - The resource supports resolving nested resource
  instances from an external source. A resource has this capability when it defines the
  [resolve][21] property in its resource manifest. This functionality is primarily used by
  [importer resources][22].

```yaml
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsType:         string
ItemsValidValues: [Get, Set, SetHandlesExist, Test, Delete, Export]
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
[Command-based DSC Resource manifest schema reference][23].

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
[07]: ../../resource/manifest/get.md
[08]: ../../resource/manifest/set.md
[09]: ../../../cli/resource/set.md
[10]: ../../../cli/config/set.md
[11]: ../../resource/properties/exist.md
[12]: ../../resource/manifest/set.md#handlesexist
[13]: ../../resource/manifest/delete.md
[14]: ../../../cli/config/set.md#-w---what-if
[15]: ../../resource/manifest/whatif.md
[16]: ../../resource/manifest/test.md
[17]: ../../../cli/resource/delete.md
[18]: ../../resource/manifest/export.md
[19]: ../../../cli/resource/export.md
[20]: ../../../cli/config/export.md
[21]: ../../resource/manifest/resolve.md
[22]: ../../definitions/resourceKind.md#importer-resources
[23]: ../../resource/manifest/root.md
