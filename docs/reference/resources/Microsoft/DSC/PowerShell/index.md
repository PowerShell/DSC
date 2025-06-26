---
description: Microsoft.DSC/PowerShell resource reference documentation
ms.date:     03/18/2025
ms.topic:    reference
title:       Microsoft.DSC/PowerShell
---

# Microsoft.DSC/PowerShell

## Synopsis

Adapter for resources implemented as PowerShell classes

## Metadata

```yaml
Version:    0.1.0
Kind:       adapter
Tags:       [linux, windows, macos, pwsh, powershell]
Executable: powershell.resource.ps1
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.DSC/PowerShell
    properties:
      # Required Properties
      resources:
      - name: <nested instance name>
        type: <module name>/<class name>
        properties: # adapted resource properties

# Or from v3.1.0-preview.2 onwards
resources:
- name: <instanceName>
  type: <moduleName>/<class name>
  properties: # adapted resource properties
```

## Description

The `Microsoft.DSC/PowerShell` adapter resource enables you to use PowerShell Desired State
Configuration (PSDSC) resources in DSC. The adapter is able to discover and invoke PSDSC resources
implemented as PowerShell classes.

The adapter manages the PSDSC resources in PowerShell, not Windows PowerShell. To use MOF-based
PSDSC resources or PSDSC resources that require Windows PowerShell, use the
[Microsoft.Windows/WindowsPowerShell](../../windows/windowspowershell/index.md) adapter.

This adapter doesn't use the **PSDesiredStateConfiguration** module. You don't need to install the
**PSDesiredStateConfiguration** module to use PSDSC resources in DSC through this adapter.

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
$adapterScript = dsc resource list Microsoft.DSC/PowerShell |
    ConvertFrom-Json |
    Select-Object -ExpandProperty directory |
    Join-Path -ChildPath ([System.IO.Path]::Combine("psDscAdapter", "powershell.resource.ps1"))

& $adapterScript -Operation ClearCache
```

## Requirements

- Using this adapter requires a supported version of PowerShell. DSC invokes the adapter as a
  PowerShell script. For more information about installing PowerShell, see
  [Install PowerShell on Windows, Linux, and macOS](/powershell/scripting/install/installing-powershell).

## Examples

- [Invoke a resource with the PowerShell adapter][02]
- [Configure a machine with the PowerShell adapter][03]

## Required properties

The following properties are required.

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
[type](#adapted-instance-name), and [properties](#adapted-instance-properties) for the instance.

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

## Exit Codes

The resource uses the following exit codes to report success and errors:

- `0` - Success
- `1` - Error

## See also

- [Microsoft.Windows/WindowsPowerShell](../../windows/windowspowershell/resource.md)

<!-- Link references -->
[01]: ../../../concepts/resources.md#test-operations
[02]: examples/validate-with-dsc-resource.md
[03]: examples/validate-in-a-configuration.md
[04]: cli/osinfo.md
