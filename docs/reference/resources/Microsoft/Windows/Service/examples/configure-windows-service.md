---
description: >
  Example showing how to use the Microsoft.Windows/Service resource in a DSC configuration
  document to enforce the desired state of Windows services.
ms.date:     05/08/2026
ms.topic:    reference
title:       Configure a Windows service
---

# Configure a Windows service

This example shows how you can use the `Microsoft.Windows/Service` resource in a DSC configuration
document to enforce the desired configuration and runtime status of multiple Windows services.

> [!IMPORTANT]
> **Set** operations for this resource require an elevated (administrator) process context. Run
> your terminal or PowerShell session as Administrator before using `dsc config set`.

## Definition

The configuration document for this example defines two instances of the `Service` resource.

The first instance ensures that the Windows Update service (`wuauserv`) is stopped and configured
for manual start. The second instance ensures that the Windows Time service (`W32Time`) is running
and configured to start automatically.

:::code language="yaml" source="service.config.dsc.yaml":::

Copy the configuration document and save it as `service.config.dsc.yaml`.

## Setup

The output in this example assumes that the system has the `wuauserv` service stopped with a manual
startup and the `W32Time` service stopped with an automatic startup. You can set the system to
this starting state with the following commands:

```powershell
Set-Service -Name wuauserv -StartupType Manual    -Status Stopped
Set-Service -Name W32Time  -StartupType Automatic -Status Stopped
```

## Test the configuration

To see whether the system is already in the desired state, use the [dsc config test][01] command.

```powershell
dsc config test --file ./service.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.1113118S
  metadata:
    Microsoft.DSC:
      duration: PT0.1113118S
  name: Ensure Windows Update is stopped and set to manual start
  type: Microsoft.Windows/Service
  result:
    desiredState:
      name: wuauserv
      status: Stopped
      startType: Manual
    actualState:
      name: wuauserv
      displayName: Windows Update
      description: Enables the detection, download, and installation of updates for Windows and other programs. If this service is disabled, users of this computer will not be able to use Windows Update or its automatic updating feature, and programs will not be able to use the Windows Update Agent (WUA) API.
      _exist: true
      status: Stopped
      startType: Manual
      executablePath: C:\Windows\system32\svchost.exe -k netsvcs -p
      logonAccount: LocalSystem
      errorControl: Normal
      dependencies:
      - rpcss
    inDesiredState: true
    differingProperties: []
- executionInformation:
    duration: PT0.0353328S
  metadata:
    Microsoft.DSC:
      duration: PT0.0353328S
  name: Ensure Windows Time service is running
  type: Microsoft.Windows/Service
  result:
    desiredState:
      name: W32Time
      status: Running
      startType: Automatic
    actualState:
      name: W32Time
      displayName: Windows Time
      description: Maintains date and time synchronization on all clients and servers in the network. If this service is stopped, date and time synchronization will be unavailable. If this service is disabled, any services that explicitly depend on it will fail to start.
      _exist: true
      status: Stopped
      startType: Automatic
      executablePath: C:\Windows\system32\svchost.exe -k LocalService
      logonAccount: NT AUTHORITY\LocalService
      errorControl: Normal
    inDesiredState: false
    differingProperties:
    - status
messages: []
hadErrors: false
```

The `inDesiredState` field for the first instance is `true` because the Windows Update service is
already `Stopped` with `Manual` start, so no change is required. The second instance is `false`:
the Windows Time service exists and already has `startType: Automatic`, but its `status` is
`Stopped` while the desired state requires `Running`. Only `status` is listed in
`differingProperties`.

## Set the configuration

To enforce the desired state, use the [dsc config set][02] command.

```powershell
dsc config set --file ./service.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0924309S
  metadata:
    Microsoft.DSC:
      duration: PT0.0924309S
  name: Ensure Windows Update is stopped and set to manual start
  type: Microsoft.Windows/Service
  result:
    beforeState:
      name: wuauserv
      status: Stopped
      startType: Manual
    afterState:
      name: wuauserv
      displayName: Windows Update
      description: Enables the detection, download, and installation of updates for Windows and other programs. If this service is disabled, users of this computer will not be able to use Windows Update or its automatic updating feature, and programs will not be able to use the Windows Update Agent (WUA) API.
      _exist: true
      status: Stopped
      startType: Manual
      executablePath: C:\Windows\system32\svchost.exe -k netsvcs -p
      logonAccount: LocalSystem
      errorControl: Normal
      dependencies:
      - rpcss
    changedProperties: null
- executionInformation:
    duration: PT0.3682548S
  metadata:
    Microsoft.DSC:
      duration: PT0.3682548S
  name: Ensure Windows Time service is running
  type: Microsoft.Windows/Service
  result:
    beforeState:
      name: W32Time
      displayName: Windows Time
      description: Maintains date and time synchronization on all clients and servers in the network. If this service is stopped, date and time synchronization will be unavailable. If this service is disabled, any services that explicitly depend on it will fail to start.
      _exist: true
      status: Stopped
      startType: Automatic
      executablePath: C:\Windows\system32\svchost.exe -k LocalService
      logonAccount: NT AUTHORITY\LocalService
      errorControl: Normal
    afterState:
      name: W32Time
      displayName: Windows Time
      description: Maintains date and time synchronization on all clients and servers in the network. If this service is stopped, date and time synchronization will be unavailable. If this service is disabled, any services that explicitly depend on it will fail to start.
      _exist: true
      status: Running
      startType: Automatic
      executablePath: C:\Windows\system32\svchost.exe -k LocalService
      logonAccount: NT AUTHORITY\LocalService
      errorControl: Normal
    changedProperties:
    - status
messages: []
hadErrors: false
```

The Windows Update instance shows `changedProperties: null` because it was already in the desired
state and DSC made no changes to it. The Windows Time instance lists only `status` in
`changedProperties` because DSC only needed to start the service. The `startType` was already
`Automatic` and required no update.

<!-- Link definitions -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md
