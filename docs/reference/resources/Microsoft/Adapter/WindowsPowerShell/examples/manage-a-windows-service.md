---
description: >
    Example showing how to manage a Windows service using the PSDesiredStateConfiguration module
    with the Microsoft.Adapter/WindowsPowerShell adapter in a DSC configuration document.

ms.date: 03/23/2026
ms.topic: reference
title: Manage a Windows service
---

# Manage a Windows service

This example shows how to use the `Microsoft.Adapter/WindowsPowerShell` adapter with the
`PSDesiredStateConfiguration/Service` adapted PSDSC resource to manage a Windows service. These
examples manage the `Spooler` print spooler service.

> [!NOTE]
> Run this example in an elevated PowerShell session with `dsc.exe` version `3.2.0` or later.

## Definition

The following configuration document defines a single `PSDesiredStateConfiguration/Service`
instance. It expects the `Spooler` service to be set to startup automatically.

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
directives:
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

Copy the configuration document and save it as `spooler.dsc.yaml`.

## Setup

This example modifies the `Spooler` service. The example assumes that the service is currently
running and set to require manual startup.

To put your system into the starting state for this example, run the following PowerShell command:

```powershell
Get-Service Spooler | ForEach-Object -Process {
  if ($_.StartType -ne 'Manual') {
    $_ | Set-Service -StartupType Manual
  }
  if ($_.Status -ne 'Running') {
    $_ | Start-Service
  }
}

Get-Service Spooler | Format-Table -Property Name, Status, StartType
```

You should get output from the final `Get-Service` command that shows the service is in the
expected state for the example:

```console
Name     Status StartType
----     ------ ---------
Spooler Running    Manual
```

## Test whether a service is running

To see whether the system is in the desired state, use the [`dsc config test`][01] command on the
configuration document.

```powershell
dsc config test --file spooler.dsc.yaml
```

When the service has a different startup type, DSC returns the following result:

```yaml
executionInformation:
  duration: PT424.3956473S
  endDatetime: 2026-05-07T10:30:10.621622500-05:00
  executionType: actual
  operation: test
  securityContext: elevated
  startDatetime: 2026-05-07T10:23:06.225975200-05:00
  version: 3.3.0-preview.1
metadata:
  Microsoft.DSC:
    duration: PT424.395627S
    endDatetime: 2026-05-07T10:30:10.621602200-05:00
    executionType: actual
    operation: test
    securityContext: elevated
    startDatetime: 2026-05-07T10:23:06.225975200-05:00
    version: 3.3.0-preview.1
results:
- executionInformation:
    duration: PT95.7531029S
  metadata:
    Microsoft.DSC:
      duration: PT95.7531029S
  name: Spooler service
  type: PSDesiredStateConfiguration/Service
  result:
    desiredState:
      Name: Spooler
      StartupType: Automatic
    actualState:
      Status: null
      Description: This service spools print jobs and handles interaction with the printer.  If you turn off this service, you won't be able to print or see your printers.
      DisplayName: Print Spooler
      ResourceId: null
      PsDscRunAsCredential: null
      Name: Spooler
      Credential: null
      PSComputerName: localhost
      ConfigurationName: null
      Ensure: null
      DependsOn: null
      SourceInfo: null
      BuiltInAccount: LocalSystem
      StartupType: Manual
      State: Running
      ModuleVersion: '1.1'
      ModuleName: PSDesiredStateConfiguration
      Path: C:\WINDOWS\System32\spoolsv.exe
      Dependencies:
      - RPCSS
      - http
      _inDesiredState: false
    inDesiredState: false
    differingProperties:
    - StartupType
messages: []
hadErrors: false
```

The `inDesiredState` field is `false` and `differingProperties` shows that `StartupType` differs.

## Ensure a service is running with automatic startup

Use the [`dsc config set`][02] command to configure the service:

```powershell
dsc config set --file spooler.dsc.yaml
```

When the resource configures the service, DSC returns the following result:

