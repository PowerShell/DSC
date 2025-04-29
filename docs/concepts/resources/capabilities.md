---
description: >-
  Describes the capabilities of DSC resources, how DSC discovers them, and how
  the capabilities affect resource behavior and usage.
ms.date:     03/25/2025
ms.topic:    conceptual
title:       DSC resource capabilities
---

# DSC resource capabilities

DSC resources always have at least one capability. Resource capabilities define the operations you
can invoke for a resource and how the resource behaves when invoked.

The rest of this document describes the available capabilities.

## get

A resource with the `get` capability supports retrieving the current state of an instance with the
[Get][01] operation.

A command resource has this capability when it defines the required [get][02] property in its
resource manifest.

## set

A resource with the `set` capability supports enforcing the desired state of an instance with the
[Set][03] operation. Resources without this capability can't be used with the
[dsc resource set][04] or [dsc config set][05] commands unless they're defined in a
`Microsoft.DSC/Assertion` group as a nested instance.

A command resource has this capability when it defines the [set][06] property in its resource
manifest.

## setHandlesExist

A resource with the `setHandlesExist` capability indicates that you can use the [Set][03] operation
to delete an instance. Resources with this capability must have the [_exist][07] canonical resource
property. Resources that don't have the `_exist` property never have this capability.

When a resource has the `_exist` property but not the `setHandlesExist` capability:

- If the resource has the `delete` capability, DSC invokes the [Delete][08] operation instead of
  **Set** when the desired state for an instance defines `_exist` as false.
- If the resource doesn't have the `delete` capability, DSC raises an error during a **Set**
  operation when the desired state for an instance defines _exist` as false.

A command resource has this capability when it defines the [set.handlesExist][09] property as
`true` in its resource manifest.

## whatIf

A resource with the `whatIf` capability indicates that you can use the [Set][03] operation in
what-if mode to have the resource return explicit information about how it would modify state in an
actual **Set** operation.

When a resource doesn't have this capability, DSC synthesizes how the resource would change an
instance by converting the **Test** result for the instance into a **Set** result. The
synthetic operation can't indicate potential issues or changes that can't be determined by
comparing the result of the **Test** operation against the resource's desired state. For example,
the credentials used to test a resource might be valid for that operation, but not have permissions
to actually modify the system state. Only a resource with this capability can fully report whether
and how the resource would change system state.

A resource has this capability when it defines the [whatIf][10] property in its resource manifest.

## test

A resource with the `test` capability indicates that it implements the **Test** operation directly.
Resources with this capability must have the [_inDesiredState][11] canonical resource property.
Resources that don't have the `_inDesiredState` property never have this capability.

When a resource doesn't have this capability, DSC uses a synthetic test for instances of the
resource. DSC performs the synthetic test by:

1. Invoking the **Get** operation on the resource to retrieve the actual state of the instance.
1. Synthetically testing each property for the desired state of an instance against the actual
   state returned. The synthetic test:
   
   - Uses strict, case-sensitive equivalence for strings.
   - Uses simple equivalence for numerical, boolean, and null values.
   - For arrays, item order doesn't matter. Arrays are considered equivalent if both the desired
     state and actual state arrays have the same number of items and if each item in the desired
     state is contained in the actual state array.
   - For objects, property order doesn't matter. The actual state of the resource can be a superset
     of the desired state. Objects are considered equivalent if each specified property for the
     desired state is equal to the same property for the actual state. If an actual state property
     isn't defined in the desired state, DSC ignores that property for the synthetic test.
1. If the desired state for a property and the actual state aren't the same, DSC marks the property
   as out of the desired state.
1. If any properties are out of the desired state, DSC reports the entire instance as not being in
   the desired state.

Synthetic testing can't account for all resource behaviors. For example, if a package resource
allows users to define a version range for the package, the **Get** operation returns the
actual version of the package, like `1.2.3`. If the user specified the version range `~1` (NPM
syntax indicating the package should be latest released semantic version with major version `1`),
DSC would compare the desired state `~1` against the actual state `1.2.3` and consider the package
to be in the incorrect state, even if `1.2.3` is actually the latest release matching the version
pin.

Any resource that has properties which can't use a strict case-sensitive comparison check should
have this capability.

A command resource has this capability when it defines the [test][12] operation in its resource
manifest.

## delete

A resource with the `delete` capability supports removing an instance with the [Delete][08]
operation and the [dsc resource delete][13] command.

This capability isn't mutually exclusive with the `setHandlesExist` property. A resource can handle
the `_exist` property in **Set** operations and be called directly with `dsc resource delete` to
remove an instance.

For resources with the `delete` capability and the [_exist][07] canonical resource property:

- If the resource doesn't have the [setHandlesExist](#sethandlesexist) capability, DSC invokes the
  **Delete** operation for the resource instead of **Set** when the desired state defines `_exist`
  as `false`.
- If the resource does have the `setHandlesExist` capability, DSC invokes the **Set** operation for
  the resource when the desired state defines `_exist` as `false`.

Resources with the `delete` capability that don't have the `_exist` canonical resource property
must implement their **Set** operation to handle removing instances. DSC can't infer existence
semantics without the `_exist` property.

A command resource has this capability when it defines the [delete][14] property in its resource
manifest.

## export

A resource with the `export` capability supports enumerating every instance of the resource with
the **Export** operation.

You can use resources with this capability with the following commands:

- [dsc config export][15] to return a configuration document
  representing the actual state for every instance of each resource defined in the input document.
- [dsc resource export][16] to return a configuration document
  representing the actual state for every instance of the input resource.
- `dsc resource get` with the [--all][17] option to return
  the actual state of every instance of the input resource.

A command resource has this capability when it defines the [export][18] property in its resource
manifest.

## resolve

A resource with the `resolve` capability supports resolving nested resource instances from an
external source. This capability is primarily used by [importer resources][19] to enable users to
compose configuration documents.

A command resource has this capability when it defines the [resolve][20] property in its resource
manifest.

## See also

- [DSC resource operations][21]
- [DSC resource kinds][22]
- [DSC resource properties][23]

<!-- Link reference definitions -->
[01]: operations.md#get-operation
[02]: ../../reference/schemas/resource/manifest/get.md
[03]: operations.md#set-operation
[04]: ../../reference/cli/resource/set.md
[05]: ../../reference/cli/config/set.md
[06]: ../../reference/schemas/resource/manifest/set.md
[07]: ../../reference/schemas/resource/properties/exist.md
[08]: operations.md#delete-operation
[09]: ../../reference/schemas/resource/manifest/set.md#handlesexist
[10]: ../../reference/schemas/resource/manifest/whatif.md
[11]: ../../reference/schemas/resource/properties/inDesiredState.md
[12]: ../../reference/schemas/resource/manifest/test.md
[13]: ../../reference/cli/resource/delete.md
[14]: ../../reference/schemas/resource/manifest/delete.md
[15]: ../../reference/cli/config/export.md
[16]: ../../reference/cli/resource/export.md
[17]: ../../reference/cli/resource/get.md#--all
[18]: ../../reference/schemas/resource/manifest/export.md
[19]: ../resources/kinds.md#importer-resources
[20]: ../../reference/schemas/resource/manifest/resolve.md
[21]: operations.md
[22]: kinds.md
[23]: ../../concepts/resources/properties.md
