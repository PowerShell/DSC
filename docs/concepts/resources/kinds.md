# DSC resource kinds

DSC supports different behaviors and expectations for different kinds of resources.

For command resources, DSC determines what kind a resource is by analyzing the resource manifest. For more information about how DSC determines resource kinds, see [DSC resource kind schema reference](../../reference/schemas/definitions/resourceKind.md).

## Typical resources

## Adapter resources

An adapter resource makes non-command resources available to DSC. They always have a `resources`
property that takes an array of nested resource instances. Adapters may provide additional control
over how the adapted resources are processed.

For example, the `Microsoft.DSC/PowerShell` adapter enables you to use PowerShell Desired State
Configuration (PSDSC) resources in DSC. PSDSC resources are published as components of PowerShell
modules. They don't define resource manifests.

## Group resources

Group resources always operate on nested DSC Resource instances. Group resources can change how the
nested instances are processed, like the `Microsoft.DSC/Assertion` group resource.

Group resources can also be used to bundle sets of resources together for processing, like the
`Microsoft.DSC/Group` resource. You can use the [dependsOn][04] property for a resource instance in
a configuration to point to a group resource instead of enumerating each resource in the list.

## Importer resources

Importer resources resolve an external source to a set of nested DSC Resource instances. The
properties of an importer resource define how to find and resolve the external source.

An importer resource must always define the [kind][03] and [resolve][05] properties in the resource
manifest.

For example, the `Microsoft.DSC/Import` importer resource resolves instances from an external
configuration document, enabling you to compose configurations from multiple files.
