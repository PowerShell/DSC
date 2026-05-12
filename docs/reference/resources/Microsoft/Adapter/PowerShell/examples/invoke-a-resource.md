---
description: >
    Example showing how to invoke a class-based PowerShell DSC resource using the
    Microsoft.Adapter/PowerShell adapter.
ms.date: 03/23/2026
ms.topic: reference
title: Invoke a resource with the PowerShell adapter
---

This example shows how to use the `Microsoft.Adapter/PowerShell` adapter to invoke a class-based
PowerShell DSC (PSDSC) resource. These examples use the `Microsoft.WinGet.DSC/WinGetPackage` resource from
the **Microsoft.WinGet.DSC** module to check whether Windows Terminal is installed.

## Setup

This example installs the WinGet software package for the Windows terminal. The output in this example shows the behavior when this package isn't already installed on the system.

This example depends on the **Microsoft.WinGet.DSC** PowerShell module at version `1.12.440`. To install the module, open a PowerShell session and invoke the following command:

```powershell
Install-PSResource -Name Microsoft.WinGet.DSC -Version 1.12.440
```

> [!WARNING]
> Uninstalling and reinstalling software may have unintentional side effects related to how that
> software behaves, especially if uninstalling the software removes all previously defined
> configuration for it.

To ensure that the packages aren't installed, invoke the following command:

```powershell
winget uninstall --id Microsoft.WindowsTerminal
```

## Discover available adapted PSDSC resources

To show available adapted PSDSC resources, use the [`dsc resource list`][01] command with the
[`--adapter`][02] option as `Microsoft.Adapter/PowerShell`:

```powershell
dsc resource list --adapter Microsoft.Adapter/PowerShell
```

```console
Type                                                       Kind      Version   Capabilities  RequireAdapter                Description      
--------------------------------------------------------------------------------------------------------------------------------------------
Microsoft.PowerToys.Configure/PowerToysConfigure           Resource  0.85.1    gs--t----     Microsoft.Adapter/PowerShell  The module enabl…
Microsoft.Windows.Developer/AdvancedNetworkSharingSetting  Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/DeveloperMode                  Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/EnableDarkMode                 Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/EnableLongPathSupport          Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/EnableRemoteDesktop            Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/FirewallRule                   Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/NetConnectionProfile           Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/OsVersion                      Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/PowerPlanSetting               Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/ShowSecondsInClock             Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/Taskbar                        Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/UserAccessControl              Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/WindowsCapability              Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.Windows.Developer/WindowsExplorer                Resource  0.4.0     gs--t----     Microsoft.Adapter/PowerShell  DSC Resource for 
Microsoft.WinGet.DSC/WinGetAdminSettings                   Resource  1.12.440  gs--t----     Microsoft.Adapter/PowerShell  PowerShell Modul…
Microsoft.WinGet.DSC/WinGetPackage                         Resource  1.12.440  gs--t----     Microsoft.Adapter/PowerShell  PowerShell Modul…
Microsoft.WinGet.DSC/WinGetPackageManager                  Resource  1.12.440  gs--t----     Microsoft.Adapter/PowerShell  PowerShell Modul…
Microsoft.WinGet.DSC/WinGetSource                          Resource  1.12.440  gs--t----     Microsoft.Adapter/PowerShell  PowerShell Modul…
Microsoft.WinGet.DSC/WinGetUserSettings                    Resource  1.12.440  gs--t----     Microsoft.Adapter/PowerShell  PowerShell Modul…
```

## Test whether an instance is in the desired state

You can use the [`dsc resource test`][03] command to invoke the `test` operation for a resource
without authoring a configuration document.

The following snippet invokes the `Microsoft.WinGet.DSC/WinGetPackage` PSDSC resource to check
whether the Windows Terminal package is installed:

```powershell
$resource = 'Microsoft.WinGet.DSC/WinGetPackage'
$instance = @{
  Id     = 'Microsoft.WindowsTerminal'
  Ensure = 'Present'
} | ConvertTo-Json -Compress

dsc resource test --resource $resource  --input $instance
```

When the package isn't installed, DSC returns the following result:

```yaml
desiredState:
  Id: Microsoft.WindowsTerminal
  Ensure: Present
actualState:
  Id: Microsoft.WindowsTerminal
  InstallMode: Silent
  Version: null
  Ensure: Absent
  MatchOption: EqualsCaseInsensitive
  Source: ''
  UseLatest: false
  _inDesiredState: false
inDesiredState: false
differingProperties:
- Ensure
```

The `inDesiredState` field is `false` and `differingProperties` shows that `Ensure` differs between
the desired state and the actual state.

## Set an instance to the desired state

Use the [`dsc config set`][04] command to enforce the desired state for the resource instance:

```powershell
$resource = 'Microsoft.WinGet.DSC/WinGetPackage'
$instance = @{
  Id     = 'Microsoft.WindowsTerminal'
  Ensure = 'Present'
} | ConvertTo-Json -Compress

dsc resource set --resource $resource  --input $instance
```

When the resource installs the package, DSC returns the following result:

```yaml
beforeState:
  Ensure: Absent
  UseLatest: false
  InstallMode: Silent
  Version: null
  Id: Microsoft.WindowsTerminal
  MatchOption: EqualsCaseInsensitive
  Source: ''
afterState:
  Version: 1.24.10921.0
  MatchOption: EqualsCaseInsensitive
  Ensure: Present
  Id: Microsoft.WindowsTerminal
  UseLatest: true
  InstallMode: Silent
  Source: winget
changedProperties:
- Version
- Ensure
- UseLatest
- Source
```

## Cleanup

To remove the installed package, define the desired state for the `Ensure` property as `Absent` and invoke the `dsc resource set` command again:

```powershell
$resource = 'Microsoft.WinGet.DSC/WinGetPackage'
$instance = @{
  Id     = 'Microsoft.WindowsTerminal'
  Ensure = 'Absent'
} | ConvertTo-Json -Compress

dsc resource set --resource $resource  --input $instance
```

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/list.md
[02]: ../../../../../cli/resource/list.md#--adapter
[03]: ../../../../../cli/resource/test.md
[04]: ../../../../../cli/resource/set.md
