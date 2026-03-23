---
description: >
    Example showing how to configure a machine using multiple class-based PowerShell DSC resources
    with the Microsoft.Adapter/PowerShell adapter in a DSC configuration document.

ms.date: 03/23/2026
ms.topic: reference
title: Configure a machine with the PowerShell adapter
---

This example shows how to use the `Microsoft.Adapter/PowerShell` adapter to configure a machine
using multiple class-based PowerShell DSC resources in a single configuration document. These
examples use the `Microsoft.WinGet.DSC/WinGetPackage` resource from the **Microsoft.WinGet.DSC**
module to ensure several packages are installed.

> [!NOTE]
> Run this example with `dsc.exe` version 3.2.0 or later and the **Microsoft.WinGet.DSC**
> PowerShell module installed. Install the module with:
> `Install-PSResource Microsoft.WinGet.DSC`

## Configuration document

The following configuration document defines multiple `Microsoft.WinGet.DSC/WinGetPackage`
instances. Each instance sets `directives.requireAdapter` to `Microsoft.Adapter/PowerShell`.

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
- name: PowerShell 7
  type: Microsoft.WinGet.DSC/WinGetPackage
  directives:
    requireAdapter: Microsoft.Adapter/PowerShell
  properties:
    Id: Microsoft.PowerShell
    Ensure: "[parameters('ensureTools')]"
- name: Windows Terminal
  type: Microsoft.WinGet.DSC/WinGetPackage
  directives:
    requireAdapter: Microsoft.Adapter/PowerShell
  properties:
    Id: Microsoft.WindowsTerminal
    Ensure: "[parameters('ensureTools')]"
- name: Visual Studio Code
  type: Microsoft.WinGet.DSC/WinGetPackage
  directives:
    requireAdapter: Microsoft.Adapter/PowerShell
  properties:
    Id: Microsoft.VisualStudioCode
    Ensure: "[parameters('ensureTools')]"
```

## Test the configuration

Run the configuration test to check whether the packages are installed:

```powershell
dsc config test --file dev-tools.dsc.yaml
```

DSC reports the results for each instance, showing which packages need to be installed.

## Apply the configuration

Run the configuration set to install any packages that aren't already present:

```powershell
dsc config set --file dev-tools.dsc.yaml
```

## Remove the packages

To uninstall the packages, override the `ensureTools` parameter when applying the configuration:

```powershell
dsc config set --file dev-tools.dsc.yaml --parameters '{"ensureTools": "Absent"}'
```

<!-- Link references -->
[01]: ../../../../../../cli/config/test.md
[02]: ../../../../../../cli/config/set.md
