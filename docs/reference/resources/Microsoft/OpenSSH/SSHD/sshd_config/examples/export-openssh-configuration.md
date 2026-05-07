---
description: >
  Example showing how to use Microsoft.OpenSSH.SSHD/sshd_config to export current SSH server
  configuration settings.
ms.date: 05/07/2026
ms.topic: reference
title: Export OpenSSH SSH server configuration
---

# Export OpenSSH SSH server configuration

This example demonstrates how to use the `Microsoft.OpenSSH.SSHD/sshd_config` resource with the
[dsc resource export][00] command to retrieve all current SSH server configuration settings as a
DSC configuration document that you can save and re-apply later.

> [!NOTE]
> You should run this example in an elevated context (as Administrator on Windows, or as root on
> Linux) to ensure the SSH server configuration can be read successfully.

## Export the current SSH server configuration

Run the following command to export the current `sshd_config` settings:

```powershell
dsc resource export --resource Microsoft.OpenSSH.SSHD/sshd_config
```

DSC returns a configuration document with one resource instance per exported setting. The output
looks similar to the following, where the exact properties and values reflect what is currently
configured on the system:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.OpenSSH.SSHD/sshd_config[0]
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
    port: '22'
    addressfamily: any
    listenaddress: '0.0.0.0'
    syslogfacility: AUTH
    loglevel: INFO
    logingracetime: 120
    strictmodes: 'yes'
    maxauthtries: 6
    pubkeyauthentication: 'yes'
    authorizedkeysfile: .ssh/authorized_keys
    passwordauthentication: 'no'
    permitemptypasswords: 'no'
    challengeresponseauthentication: 'no'
    kerberosauthentication: 'no'
    gssapiauthentication: 'no'
    usepam: 'yes'
    x11forwarding: 'no'
    printmotd: 'no'
    acceptenv: LANG LC_*
    subsystem: sftp /usr/lib/openssh/sftp-server
```

> [!NOTE]
> The output is truncated in this example. The actual output includes all effective
> `sshd_config` directives for your system, including defaults inherited from OpenSSH.

## Save the export to a configuration file

You can pipe the export output to a file to create a backup of your current SSH server
configuration:

```powershell
dsc resource export --resource Microsoft.OpenSSH.SSHD/sshd_config > sshd_backup.dsc.config.yaml
```

To re-apply the saved configuration to a system, use the [dsc config set][01] command:

```powershell
dsc config set --document sshd_backup.dsc.config.yaml
```

<!-- Link reference definitions -->
[00]: ../../../../../../cli/resource/export.md
[01]: ../../../../../../cli/config/set.md
