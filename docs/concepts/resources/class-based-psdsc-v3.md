---
description: >-
  Explains how class-based PSDSC resources can participate in DSC v3 through the
  PowerShell adapter.
ms.date:     06/15/2026
ms.topic:    conceptual
title:       Author class-based PSDSC resources for DSC v3
---

# Author class-based PSDSC resources for DSC v3

This article collects the background and examples for RFC 0001, which defines the contract for
PowerShell **class-based** PSDSC resources that participate in DSC v3 through the PowerShell
adapter.

## Related DSC concepts

This article assumes you're already familiar with the shared DSC concepts and data models described
in the following docs:

- [DSC Resources](./overview.md)
- [DSC resource operations](./operations.md)
- [DSC resource capabilities](./capabilities.md)
- [DSC resource properties](./properties.md)
- [DSC resource test operation stdout schema reference](../../reference/schemas/resource/stdout/test.md)
- [DSC resource set operation stdout schema reference](../../reference/schemas/resource/stdout/set.md)
- [DSC Resource _inDesiredState property schema](../../reference/schemas/resource/properties/inDesiredState.md)

## Relationship between PSDSC and DSC operations

Class-based PSDSC resources always expose the instance methods `Get()`, `Test()`, and `Set()`.
Through the adapter, the same class can also expose optional static methods that line up with DSC v3
operations.

| Operation | PSDSC class contract | DSC v3 adapter contract |
| --- | --- | --- |
| Get | `[<ResourceClass>] Get()` | `static [<ResourceClass>] Get([<ResourceClass>]$instance)` |
| Test | `[bool] Test()` | `static [System.Tuple[bool, <ResourceClass>]] Test(...)` or `static [System.Tuple[bool, <ResourceClass>, String[]]] Test(...)` |
| Set | `[void] Set()` | `static [void] Set(...)`, `static [<ResourceClass>] Set(...)`, or `static [System.Tuple[<ResourceClass>, String[]]] Set(...)` |
| Delete | not available | `static [void] Delete([<ResourceClass>]$instance)` |
| Export | not available | `static [<ResourceClass>[]] Export()` and/or `static [<ResourceClass>[]] Export([<ResourceClass>]$filteringInstance)` |
| Schema | MOF or class properties | `static [string] InstanceJsonSchema()` |

The biggest semantic differences are:

- DSC **Test** returns actual state and can also return differing properties.
- DSC **Set** returns richer change information and may support explicit `whatIf`.
- DSC adds **Delete** and **Export**, which have no PSDSC instance-method equivalent.

## The PSDSC `Reasons` pattern

Many PSDSC resources adopted a `Reasons` property to surface why an instance wasn't in the desired
state.

This pattern was especially important for Azure Machine Configuration and typically required:

1. A resource-specific reason type with only `Code` and `Phrase` string properties.
1. A non-configurable `Reasons` property on the resource that returned an array of those reason
   objects.

For example:

```powershell
class MyModuleReasons {
    [DscProperty()] [string] $Code
    [DscProperty()] [string] $Phrase
}

[DscResource()]
class Package {
    [DscProperty(NotConfigurable)] [MyModuleReasons[]] $Reasons
}
```

The pattern existed because PSDSC `Test()` only returned a boolean and PSDSC `Set()` returned no
data. As a result, authors often had to push compliance detail into `Get()`.

In DSC v3, richer `Test` and `Set` results reduce the need for a dedicated `Reasons` property:

- `Test` can return whether the instance is in the desired state, the actual state, and optionally
  the differing properties.
- `Set` can return the final state and optionally the changed properties.
- Resources can also emit structured messages during operations.

## Authoring a class-based resource that works for PSDSC v1/v2 and DSC v3

The following example uses a hypothetical resource named `SoftwarePackage`.

- `Name` is the package name.
- `Version` is the package version.
- `_exist` indicates whether the package should exist.

The example keeps the PSDSC instance methods and adds optional static methods for DSC v3:

```powershell
[DscResource()]
class SoftwarePackage {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $Version

    [DscProperty()]
    [bool] $_exist = $true

    static [string] InstanceJsonSchema() {
        return (
            @{
                '$schema'   = 'https://json-schema.org/draft/2020-12/schema'
                type        = 'object'
                required    = @('name')
                properties  = @{
                    name    = @{ type = 'string' }
                    version = @{ type = 'string' }
                    _exist  = @{
                        '$ref' = 'https://aka.ms/dsc/schemas/v3/resource/properties/exist.json'
                    }
                }
            } | ConvertTo-Json -Depth 10 -Compress
        )
    }

    static [System.Tuple[bool, SoftwarePackage, string[]]] Test(
        [SoftwarePackage] $instance
    ) {
        return Test-SoftwarePackageResource -Instance $instance
    }

    static [System.Tuple[SoftwarePackage, string[]]] Set(
        [SoftwarePackage] $instance
    ) {
        return Set-SoftwarePackageResource -Instance $instance
    }

    static [SoftwarePackage] Get(
        [SoftwarePackage] $instance
    ) {
        return Get-SoftwarePackageResource -Instance $instance
    }

    static [void] Delete(
        [SoftwarePackage] $instance
    ) {
        Remove-SoftwarePackageResource -Instance $instance
    }

    static [SoftwarePackage[]] Export() {
        return Export-SoftwarePackageResource
    }

    static [SoftwarePackage[]] Export(
        [SoftwarePackage] $filteringInstance
    ) {
        if ($null -eq $filteringInstance) {
            throw 'Invalid operation'
        }

        return Export-SoftwarePackageResource -FilteringInstance $filteringInstance
    }

    [SoftwarePackage] Get() {
        return Get-SoftwarePackageResource -Instance $this
    }

    [bool] Test() {
        return Test-SoftwarePackageResource -Instance $this |
            Select-Object -ExpandProperty Item1
    }

    [void] Set() {
        Set-SoftwarePackageResource -Instance $this
    }
}
```

Key points in this pattern:

- The class remains a valid PSDSC resource.
- DSC-specific methods are static and optional.
- The static methods can delegate to internal helper functions.
- The adapter can translate tuple returns into the JSON Lines DSC expects.

## Using the resource in DSC v3

After the adapter exposes the resource to DSC, you can use it like any other DSC v3 resource.

For example:

```yaml
resources:
  - type: Contoso.DSC/SoftwarePackage
    name: InstallGit
    properties:
      Name: git
```

When the class implements the optional static methods, DSC can:

- Validate input with the schema returned by `InstanceJsonSchema()`.
- Call richer `Test`, `Set`, `Delete`, and `Export` operations through the adapter.
- Surface structured state and diff information instead of only PSDSC-compatible results.

## See also

- [RFC 0001 - Class-based PSDSC resource contract for DSC v3](../../../rfc/draft/rfc0001.md)
- [DSC resource operations](./operations.md)
- [DSC resource capabilities](./capabilities.md)
- [DSC resource properties](./properties.md)
