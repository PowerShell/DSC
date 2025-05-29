---
description: Microsoft.Windows/WindowsPowerShell resource adapter reference documentation
ms.date:     03/25/2025
ms.topic:    reference
title:       Microsoft.Windows/WindowsPowerShell
---

# Microsoft.Windows/WindowsPowerShell

## Synopsis

Manage PowerShell DSC resources. This adapter enables you to use class-based, script-based, or binary PowerShell DSC resources available on the system.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instanceName>
    type: Microsoft.Windows/WindowsPowerShell
    properties:
      resources:
      - name: <instanceName>
        type: <moduleName>/<resourceName>
        properties:
          # Instance properties
          Ensure: Present

# Or from v3.1.0-preview.2 onwards
resources:
- name: <instanceName>
  type: <moduleName>/<resourceName>
  properties:
    # Instance properties
    Ensure: Present
```

## Description

The `Microsoft.Windows/WindowsPowerShell` resource adapter enables you to invoke PSDSC resources. The resource can:

- Execute script-based DSC resources
- Run class-based DSC resource methods
- Execute binary DSC resources

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

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

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because the underlying DSC resource method or Invoke-DscResource call did not succeed.
When the resource returns this exit code, it also emits an error message with details about the failure.

<!-- Link definitions -->
[01]: ./examples/manage-a-windows-service.md