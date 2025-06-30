---
description: >
  Example showing how you can invoke the Microsoft.DSC.Transitional/RunCommandOnSet resource with DSC to
  run a PowerShell command.
ms.date: 06/30/2025
ms.topic: reference
title: Run a PowerShell command
---

# Run a PowerShell command

This example shows how you can use the `Microsoft.DSC.Transitional/RunCommandOnSet` resource to execute a PowerShell command during the set operation.

## Define the PowerShell command to run

The following snippet shows how you can define a PowerShell command to run during the DSC set operation:

```powershell
$command = "Write-Output Hello | Out-File $env:TEMP\hello.txt"
$instance = @{
  executable = "powershell.exe"
  arguments  = @(
    "-Command",
    $command
  )
} | ConvertTo-Json

dsc resource set --resource Microsoft.DSC.Transitional/RunCommandOnSet --input $instance
```

To verify the results, run the following command:

```powershell
Get-Content -Path $env:TEMP\hello.txt
```

This should return:

```text
Hello
```

## Run the PowerShell command in a configuration document

You can also include this resource in a DSC configuration document:

```powershell
$command = "if ((Get-Command -Name winget -CommandType Application -ErrorAction Ignore)) {winget install --id Microsoft.PowerShell.Preview}"
$document = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: RunPowerShellCommand
    type: Microsoft.DSC.Transitional/RunCommandOnSet
    properties:
      executable: "powershell.exe"
      arguments:
        - "-Command"
        - $command
      exitCode: 0
"@
```

Apply the configuration document with the [dsc config set][00] command:

```powershell
dsc config set --input $document
```

To verify the result, you can run the `winget.exe` command:

```powershell
winget list --id Microsoft.PowerShell
```

<!-- Link reference definitions -->
[00]: ../../../../../cli/config/set.md
