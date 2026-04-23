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
```

## Instance definition syntax

> [!NOTE]
> The `directives.requireAdapter` syntax is available in DSC 3.2 and later.

```yaml
resources:
- name: <instance name>
  type: <module name>/<resource name>
  directives:
    requireAdapter: Microsoft.Adapter/WindowsPowerShell
  properties: # adapted resource properties
```

## Description

The `Microsoft.Adapter/WindowsPowerShell` adapter resource enables you to use PowerShell Desired
State Configuration (PSDSC) resources in DSC. The resource can:

- Execute script-based DSC resources
- Run class-based DSC resource methods
- Execute binary DSC resources

The adapter manages the PSDSC resources in Windows PowerShell (powershell.exe), not PowerShell
(pwsh). To use class-based PSDSC resources in PowerShell, use the
[Microsoft.Adapter/PowerShell](../powershell/index.md) adapter.

This adapter uses the **PSDesiredStateConfiguration** module v1.1. This module is built-in when
you install Windows and is located in
`%SystemRoot%\System32\WindowsPowerShell\v1.0\Modules`.

> [!NOTE]
> This adapter replaces the deprecated
> [Microsoft.Windows/WindowsPowerShell](../../Windows/WindowsPowerShell/index.md) adapter. In
> earlier versions of DSC, adapted resources were nested inside a parent adapter resource using
> the `properties.resources` array. Starting in DSC 3.2, each adapted resource is listed directly
> in the configuration document's `resources` array with `directives.requireAdapter` set to
> `Microsoft.Adapter/WindowsPowerShell`.

### PowerShell resource adapter cache

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
- The process context must have appropriate permissions for the DSC resources to be executed.
- PowerShell modules exposing DSC resources should be installed in one of the following locations:
  - `%PROGRAMFILES%\WindowsPowerShell\Modules`
  - `%SystemRoot%\System32\WindowsPowerShell\v1.0\Modules`

## Capabilities

The resource adapter has the following capabilities:

- `get` - Retrieve the actual state of a DSC resource instance.
- `set` - Enforce the desired state for a DSC resource instance.
- `test` - Determine whether a DSC resource instance is in the desired state.
- `export` - Discover and enumerate DSC resource instances available on the system.
- `list` - List available Windows PowerShell DSC resources that can be used with `dsc.exe`.

> [!NOTE]
> The `export` capability is only available with class-based DSC resources. Script-based and
> binary DSC resources don't support the export operation.

## Examples

- [Manage a Windows service with the WindowsPowerShell adapter][02]

## Adapted resource instances

Adapted resource instances are listed directly in the configuration document's `resources` array.
Set `directives.requireAdapter` to `Microsoft.Adapter/WindowsPowerShell` to indicate that DSC
should use this adapter to invoke the instance. This feature requires DSC 3.2 or later.

Every adapted instance must be an object that defines the [name](#adapted-instance-name),
[type](#adapted-instance-type), and [properties](#adapted-instance-properties) for the instance.

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
resource:

```Syntax
<module name>/<resource name>
```

For example, if a PowerShell module named **TailspinToys** has a script-based PSDSC resource named
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

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because the underlying DSC resource method or
`Invoke-DscResource` call did not succeed. When the resource returns this exit code, it also emits
an error message with details about the failure.

## See also

- [Microsoft.Adapter/PowerShell](../powershell/index.md)
- [Microsoft.Windows/WindowsPowerShell](../../Windows/WindowsPowerShell/index.md) (deprecated)

<!-- Link references -->
[01]: ../../../../concepts/type-names.md
[02]: examples/manage-a-windows-service.md
