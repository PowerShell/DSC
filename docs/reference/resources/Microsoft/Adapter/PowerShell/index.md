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

## Adapted resource instance definition syntax

### Implicitly required adapter syntax

```yaml
- name: <instance name>
  type: <module name>/<resource name>
  properties: # adapted resource properties
    <property name>: <property value>
```

### Explicitly required adapter syntax

```yaml
- name: <instance name>
  type: <module name>/<resource name>
  properties: # adapted resource properties
    <property name>: <property value>
  directives:
    requireAdapter: Microsoft.Adapter/WindowsPowerShell
```

## Description

The `Microsoft.Adapter/PowerShell` adapter resource enables you to use PowerShell Desired State
Configuration (PSDSC) resources in DSC. The adapter discovers and invokes PSDSC resources
implemented as PowerShell classes compatible with PowerShell.

The adapter manages the PSDSC resources in PowerShell (`pwsh`), not Windows PowerShell (`powershell.exe`). To use
MOF-based PSDSC resources or PSDSC resources that require Windows PowerShell, use the
[Microsoft.Adapter/WindowsPowerShell][01] adapter.

This adapter doesn't use the **PSDesiredStateConfiguration** module. You don't need to install the
**PSDesiredStateConfiguration** module to use PSDSC resources in DSC through this adapter.

> [!NOTE]
> This adapter replaces the deprecated [Microsoft.DSC/PowerShell][02] adapter.
>
> In earlier versions of DSC, adapted resources were nested inside a parent adapter resource using
> the `properties.resources` array. Starting in DSC 3.2, each adapted resource is listed directly
> in the configuration document's `resources` array.
>
> You can use the [`requireAdapter` directive][03] to explicitly indicate that the instance should
> use this adapter. When you don't specify the `requireAdapter` directive, DSC invokes the adapted
> resource through the first adapter that indicates it can invoke the resource.

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
$adapterScript = dsc resource list Microsoft.Adapter/PowerShell |
    ConvertFrom-Json |
    Select-Object -ExpandProperty directory |
    Join-Path -ChildPath 'psDscAdapter\powershell.resource.ps1'

& $adapterScript -Operation ClearCache
```

## Requirements

- This adapter is available on Linux, macOS, and Windows systems.
- Using this adapter requires a supported version of PowerShell.

  DSC invokes the adapter as a
  PowerShell script. For more information about installing PowerShell, see
  [Install PowerShell on Windows, Linux, and macOS][04].
- This adapter only supports PSDSC resources implemented as PowerShell classes.

  To use PSDSC resources in DSC that aren't defined as PowerShell classes,
  use the [`Microsoft.Adapter/WindowsPowerShell`][01] adapter.

## Capabilities

The resource adapter has the following capabilities:

- `get` - Retrieve the actual state of an adapted DSC resource instance.
- `set` - Enforce the desired state for an adapted DSC resource instance.
- `test` - Determine whether an adapted DSC resource instance is in the desired state.
- `export` - Discover and enumerate adapted DSC resource instances available on the system.
- `list` - List available Windows PowerShell DSC resources that can be used as adapted DSC
  resources.

## Examples

- [Invoke a resource with the PowerShell adapter][05]
- [Configure a machine with the PowerShell adapter][06]

## Adapted resource instances

Define adapted resource instances directly in the configuration document's `resources` array.

To explicitly indicate that DSC should use this adapter for the resource instance, define the
`requireAdapter` directive as `Microsoft.Adapter/PowerShell`. When you don't specify the
`requireAdapter` directive, DSC invokes the adapted resource through the first adapter that
indicates it can invoke the resource.

Adapted resource instances are defined identically to non-adapted resource instances in a
configuration document with the following exceptions:

1. The fully qualified type name (`type` field) for the adapted instance is defined by the adapter.
   This adapter uses the following syntax for determining the fully qualified type name of a PSDSC
   resource:

   ```Syntax
   <module name>/<resource name>
   ```

   For example, if a PowerShell module named **TailspinToys** has a class-based PSDSC resource named
   `TSToy`, the fully qualified type name for that resource is `TailspinToys/TSToy`.

   For more information about type names in DSC, see
   [DSC Resource fully qualified type name schema reference][07].

1. The `properties` field for the instance is validated at runtime when the adapter tries to invoke
   the adapted PSDSC resource instance. This adapter doesn't support static linting for adapted
   instance properties in a configuration document.

   Each property name must be a configurable property of the PSDSC resource. The property name
   isn't case sensitive. The value for each property must be valid for that property. If you
   specify an invalid property name or value, the adapter raises an error when it tries to invoke
   the resource.

## Exit codes

The resource uses the following exit codes to report success and errors:

- `0` - Success
- `1` - Error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because the underlying DSC resource method didn't succeed.
When the adapter returns this exit code, it also emits an error message with details about the
failure.

## See also

- [Microsoft.Adapter/WindowsPowerShell][01]
- [Microsoft.DSC/PowerShell][02] (deprecated)

<!-- Link references -->
[01]: ../WindowsPowerShell/index.md
[02]: ../../DSC/PowerShell/index.md
[03]: ../../../../schemas/config/resource.md#requireAdapter
[04]: /powershell/scripting/install/installing-powershell
[05]: examples/invoke-a-resource.md
[06]: examples/configure-a-machine.md
[07]: ../../../../concepts/type-names.md
