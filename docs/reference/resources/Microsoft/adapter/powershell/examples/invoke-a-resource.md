---
description: >
    Example showing how to invoke a class-based PowerShell DSC resource using
    Microsoft.Adapter/PowerShell in a DSC configuration document.
ms.date: 03/23/2026
ms.topic: reference
title: Invoke a resource with the PowerShell adapter
---

This example shows how to use the `Microsoft.Adapter/PowerShell` adapter to invoke a class-based
PowerShell DSC resource. These examples use the `Microsoft.WinGet.DSC/WinGetPackage` resource from
the **Microsoft.WinGet.DSC** module to check whether Windows Terminal is installed.

> [!NOTE]
> Run this example with `dsc.exe` version 3.2.0 or later and the **Microsoft.WinGet.DSC**
> PowerShell module installed. Install the module with:
> `Install-PSResource Microsoft.WinGet.DSC`

## Test whether a WinGet package is installed

The following configuration document defines a single `Microsoft.WinGet.DSC/WinGetPackage`
instance that uses `directives.requireAdapter` to route the resource through the
`Microsoft.Adapter/PowerShell` adapter.

Save the following YAML as `winget-test.dsc.yaml`:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  directives:
    requireAdapter: Microsoft.Adapter/PowerShell
  properties:
    Id: Microsoft.WindowsTerminal
    Ensure: Present
```

Run the configuration test operation to check whether Windows Terminal is installed:

```powershell
dsc config test --file winget-test.dsc.yaml
```

When the package isn't installed, DSC returns the following result:

```yaml
metadata:
  Microsoft.DSC:
    version: 3.2.0
    operation: test
    executionType: actual
    startDatetime: '2026-03-23T00:00:00.000000000+00:00'
    endDatetime: '2026-03-23T00:00:01.000000000+00:00'
    duration: PT1S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.5S
  name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  result:
    desiredState:
      Id: Microsoft.WindowsTerminal
      Ensure: Present
    actualState:
      Id: Microsoft.WindowsTerminal
      Ensure: Absent
    inDesiredState: false
    differingProperties:
    - Ensure
messages: []
hadErrors: false
```

The `inDesiredState` field is `false` and `differingProperties` shows that `Ensure` differs between
the desired state and the actual state.

## Install a WinGet package

Use the `dsc config set` command to configure the system to the desired state:

```powershell
dsc config set --file winget-test.dsc.yaml
```

When the resource installs the package, DSC returns the following result:

```yaml
metadata:
  Microsoft.DSC:
    version: 3.2.0
    operation: set
    executionType: actual
    startDatetime: '2026-03-23T00:00:00.000000000+00:00'
    endDatetime: '2026-03-23T00:00:05.000000000+00:00'
    duration: PT5S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT4S
  name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  result:
    beforeState:
      Id: Microsoft.WindowsTerminal
      Ensure: Absent
    afterState:
      Id: Microsoft.WindowsTerminal
      Ensure: Present
    changedProperties:
    - Ensure
messages: []
hadErrors: false
```
