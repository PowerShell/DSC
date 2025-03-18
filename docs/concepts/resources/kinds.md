---
description: >-
  Describes the different kinds of resources that DSC supports.
ms.date:     03/25/2025
ms.topic:    conceptual
title:       DSC resource kinds
---

# DSC resource kinds

DSC supports different behaviors and expectations for different kinds of resources.

For command resources, DSC determines what kind a resource is by analyzing the resource manifest.
For more information about how DSC determines resource kinds, see
[DSC resource kind schema reference][01].

## Typical resources

Typical resources manage the state of a configurable component. The properties of these resources
define the configurable settings of the component they represent. Each instance of a typical
resource represents a distinct item, like an installable software package, a service, or a file.

You can always invoke the **Get** operation for typical resources to return the actual state of
a specific instance. If the resource has the `set` capability, you can use the **Set** operation
to enforce the desired state for a specific instance.

## Adapter resources

An adapter resource makes noncommand resources available to DSC. They always have a `resources`
property that takes an array of nested resource instances. Adapters can provide extra control over
how the adapted resources are processed.

For example, the `Microsoft.DSC/PowerShell` adapter enables you to use PowerShell Desired State
Configuration (PSDSC) resources in DSC. PSDSC resources are published as components of PowerShell
modules. They don't define resource manifests.

## Group resources

Group resources always operate on nested DSC Resource instances. Group resources can change how the
nested instances are processed, like the `Microsoft.DSC/Assertion` group resource.

Group resources can also be used to bundle sets of resources together for processing, like the
`Microsoft.DSC/Group` resource. You can use the [dependsOn][02] property for a resource instance in
a configuration to point to a group resource instead of enumerating each resource in the list.

## Importer resources

Importer resources resolve an external source to a set of nested DSC Resource instances. The
properties of an importer resource define how to find and resolve the external source.

An importer resource must always define the [kind][03] and [resolve][04] properties in the resource
manifest.

For example, the `Microsoft.DSC/Import` importer resource resolves instances from an external
configuration document, enabling you to compose configurations from multiple files.

<!-- Link reference definitions -->
[01]: ../../reference/schemas/definitions/resourceKind.md
[02]: ../../reference/schemas/config/resource.md#dependson
[03]: ../../reference/schemas/resource/manifest/root.md#kind
[04]: ../../reference/schemas/resource/manifest/resolve.md
