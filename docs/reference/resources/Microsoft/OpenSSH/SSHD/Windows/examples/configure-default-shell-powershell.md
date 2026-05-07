---
description: >
  Example showing how to use Microsoft.OpenSSH.SSHD/Windows to configure the default shell for SSH sessions.
ms.date: 07/15/2025
ms.topic: reference
title: Configure default shell for SSH
---

# Configure default shell for SSH

This example demonstrates how to use the `Microsoft.OpenSSH.SSHD/Windows` resource to
set the default shell for SSH connections. The examples below configure PowerShell
as the default shell for all SSH sessions.

> [!NOTE]
> You should run this example in an elevated context (as Administrator) to
> ensure the SSH server configuration can be updated successfully.

## Test the current default shell

The following snippet shows how you can use the resource with the [dsc resource test][00] command to check whether PowerShell is set as the default shell.

```powershell
$instance = @{
  shell = 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe'
} | ConvertTo-Json

dsc resource test --resource Microsoft.OpenSSH.SSHD/Windows --input $instance
```

When PowerShell is not set as the default shell, DSC returns the following result:

```yaml
desiredState:       
  shell: C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe
actualState: {}
inDesiredState: false
differingProperties:
- shell
```

## Set PowerShell as the default shell

To set PowerShell as the default shell for SSH, use the [dsc resource set][01] command.

```powershell
dsc resource set --resource Microsoft.OpenSSH.SSHD/Windows --input $instance
```

When the resource updates the default shell, DSC returns the following result:

```yaml
beforeState: {}
afterState:
  shell: C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe
changedProperties:
- shell
```

You can test the instance again to confirm that PowerShell is now the default shell:

```powershell
dsc resource test --resource Microsoft.OpenSSH.SSHD/Windows --input $instance
```

```yaml
desiredState:
  shell: C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe
actualState:
  shell: C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe
inDesiredState: true
differingProperties: []
```

## Cleanup

To restore your system to its original state, use the following command to delete the registry key:

```powershell
$params = @{
    Path = 'HKLM:\SOFTWARE\OpenSSH'
    Name = 'DefaultShell'
}
Remove-ItemProperty @params
```

To verify the configuration is removed, use the `dsc resource get` command:

```powershell
dsc resource get --resource Microsoft.OpenSSH.SSHD/Windows --input $instance
```

```yaml
actualState: {}
```

<!-- Link reference definitions -->
[00]: ../../../../../cli/resource/test.md
[01]: ../../../../../cli/resource/set.md
