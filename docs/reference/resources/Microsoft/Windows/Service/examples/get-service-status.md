---
description: >
  Example showing how to use the Microsoft.Windows/Service resource with DSC to retrieve the
  current state of a Windows service.
ms.date:     05/08/2026
ms.topic:    reference
title:       Get service status
---

# Get service status

This example shows how you can use the `Microsoft.Windows/Service` resource to retrieve the
current configuration and runtime status of a Windows service.

## Get the state of a service by name

The following snippet shows how to use the resource with the [dsc resource get][01] command to
retrieve the current state of the `wuauserv` (Windows Update) service by its key name.

```powershell
$instance = @{ name = 'wuauserv' } | ConvertTo-Json -Compress

dsc resource get --resource Microsoft.Windows/Service --input $instance
```

When the service exists, DSC returns its full configuration and status:

```yaml
actualState:
  name: wuauserv
  displayName: Windows Update
  description: Enables the detection, download, and installation of updates for Windows and other programs. If this service is disabled, users of this computer will not be able to use Windows Update or its automatic updating feature, and programs will not be able to use the Windows Update Agent (WUA) API.
  _exist: true
  status: Stopped
  startType: Manual
  executablePath: C:\WINDOWS\System32\svchost.exe -k netsvcs -p
  logonAccount: LocalSystem
  errorControl: Normal
  dependencies:
    - rpcss
```

## Get the state of a service by display name

You can also identify the service by its display name when you don't know the key name.

```powershell
$instance = @{ displayName = 'Windows Update' } | ConvertTo-Json -Compress

dsc resource get --resource Microsoft.Windows/Service --input $instance
```

DSC resolves the display name to the corresponding key name and returns the same result.

## Get the state of a non-existent service

When you request a service that isn't registered with the SCM, the resource returns `_exist: false`
and leaves all other properties unset.

```powershell
$instance = @{ name = 'MyMissingService' } | ConvertTo-Json

dsc resource get --resource Microsoft.Windows/Service --input $instance
```

```yaml
actualState:
  name: MyMissingService
  _exist: false
```

## Export all services

To retrieve the state of every service registered on the system, use the [dsc resource export][02]
command without an input instance.

```powershell
dsc resource export --resource Microsoft.Windows/Service
```

DSC writes a single JSON configuration document to stdout. That document contains a `resources`
array with one entry per service, which you can pipe to a file or process further with other
tools.

<!-- Link definitions -->
[01]: ../../../../../cli/resource/get.md
[02]: ../../../../../cli/resource/export.md
