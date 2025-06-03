---
description: >
    Examples showing how you can invoke the Microsoft.Windows/WindowsPowerShell with DSC to manage 
    a Windows service using the PSDesiredStateConfiguration module.

ms.date: 03/25/2025
ms.topic: reference
title: Manage a Windows service
---

This example shows how you can use the `Microsoft.Windows/WindowsPowerShell` resource with the `PSDesiredStateConfiguration` module to manage a Windows service.
These examples manage the `Spooler` print spooler service.

> [!NOTE]
> Run this example in an elevated PowerShell session with `dsc.exe` version 3.1.0-preview.2 or later.

## Test whether a service is running

The following snippet shows how you can use the resource with the [dsc resource test][01] command to check whether the `Spooler` service is running.

```powershell
$instance = @{
    Name        = 'Spooler'
    StartupType = 'Automatic'
} | ConvertTo-Json

dsc resource test --resource PSDesiredStateConfiguration/Service --input $instance
```

When the service isn't running or has a different startup type, DSC returns the following result:

```yaml
desiredState:
  Name: Spooler
  StartupType: Manual
actualState:
  InDesiredState: false
inDesiredState: false
differingProperties:
- StartupType
```

The `inDesiredState` field of the result object is set to `false`, indicating that the instance isn't in the desired state. The `differingProperties` field indicates that the `property` property is mismatched between the desired state and actual state.

## Ensure a service is running with automatic startup

To set the system to the desired state and configure the service, use the [dsc resource set][02] command.

```powershell
dsc resource set --resource PSDesiredStateConfiguration/Service --input $instance
```

When the resource configures the service, DSC returns the following result:

```yaml
beforeState:
  Status: null                                                                                                              /
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
```

You can test the instance again to confirm that the service is configured correctly:

```powershell
dsc resource test --resource PSDesiredStateConfiguration/Service --input $instance
```

```yaml
desiredState:
  Name: Spooler                                                                                                             /
  StartupType: Manual
actualState:
  InDesiredState: true
inDesiredState: true
differingProperties: []
```

## Stop a service

The following snippet shows how you can configure the `Spooler` service to be stopped with manual startup.

```powershell
$stopInstance = @{
    Name        = 'Spooler'
    State       = 'Stopped'
    StartupType = 'Manual'
} | ConvertTo-Json

dsc resource set --resource PSDesiredStateConfiguration/Service --input $stopInstance
```

When the resource stops the service, DSC returns the following result:

```yaml
beforeState:
  Status: null                                                                                                              /
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
  StartupType: Manual
  State: Stopped
  ModuleVersion: '1.1'
  ModuleName: PSDesiredStateConfiguration
  Path: C:\WINDOWS\System32\spoolsv.exe
  Dependencies:
  - RPCSS
  - http
changedProperties:
- State
```

## Verify the current state of a service

To check the current state of the service, use the `dsc resource get` command.

```powershell
dsc resource get --resource PSDesiredStateConfiguration/Service --input $instance
```

```yaml
actualState:
  Status: null                                                                                                              /
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
  State: Stopped
  ModuleVersion: '1.1'
  ModuleName: PSDesiredStateConfiguration
  Path: C:\WINDOWS\System32\spoolsv.exe
  Dependencies:
  - RPCSS
  - http
```

## Restore the original service configuration

If you want to restore the service to its original running state, you can reapply the first configuration.

```powershell
dsc resource set --resource Microsoft.Windows/WindowsPowerShell --input $instance
```

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/test.md
[02]: ../../../../../cli/resource/set.md