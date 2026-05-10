---
description: >
  Example showing how to use the Microsoft.DSC.Transitional/WindowsPowerShellScript resource
  to get, test, and set Windows service state using inline Windows PowerShell 5.1 scripts.
ms.date:     05/10/2026
ms.topic:    reference
title:       Manage Windows service state with PowerShell
---

# Manage Windows service state with PowerShell

> [!IMPORTANT]
> This example is intended to illustrate the capabilities and patterns of the
> `Microsoft.DSC.Transitional/WindowsPowerShellScript` resource, not as a recommended approach for
> managing Windows services. DSC ships the [`Microsoft.Windows/Service`][02] resource specifically
> for this purpose. Use `Microsoft.Windows/Service` instead — it provides a dedicated schema,
> better error messages, and does not require you to write inline scripts.

This example shows how to use Windows PowerShell 5.1 scripts to get the status of a Windows
service, test whether it is running, and start or stop it.

## Get service status

The following snippet uses the [dsc resource get][03] command to retrieve the current status of the
**Print Spooler** service.

```powershell
$instance = @'
getScript: |
  param($inputObj)
  $svc = Get-Service -Name $inputObj.name
  [PSCustomObject]@{
      name        = $svc.Name
      displayName = $svc.DisplayName
      status      = $svc.Status.ToString()
      startType   = $svc.StartType.ToString()
  }
input:
  name: spooler
'@

dsc resource get --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```yaml
actualState:
  output:
    - name:        spooler
      displayName: Print Spooler
      status:      Running
      startType:   Automatic
```

## Test service state

The following snippet uses the [dsc resource test][04] command to check whether the **Print
Spooler** service is running. The `testScript` must return exactly one boolean value.

```powershell
$instance = @'
testScript: |
  param($inputObj)
  $svc = Get-Service -Name $inputObj.name
  $svc.Status -eq 'Running'
input:
  name: spooler
'@

dsc resource test --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```yaml
actualState:
  _inDesiredState: true
inDesiredState: true
differingProperties: []
```

## Set service state

The following snippet uses the [dsc resource set][05] command to ensure the **Print Spooler**
service is running. The `getScript` captures `beforeState` and the `setScript` starts the service
if it is not already running, then captures `afterState`.

```powershell
$instance = @'
getScript: |
  param($inputObj)
  $svc = Get-Service -Name $inputObj.name
  [PSCustomObject]@{
      name   = $svc.Name
      status = $svc.Status.ToString()
  }
setScript: |
  param($inputObj)
  $svc = Get-Service -Name $inputObj.name
  if ($svc.Status -ne 'Running') {
      Start-Service -Name $inputObj.name
  }
  $svc.Refresh()
  [PSCustomObject]@{
      name   = $svc.Name
      status = $svc.Status.ToString()
  }
input:
  name: spooler
'@

dsc resource set --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```yaml
beforeState:
  output:
    - name:   spooler
      status: Stopped
afterState:
  output:
    - name:   spooler
      status: Running
```

## Using a configuration document

The following configuration document ensures the **Print Spooler** service is running and the
**Fax** service is stopped. Use the [dsc config set][01] command to enforce it.

```yaml
# services.config.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: ensureSpoolerRunning
    type: Microsoft.DSC.Transitional/WindowsPowerShellScript
    properties:
      getScript: |
        param($inputObj)
        [PSCustomObject]@{ name = $inputObj.name; status = (Get-Service $inputObj.name).Status.ToString() }
      testScript: |
        param($inputObj)
        (Get-Service -Name $inputObj.name).Status -eq 'Running'
      setScript: |
        param($inputObj)
        Start-Service -Name $inputObj.name
      input:
        name: spooler

  - name: ensureFaxStopped
    type: Microsoft.DSC.Transitional/WindowsPowerShellScript
    properties:
      getScript: |
        param($inputObj)
        [PSCustomObject]@{ name = $inputObj.name; status = (Get-Service $inputObj.name).Status.ToString() }
      testScript: |
        param($inputObj)
        (Get-Service -Name $inputObj.name).Status -eq 'Stopped'
      setScript: |
        param($inputObj)
        Stop-Service -Name $inputObj.name -Force
      input:
        name: fax
```

```powershell
dsc config set --file services.config.dsc.yaml
```

> [!TIP]
> If you find yourself writing patterns like the one above for multiple services, switch to
> [`Microsoft.Windows/Service`][02]. It handles the get/test/set logic for you with a declarative
> schema and requires no inline scripts.
>
> More broadly, before writing any inline script resource, check whether DSC already ships a
> purpose-built resource for what you need. Native resources provide validated schemas, better
> error messages, and safer idempotency guarantees than hand-written scripts. Browse the
> [resource reference][06] to see what is available.

<!-- Link definitions -->
[01]: ../../../../../../cli/config/set.md
[02]: ../../../Windows/Service/index.md
[03]: ../../../../../../cli/resource/get.md
[04]: ../../../../../../cli/resource/test.md
[05]: ../../../../../../cli/resource/set.md
[06]: ../../../../../../resources/overview.md
