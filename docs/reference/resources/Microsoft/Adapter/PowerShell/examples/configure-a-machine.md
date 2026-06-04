---
description: >
    Example showing how to configure a machine using multiple class-based PowerShell DSC resources
    with the Microsoft.Adapter/PowerShell adapter in a DSC configuration document.

ms.date: 03/23/2026
ms.topic: reference
title: Configure a machine with the PowerShell adapter
---

# Configure a machine with the PowerShell adapter

This example shows how to use the `Microsoft.Adapter/PowerShell` adapter to configure a machine
using multiple class-based PowerShell DSC resources in a single configuration document. These
examples use the `Microsoft.WinGet.DSC/WinGetPackage` resource from the **Microsoft.WinGet.DSC**
module to ensure several packages are installed.

## Definition

The following configuration document defines multiple `Microsoft.WinGet.DSC/WinGetPackage`
instances.

Save the following YAML as `dev-tools.dsc.yaml`:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  ensureTools:
    type: string
    defaultValue: Present
    allowedValues:
      - Present
      - Absent
resources:
- name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  properties:
    Id: Microsoft.WindowsTerminal
    Ensure: "[parameters('ensureTools')]"
- name: Visual Studio Code
  type: Microsoft.WinGet.DSC/WinGetPackage
  properties:
    Id: Microsoft.VisualStudioCode
    Ensure: "[parameters('ensureTools')]"
```

## Setup

This example installs the WinGet software packages for the Windows Terminal and VS Code. The output
in this example shows the behavior when these packages aren't already installed on the system.

This example depends on the **Microsoft.WinGet.DSC** PowerShell module at version `1.12.440`. To
install the module, open a PowerShell session and invoke the following command:

```powershell
Install-PSResource -Name Microsoft.WinGet.DSC -Version 1.12.440
```

> [!WARNING]
> Uninstalling and reinstalling software may have unintentional side effects related to how that
> software behaves, especially if uninstalling the software removes all previously defined
> configuration for it.

To ensure that the packages aren't installed, invoke the following commands:

```powershell
winget uninstall --id Microsoft.WindowsTerminal
winget uninstall --id Microsoft.VisualStudioCode
```

## Test the configuration

To see whether the system is in the desired state, use the [`dsc config test`][01] command on the
configuration document.

```powershell
dsc config test --file dev-tools.dsc.yaml
```

DSC reports the results for each instance, showing which packages need to be installed. For this
example, neither package shows as installed:

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT8.0298239S
  metadata:
    Microsoft.DSC:
      duration: PT8.0298239S
  name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  result:
    desiredState:
      Id: Microsoft.WindowsTerminal
      Ensure: Present
    actualState:
      Version: null
      MatchOption: EqualsCaseInsensitive
      UseLatest: false
      InstallMode: Silent
      Id: Microsoft.WindowsTerminal
      Ensure: Absent
      Source: ''
      _inDesiredState: false
    inDesiredState: false
    differingProperties:
    - Ensure
- executionInformation:
    duration: PT7.6445836S
  metadata:
    Microsoft.DSC:
      duration: PT7.6445836S
  name: Visual Studio Code
  type: Microsoft.WinGet.DSC/WinGetPackage
  result:
    desiredState:
      Id: Microsoft.VisualStudioCode
      Ensure: Present
    actualState:
      UseLatest: false
      Version: null
      Source: ''
      Ensure: Absent
      Id: Microsoft.VisualStudioCode
      MatchOption: EqualsCaseInsensitive
      InstallMode: Silent
      _inDesiredState: false
    inDesiredState: false
    differingProperties:
    - Ensure
messages: []
hadErrors: false
```

## Apply the configuration

Use the [`dsc config set`][02] command to install any packages that aren't already present:

```powershell
dsc config set --file dev-tools.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT34.4280028S
  metadata:
    Microsoft.DSC:
      duration: PT34.4280028S
  name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  result:
    beforeState:
      InstallMode: Silent
      UseLatest: false
      Source: ''
      Id: Microsoft.WindowsTerminal
      Ensure: Absent
      Version: null
      MatchOption: EqualsCaseInsensitive
    afterState:
      Version: 1.24.10921.0
      UseLatest: true
      Source: winget
      InstallMode: Silent
      Ensure: Present
      MatchOption: EqualsCaseInsensitive
      Id: Microsoft.WindowsTerminal
    changedProperties:
    - Version
    - UseLatest
    - Source
    - Ensure
- executionInformation:
    duration: PT11.6464059S
  metadata:
    Microsoft.DSC:
      duration: PT11.6464059S
  name: Visual Studio Code
  type: Microsoft.WinGet.DSC/WinGetPackage
  result:
    beforeState:
      UseLatest: false
      Ensure: Absent
      Id: Microsoft.VisualStudioCode
      MatchOption: EqualsCaseInsensitive
      InstallMode: Silent
      Source: ''
      Version: null
    afterState:
      Id: Microsoft.VisualStudioCode
      MatchOption: EqualsCaseInsensitive
      Source: winget
      Version: 1.119.0
      InstallMode: Silent
      Ensure: Present
      UseLatest: true
    changedProperties:
    - Source
    - Version
    - Ensure
    - UseLatest
messages: []
hadErrors: false
```

DSC installed both of the missing packages and reports that the state of each instance was changed
during the `set` operation.

## Remove the packages

To uninstall the packages, override the `ensureTools` parameter when applying the configuration:

```powershell
$params = @{
  parameters = @{
    ensureTools = 'Absent'
  }
} | ConvertTo-Json -Compress

dsc config --parameters $params set --file dev-tools.dsc.yaml
```

<!-- Link references -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md