```yaml
executionInformation:
  duration: PT282.1686621S
  endDatetime: 2026-05-07T13:38:50.583007700-05:00
  executionType: actual
  operation: set
  securityContext: elevated
  startDatetime: 2026-05-07T13:34:08.414345600-05:00
  version: 3.3.0-preview.1
metadata:
  Microsoft.DSC:
    duration: PT282.1686429S
    endDatetime: 2026-05-07T13:38:50.582988500-05:00
    executionType: actual
    operation: set
    securityContext: elevated
    startDatetime: 2026-05-07T13:34:08.414345600-05:00
    version: 3.3.0-preview.1
results:
- executionInformation:
    duration: PT180.7721614S
  metadata:
    Microsoft.DSC:
      duration: PT180.7721614S
  name: Spooler service
  type: PSDesiredStateConfiguration/Service
  result:
    beforeState:
      Status: null
      Description: This service spools print jobs and handles interaction with the printer.  If you turn off this service, you won't be able to print or see your printers.
      DisplayName: Print Spooler
      ResourceId: null
      PsDscRunAsCredential: null
      Name: Spooler
      Credential: null
      PSComputerName: localhost
      ConfigurationName: null
      Ensure: null
      DependsOn: null
      SourceInfo: null
      BuiltInAccount: LocalSystem
      StartupType: Manual
      State: Running
      ModuleVersion: '1.1'
      ModuleName: PSDesiredStateConfiguration
      Path: C:\WINDOWS\System32\spoolsv.exe
      Dependencies:
      - RPCSS
      - http
    afterState:
      Status: null
      Description: This service spools print jobs and handles interaction with the printer.  If you turn off this service, you won't be able to print or see your printers.
      DisplayName: Print Spooler
      ResourceId: null
      PsDscRunAsCredential: null
      Name: Spooler
      Credential: null
      PSComputerName: localhost
      ConfigurationName: null
      Ensure: null
      DependsOn: null
      SourceInfo: null
      BuiltInAccount: LocalSystem
      StartupType: Automatic
      State: Running
      ModuleVersion: '1.1'
      ModuleName: PSDesiredStateConfiguration
      Path: C:\WINDOWS\System32\spoolsv.exe
      Dependencies:
      - RPCSS
      - http
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
executionInformation:
  duration: PT188.0880439S
  endDatetime: 2026-05-07T13:49:04.563267-05:00
  executionType: actual
  operation: test
  securityContext: elevated
  startDatetime: 2026-05-07T13:45:56.475223100-05:00
  version: 3.3.0-preview.1
metadata:
  Microsoft.DSC:
    duration: PT188.0880252S
    endDatetime: 2026-05-07T13:49:04.563248300-05:00
    executionType: actual
    operation: test
    securityContext: elevated
    startDatetime: 2026-05-07T13:45:56.475223100-05:00
    version: 3.3.0-preview.1
results:
- executionInformation:
    duration: PT94.8140892S
  metadata:
    Microsoft.DSC:
      duration: PT94.8140892S
  name: Spooler service
  type: PSDesiredStateConfiguration/Service
  result:
    desiredState:
      Name: Spooler
      StartupType: Automatic
    actualState:
      Status: null
      Description: This service spools print jobs and handles interaction with the printer.  If you turn off this service, you won't be able to print or see your printers.
      DisplayName: Print Spooler
      ResourceId: null
      PsDscRunAsCredential: null
      Name: Spooler
      Credential: null
      PSComputerName: localhost
      ConfigurationName: null
      Ensure: null
      DependsOn: null
      SourceInfo: null
      BuiltInAccount: LocalSystem
      StartupType: Automatic
      State: Running
      ModuleVersion: '1.1'
      ModuleName: PSDesiredStateConfiguration
      Path: C:\WINDOWS\System32\spoolsv.exe
      Dependencies:
      - RPCSS
      - http
      _inDesiredState: true
    inDesiredState: true
    differingProperties: []
messages: []
hadErrors: false
```

## Cleanup

To stop the service and set startup type to manual, use the [`dsc resource set`][03] command:

```powershell
dsc resource set PSDesiredStateConfiguration/Service --input @'
Name:        Spooler
State:       Stopped
StartupType: Manual
'@
```

<!-- Link reference definitions -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md
[03]: ../../../../../cli/resource/set.md
