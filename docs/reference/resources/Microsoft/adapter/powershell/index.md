---
description: Microsoft.Adapter/PowerShell resource adapter reference documentation
ms.date:     03/23/2026
ms.topic:    reference
title:       Microsoft.Adapter/PowerShell
---

# Microsoft.Adapter/PowerShell

## Synopsis

Adapter for resources implemented as PowerShell DSC classes.

## Metadata

```yaml
Version:            0.1.0
Kind:               adapter
Tags:               [linux, windows, macos, pwsh, powershell]
Executable:         pwsh
MinimumDSCVersion:  3.2.0
```

## Instance definition syntax

> [!NOTE]
> The `directives.requireAdapter` syntax is available in DSC 3.2 and later.

```yaml
resources:
- name: <instance name>
  type: <module name>/<class name>
  directives:
    requireAdapter: Microsoft.Adapter/PowerShell
  properties: # adapted resource properties
```

## Description

The `Microsoft.Adapter/PowerShell` adapter resource enables you to use PowerShell Desired State
Configuration (PSDSC) resources in DSC. The adapter discovers and invokes PSDSC resources
implemented as PowerShell classes.

The adapter manages the PSDSC resources in PowerShell (pwsh), not Windows PowerShell. To use
MOF-based PSDSC resources or PSDSC resources that require Windows PowerShell, use the
[Microsoft.Adapter/WindowsPowerShell](../windowspowershell/index.md) adapter.

This adapter doesn't use the **PSDesiredStateConfiguration** module. You don't need to install the
**PSDesiredStateConfiguration** module to use PSDSC resources in DSC through this adapter.

> [!NOTE]
> This adapter replaces the deprecated [Microsoft.DSC/PowerShell](../../DSC/PowerShell/index.md)
> adapter. In earlier versions of DSC, adapted resources were nested inside a parent adapter
> resource using the `properties.resources` array. Starting in DSC 3.2, each adapted resource is
> listed directly in the configuration document's `resources` array with `directives.requireAdapter`
> set to `Microsoft.Adapter/PowerShell`.

### PowerShell resource adapter cache

The process for discovering the PowerShell resources available to the adapter can be
time-consuming. To improve performance, the adapter caches PowerShell resources and modules during
discovery. If the cache doesn't exist during discovery, the adapter creates it.

The location of the cache depends on your operating system. The following table defines the path
for each platform.

| Platform |                      Path                |
| :------: | :----------------------------------------|
|  Linux   | `$HOME/.dsc/PSAdapterCache.json`         |
|  macOS   | `$HOME/.dsc/PSAdapterCache.json`         |
| Windows  | `%LOCALAPPDATA%\dsc\PSAdapterCache.json` |

The adapter versions the cache. The current version is `2`. If the version of the cache on a
machine differs from the current version, the adapter refreshes the cache.

The adapter checks whether the cache is stale on each run and refreshes it if:

- The `PSModulePath` environmental variable is updated.
- Any module is added or removed from the `PSModulePath`.
- Any related file in a cached PSDSC resource module has been updated since the cache was written.
  The adapter watches the `LastWriteTime` property of module files with the following extensions:
  `.ps1`, `.psd1`, and `.psm1`.

You can directly call the adapter script to clear the cache with the **Operation** parameter value
set to `ClearCache`:

```powershell
$adapterScript = dsc resource list -a Microsoft.Adapter/PowerShell |
    ConvertFrom-Json |
    Select-Object -ExpandProperty directory |
    Join-Path -ChildPath 'psDscAdapter' -AdditionalChildPath 'powershell.resource.ps1'

& $adapterScript -Operation ClearCache
```

## Requirements

- Using this adapter requires a supported version of PowerShell. DSC invokes the adapter as a
  PowerShell script. For more information about installing PowerShell, see
  [Install PowerShell on Windows, Linux, and macOS](/powershell/scripting/install/installing-powershell).

## Examples

- [Invoke a resource with the PowerShell adapter][02]
- [Configure a machine with the PowerShell adapter][03]

## Adapted resource instances

Adapted resource instances are listed directly in the configuration document's `resources` array.
Set `directives.requireAdapter` to `Microsoft.Adapter/PowerShell` to indicate that DSC should use
this adapter to invoke the instance. This feature requires DSC 3.2 or later.

Every adapted instance must be an object that defines the [name](#adapted-instance-name),
[type](#adapted-instance-type), and [properties](#adapted-instance-properties) for the
instance.

### Adapted instance name

The `name` property of the adapted resource instance defines the short, human-readable name for the
instance. The adapted instance name must be a non-empty string containing only letters, numbers,
and spaces. This property must be unique within the configuration document's `resources` array.

```yaml
PropertyName:  name
Type:          string
Required:      true
MinimumLength: 1
Pattern:       ^[a-zA-Z0-9 ]+$
```

### Adapted instance type

The `type` property identifies the adapted instance's PSDSC resource. The value for this property
must be the valid fully qualified type name for the resource.

This adapter uses the following syntax for determining the fully qualified type name of a PSDSC
resource implemented as a PowerShell class:

```Syntax
<module name>/<class name>
```

For example, if a PowerShell module named **TailspinToys** has a class-based PSDSC resource named
`TSToy`, the fully qualified type name for that resource is `TailspinToys/TSToy`.

For more information about type names in DSC, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### Adapted instance properties

The `properties` of an adapted resource instance define its desired state. The value of this
property must be an object. The specified properties are validated at runtime when the adapter
tries to invoke the adapted PSDSC resource instance. This adapter doesn't support static linting
for adapted instance properties in a configuration document.

Each name for each property must be a configurable property of the PSDSC resource. The property
name isn't case sensitive. The value for each property must be valid for that property. If you
specify an invalid property name or value, the adapter raises an error when it tries to invoke the
resource.

```yaml
Type:     object
Required: true
```

## Exit codes

The resource uses the following exit codes to report success and errors:

- `0` - Success
- `1` - Error

## See also

- [Microsoft.Adapter/WindowsPowerShell](../windowspowershell/index.md)
- [Microsoft.DSC/PowerShell](../../DSC/PowerShell/index.md) (deprecated)

<!-- Link references -->
[01]: ../../../../concepts/type-names.md
[02]: examples/invoke-a-resource.md
[03]: examples/configure-a-machine.md
