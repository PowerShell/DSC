---
description: >
    Example showing how to manage a Windows service using the PSDesiredStateConfiguration module
    with the Microsoft.Adapter/WindowsPowerShell adapter in a DSC configuration document.

ms.date: 03/23/2026
ms.topic: reference
title: Manage a Windows service
---

This example shows how to use the `Microsoft.Adapter/WindowsPowerShell` adapter with the
`PSDesiredStateConfiguration/Service` resource to manage a Windows service. These examples manage
the `Spooler` print spooler service.

> [!NOTE]
> Run this example in an elevated PowerShell session with `dsc.exe` version 3.2.0 or later.

## Test whether a service is running

The following configuration document defines a single `PSDesiredStateConfiguration/Service`
instance that uses `directives.requireAdapter` to route the resource through the
`Microsoft.Adapter/WindowsPowerShell` adapter.

Save the following YAML as `spooler.dsc.yaml`:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: Spooler service
  type: PSDesiredStateConfiguration/Service
  directives:
    requireAdapter: Microsoft.Adapter/WindowsPowerShell
  properties:
    Name: Spooler
    StartupType: Automatic
```

Run the configuration test to check whether the service has the desired startup type:

```powershell
dsc config test --file spooler.dsc.yaml
```

When the service has a different startup type, DSC returns the following result:

```yaml
metadata:
  Microsoft.DSC:
    version: 3.2.0
    operation: test
    executionType: actual
    startDatetime: '2026-03-23T00:00:00.000000000+00:00'
    endDatetime: '2026-03-23T00:00:01.000000000+00:00'
    duration: PT1S
    securityContext: elevated
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.5S
  name: Spooler service
  type: PSDesiredStateConfiguration/Service
  result:
    desiredState:
      Name: Spooler
      StartupType: Automatic
    actualState:
      InDesiredState: false
    inDesiredState: false
    differingProperties:
    - StartupType
messages: []
hadErrors: false
```

The `inDesiredState` field is `false` and `differingProperties` shows that `StartupType` differs.

## Ensure a service is running with automatic startup

Use the `dsc config set` command to configure the service:

```powershell
dsc config set --file spooler.dsc.yaml
```

When the resource configures the service, DSC returns the following result:

```yaml
metadata:
  Microsoft.DSC:
    version: 3.2.0
    operation: set
    executionType: actual
    startDatetime: '2026-03-23T00:00:00.000000000+00:00'
    endDatetime: '2026-03-23T00:00:02.000000000+00:00'
    duration: PT2S
    securityContext: elevated
results:
- metadata:
    Microsoft.DSC:
      duration: PT1S
  name: Spooler service
  type: PSDesiredStateConfiguration/Service
  result:
    beforeState:
      Name: Spooler
      StartupType: Manual
      State: Running
    afterState:
      Name: Spooler
      StartupType: Automatic
      State: Running
    changedProperties:
    - StartupType
messages: []
hadErrors: false
```

Run the test again to confirm the service is now configured correctly:

```powershell
dsc config test --file spooler.dsc.yaml
```

```yaml
results:
- name: Spooler service
  type: PSDesiredStateConfiguration/Service
  result:
    desiredState:
      Name: Spooler
      StartupType: Automatic
    actualState:
      InDesiredState: true
    inDesiredState: true
    differingProperties: []
```

## Stop a service

To stop the service and set startup type to manual, update the configuration document:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: Spooler service stopped
  type: PSDesiredStateConfiguration/Service
  directives:
    requireAdapter: Microsoft.Adapter/WindowsPowerShell
  properties:
    Name: Spooler
    State: Stopped
    StartupType: Manual
```

Apply the configuration with `dsc config set` to stop the service.
