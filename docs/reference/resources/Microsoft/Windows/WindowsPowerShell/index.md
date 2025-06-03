---
description: Microsoft.Windows/WindowsPowerShell resource adapter reference documentation
ms.date:     03/25/2025
ms.topic:    reference
title:       Microsoft.Windows/WindowsPowerShell
---

# Microsoft.Windows/WindowsPowerShell

## Synopsis

Adapter for resources implemented as binary, script or PowerShell classes.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [windows, powershell]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instanceName>
    type: Microsoft.Windows/WindowsPowerShell
    properties:
      # Required properties
      resources:
      - name: <nested instance name>
        type: <moduleName>/<resource name>
        properties: # adapted resource properties

# Or from v3.1.0-preview.2 onwards
resources:
- name: <instanceName>
  type: <moduleName>/<resource name>
  properties: # adapted resource properties
```

## Description

The `Microsoft.Windows/WindowsPowerShell` resource adapter enables you to invoke and discover PSDSC resources. The resource can:

- Execute script-based DSC resources
- Run class-based DSC resource methods
- Execute binary DSC resources

The adapter manages the PDSC resources in Windows PowerShell, not PowerShell. To use PowerShell classes in PowerShell, use the [Microsoft.DSC/PowerShell](../../dsc/powershell/index.md) adapter.

This adapter uses the **PSDesiredStateConfiguration** module v1.1. This module is built-in when you install Windows and is located in `%SystemRoot%\System32\WindowsPowerShell\v1.0\Modules`

### PowerShell resource adapter cache

The process for discovering the Windows PowerShell resources available to the adapter can be
time-consuming. To improve performance, the adapter caches Windows PowerShell resources and modules during
discovery. If the cache doesn't exist during discovery, the adapter creates it.

The location of the cache depends on your operating system. The following table defines the path
for the Windows platform.

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
$adapterScript = dsc resource list Microsoft.Windows/WindowsPowerShell |
    ConvertFrom-Json |
    Select-Object -ExpandProperty directory |
    Join-Path

& $adapterScript -Operation ClearCache
```

## Requirements

- The resource is only usable on a Windows system.
- The resource must run in a process context that has appropriate permissions for the DSC resource to be executed.
- The PowerShell modules exposing DSC resources should be installed in
    `%PROGRAMFILES%\WindowsPowerShell\Modules` or
    `%SystemRoot%\System32\WindowsPowerShell\v1.0\Modules`

## Capabilities

The resource adapter has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of a DSC resource instance.
- `set` - You can use the resource to enforce the desired state for a DSC resource instance.
- `test` - You can use the resource to determine whether a DSC resource instance is in the desired state.
- `export` - You can use the resource to discover and enumerate DSC resource instances currently installed and available on the system.
- `list` - Lists available PowerShell DSC resources on your system that can be used with `dsc.exe`.

> [!NOTE]
> The `export` capability is only available with class-based DSC resources.
> Script-based and binary DSC resources do not support the export operation.

## Examples

1. [Manage a Windows Service][01] - Shows how to manage a Windows service

## Properties

Unlike standard resources, the `Microsoft.Windows/WindowsPowerShell` resource adapter doesn't have directly exposed properties
in its schema because it acts as a bridge to PowerShell DSC resource. Instead, the adapter:

1. Dynamically discovers the property schema for each PowerShell DSC resource
2. Stores the schema properties in a cache file for improved performance in subsequent operations
3. Passes properties to the underlying PowerShell DSC resource

The adapter maintains a cache of resource schemas at:

- Windows: `%LOCALAPPDATA%\dsc\WindowsPSAdapterCache.json`

To list the schema properties for a PowerShell DSC resource, you can run the following command:

```powershell
dsc resource list --adapter Microsoft.Windows/WindowsPowerShell <moduleName>/<resourceName> |
    ConvertFrom-Json | 
    Select-Object properties
```

You can also retrieve more information by directly reading it from the cache file:

```powershell
$cache = Get-Content -Path "$env:LOCALAPPDATA\dsc\WindowsPSAdapterCache.json" |
    ConvertFrom-Json

($cache.ResourceCache | Where-Object -Property type -EQ '<moduleName>/<resourceName>').DscResourceInfo.Properties
```

When defining a configuration document, the following properties are required.

### resources

The `resources` property defines a list of adapted PSDSC resource instances that the adapter manages.
Every instance in the list must be unique, but instances may share the same DSC resource type.

For more information about defining a valid adapted resource instance, see the
[Adapted resource instances](#adapted-resource-instances) section of this document.

```yaml
Type:             array
Required:         true
MinimumItemCount: 1
ValidItemSchema:  https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.resource.json
```

## Adapted resource instances

Adapted resources instances always adhere to the
[DSC Configuration document resource instance schema](../../../../schemas/config/resource.md).

Every adapted instance must be an object that defines the [name](#adapted-instance-name),
[type](#adapted-instance-type), and [properties](#adapted-instance-properties) for the instance.

### Adapted instance name

The `name` property of the adapted resource instance defines the short, human-readable name for the
instance. The adapted instance name must be a non-empty string containing only letters, numbers,
and spaces. This property should be unique within the adapter's `resources` array.

> ![NOTE]
> The adapter doesn't currently raise an error when you define two adapted instances with the same
> name. In a future release, the adapter will be updated to emit a warning when adapted instances
> share the same name. In the next major version of the adapter, name conflicts will raise an
> error.
>
> Using the same name for multiple instances can make debugging and reviewing output more
> difficult. Always use unique names for every instance.

```yaml
PropertyName:  name
Type:          string
Required:      true
MinimumLength: 1
Pattern:       ^[a-zA-Z0-9 ]+$
```

### Adapted instance type

The `type` property identifies the adapted instance's PSDSC Resource. The value for this property
must be the valid fully qualified type name for the resource.

This adapter uses the following syntax for determining the fully qualified type name of a PSDSC
resource implemented as a Windows PowerShell script-based:

```Syntax
<module name>/<script-based name>
```

For example, if a PowerShell module named **TailspinToys** has a script-based PSDSC resource named
`TSToy`, the fully qualified type name for that resource is `TailspinToys/TSToy`.

For more information about type names in DSC, see
[DSC Resource fully qualified type name schema reference][02].

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

Indicates the resource operation failed because the underlying DSC resource method or `Invoke-DscResource` call did not succeed.
When the resource returns this exit code, it also emits an error message with details about the failure.

<!-- Link definitions -->
[01]: ./examples/manage-a-windows-service.md
[02]: ../../../../schemas/config/type.md