---
description: Microsoft.Adapter/WindowsPowerShell resource adapter reference documentation
ms.date:     03/23/2026
ms.topic:    reference
title:       Microsoft.Adapter/WindowsPowerShell
---

# Microsoft.Adapter/WindowsPowerShell

## Synopsis

Adapter for resources implemented as binary, script, or PowerShell classes in Windows PowerShell.

## Metadata

```yaml
Version:            0.1.0
Kind:               adapter
Tags:               [windows, powershell]
Executable:         powershell
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

The `Microsoft.Adapter/WindowsPowerShell` adapter resource enables you to use PowerShell Desired
State Configuration (PSDSC) resources in DSC. The adapter discovers and invokes PSDSC resources
compatible with Windows PowerShell and PSDSC version `1.1`.

The adapter manages the PSDSC resources in Windows PowerShell (powershell.exe), not PowerShell
(pwsh). To use class-based PSDSC resources in PowerShell, use the
[Microsoft.Adapter/PowerShell][01] adapter.

This adapter uses the **PSDesiredStateConfiguration** module v1.1. This module is built-in when
you install Windows and is located in
`%SystemRoot%\System32\WindowsPowerShell\v1.0\Modules`.

> [!NOTE]
> This adapter replaces the deprecated [Microsoft.Windows/WindowsPowerShell][02] adapter.
>
> In earlier versions of DSC, adapted resources were nested inside a parent adapter resource using
> the `properties.resources` array. Starting in DSC 3.2, each adapted resource is listed directly
> in the configuration document's `resources` array.
>
> You can use the [`requireAdapter` directive][03] to explicitly indicate that the instance should
> use this adapter. When you don't specify the `requireAdapter` directive, DSC invokes the adapted
> resource through the first adapter that indicates it can invoke the resource.

### Windows PowerShell resource adapter cache

The process for discovering the Windows PowerShell resources available to the adapter can be
time-consuming. To improve performance, the adapter caches Windows PowerShell resources and modules
during discovery. If the cache doesn't exist during discovery, the adapter creates it.

The following table defines the cache path for the Windows platform.

| Platform | Path                                            |
| :------: | :---------------------------------------------- |
| Windows  | `%LOCALAPPDATA%\dsc\WindowsPSAdapterCache.json` |

The adapter versions the cache. The current version is `1`. If the version of the cache on a
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
$adapterScript = dsc resource list Microsoft.Adapter/WindowsPowerShell |
    ConvertFrom-Json |
    Select-Object -ExpandProperty directory |
    Join-Path -ChildPath 'psDscAdapter\powershell.resource.ps1'

& $adapterScript -Operation ClearCache
```

## Requirements

- This adapter is only available on Windows.
- The process security context must be elevated.

  For PSDSC 1.1, invoking DSC resources requires the process to run as Administrator. Attempting to
  invoke the resources in a non-elevated context fails.
- Windows PowerShell Desired State Configuration (PSDSC) depends on WinRM. If WinRM isn't setup on
  the machine, invoking PSDSC resources through the adapter will raise an error.

  You can use the [`Enable-PSRemoting` cmdlet][04] in an elevated Windows PowerShell session to
  enable WinRM.
- PowerShell modules exposing PSDSC resources for use with this adapter must be installed in one of
  the following locations:

  - `%PROGRAMFILES%\WindowsPowerShell\Modules`
  - `%SystemRoot%\System32\WindowsPowerShell\v1.0\Modules`

  PSDSC 1.1 only finds PSDSC resources when the module containing them is installed in the machine
  scope. Modules containing PSDSC resources in the user scope or another non-default location
  aren't recognized by PSDSC 1.1 and can't be invoked through this adapter.

## Capabilities

The resource adapter has the following capabilities:

- `get` - Retrieve the actual state of an adapted DSC resource instance.
- `set` - Enforce the desired state for an adapted DSC resource instance.
- `test` - Determine whether an adapted DSC resource instance is in the desired state.
- `export` - Discover and enumerate adapted DSC resource instances available on the system.
- `list` - List available Windows PowerShell DSC resources that can be used as adapted DSC
  resources.

> [!NOTE]
> The `export` capability is only available for class-based PSDSC resources. Script-based and
> binary PSDSC resources don't support the export operation.

## Examples

- [Manage a Windows service with the WindowsPowerShell adapter][05]

## Adapted resource instances

Define adapted resource instances directly in the configuration document's `resources` array.

To explicitly indicate that DSC should use this adapter for the resource instance, define the
`requireAdapter` directive as `Microsoft.Adapter/WindowsPowerShell`. When you don't specify the
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

   For example, if a PowerShell module named **TailspinToys** has a script-based PSDSC resource named
   `TSToy`, the fully qualified type name for that resource is `TailspinToys/TSToy`.

   For more information about type names in DSC, see
   [DSC Resource fully qualified type name schema reference][06].

1. The `properties` field for the instance is validated at runtime when the adapter tries to invoke
   the adapted PSDSC resource instance. This adapter doesn't support static linting for adapted
   instance properties in a configuration document.

   Each property name must be a configurable property of the PSDSC resource. The property name
   isn't case sensitive. The value for each property must be valid for that property. If you
   specify an invalid property name or value, the adapter raises an error when it tries to invoke
   the resource.

## Exit codes

The adapter resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because the underlying DSC resource method or
`Invoke-DscResource` call didn't succeed. When the adapter returns this exit code, it also emits
an error message with details about the failure.

## See also

- [Microsoft.Adapter/PowerShell][01]
- [Microsoft.Windows/WindowsPowerShell][02] (deprecated)

<!-- Link references -->
[01]: ../PowerShell/index.md
[02]: ../../Windows/WindowsPowerShell/index.md
[03]: ../../../../schemas/config/resource.md#requireAdapter
[04]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/enable-psremoting?view=powershell-5.1&preserveView=true
[05]: examples/manage-a-windows-service.md
[06]: ../../../../schemas/definitions/resourceType.md
