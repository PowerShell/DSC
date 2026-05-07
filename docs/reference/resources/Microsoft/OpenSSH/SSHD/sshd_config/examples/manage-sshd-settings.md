---
description: >
  Example showing how to use Microsoft.OpenSSH.SSHD/sshd_config to get and enforce specific SSH
  server configuration settings.
ms.date: 05/07/2026
ms.topic: reference
title: Manage SSH server configuration settings
---

# Manage SSH server configuration settings

This example demonstrates how to use the `Microsoft.OpenSSH.SSHD/sshd_config` resource to check
and enforce secure SSH server configuration settings, such as disabling password-based
authentication.

> [!NOTE]
> You should run this example in an elevated context (as Administrator on Windows, or with `sudo`
> on Linux) to ensure the SSH server configuration can be updated successfully.

## Get the current state of specific settings

The following snippet shows how to use the [dsc resource get][00] command to retrieve the current
values of specific `sshd_config` directives.

```powershell
$instance = @{
    passwordauthentication = 'no'
    permitrootlogin        = 'no'
} | ConvertTo-Json

dsc resource get --resource Microsoft.OpenSSH.SSHD/sshd_config --input $instance
```

When the settings differ from the desired values, DSC returns the current state from the
`sshd_config` file:

```yaml
actualState:
  passwordauthentication: 'yes'
  permitrootlogin: prohibit-password
```

## Test whether the settings are in the desired state

Use the [dsc resource test][01] command to check whether the current settings match your desired
values:

```powershell
dsc resource test --resource Microsoft.OpenSSH.SSHD/sshd_config --input $instance
```

When the settings are not in the desired state, DSC returns the following result:

```yaml
desiredState:
  passwordauthentication: 'no'
  permitrootlogin: 'no'
actualState:
  passwordauthentication: 'yes'
  permitrootlogin: prohibit-password
inDesiredState: false
differingProperties:
- passwordauthentication
- permitrootlogin
```

## Enforce the desired settings

Use the [dsc resource set][02] command to apply the desired settings:

```powershell
dsc resource set --resource Microsoft.OpenSSH.SSHD/sshd_config --input $instance
```

DSC updates the `sshd_config` file and returns the before and after states:

```yaml
beforeState:
  passwordauthentication: 'yes'
  permitrootlogin: prohibit-password
afterState:
  passwordauthentication: 'no'
  permitrootlogin: 'no'
changedProperties:
- passwordauthentication
- permitrootlogin
```

Test the instance again to confirm that the settings are now in the desired state:

```powershell
dsc resource test --resource Microsoft.OpenSSH.SSHD/sshd_config --input $instance
```

```yaml
desiredState:
  passwordauthentication: 'no'
  permitrootlogin: 'no'
actualState:
  passwordauthentication: 'no'
  permitrootlogin: 'no'
inDesiredState: true
differingProperties: []
```

> [!IMPORTANT]
> After changing `sshd_config` settings, restart the SSH server service for the changes to take
> effect. On Windows, run `Restart-Service sshd`. On Linux, run `sudo systemctl restart sshd`.

<!-- Link reference definitions -->
[00]: ../../../../../../cli/resource/get.md
[01]: ../../../../../../cli/resource/test.md
[02]: ../../../../../../cli/resource/set.md
