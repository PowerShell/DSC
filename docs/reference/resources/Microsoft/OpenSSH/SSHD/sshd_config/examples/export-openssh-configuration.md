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
> You should run this example in an elevated context (as Administrator on Windows, or with `sudo`
> on Linux) to ensure the SSH server configuration can be read successfully.

## Export the current SSH server configuration

Run the following command to export the current `sshd_config` settings:

```powershell
dsc resource export --resource Microsoft.OpenSSH.SSHD/sshd_config
```

DSC returns a configuration document with the exported settings. By default, the export operation
returns only the directives that are explicitly set in the `sshd_config` file. Directives that the
SSH server inherits from its built-in defaults aren't included. The output looks similar to the
following, where the exact properties and values reflect what is currently configured on the
system:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.OpenSSH.SSHD/sshd_config[0]
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
    port: '22'
    passwordauthentication: 'no'
    permitrootlogin: 'no'
    pubkeyauthentication: 'yes'
    subsystem: sftp /usr/lib/openssh/sftp-server
```

## Include the inherited default settings

To export the full effective configuration, including the values that OpenSSH applies when a
directive isn't explicitly set, set the `_includeDefaults` property to `true`:

```powershell
$instance = @{
    _includeDefaults = $true
} | ConvertTo-Json

dsc resource export --resource Microsoft.OpenSSH.SSHD/sshd_config --input $instance
```

The output then includes every effective directive for the system, not only the explicitly
configured ones:

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
    usepam: 'yes'
    x11forwarding: 'no'
    printmotd: 'no'
    acceptenv: LANG LC_*
    subsystem: sftp /usr/lib/openssh/sftp-server
```

> [!NOTE]
> The output is truncated in this example.
>
> `_includeDefaults` has a different default value for each operation. The `export` operation
> excludes inherited defaults unless you set `_includeDefaults` to `true`, while the `get`
> operation includes them unless you set `_includeDefaults` to `false`. When the `get` operation
> includes defaults, it also returns an `_inheritedDefaults` array that lists which directives
> came from the OpenSSH defaults rather than the configuration file, such as
> `_inheritedDefaults: [port, addressfamily]`.

## Save the export to a configuration file

You can pipe the export output to a file to create a backup of your current SSH server
configuration:

```powershell
dsc resource export --resource Microsoft.OpenSSH.SSHD/sshd_config > sshd_backup.dsc.config.yaml
```

To re-apply the saved configuration to a system, use the [dsc config set][01] command:

```powershell
dsc config set --file sshd_backup.dsc.config.yaml
```

By default, the resource enforces only the directives present in the document and leaves any other
directives in the target system's `sshd_config` file untouched. This is because the `_purge`
property defaults to `false`.

To apply the saved configuration exactly, so that directives that aren't in the document are
removed from the target file, set `_purge` to `true` on the instance:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.OpenSSH.SSHD/sshd_config[0]
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
    _purge: true
    port: '22'
    passwordauthentication: 'no'
    permitrootlogin: 'no'
    pubkeyauthentication: 'yes'
    subsystem: sftp /usr/lib/openssh/sftp-server
```

> [!CAUTION]
> When `_purge` is `true`, the resource rewrites the `sshd_config` file to match the instance.
> Any directive that isn't in the document is removed from the target system, including
> directives that were added outside of DSC.

<!-- Link reference definitions -->
[00]: ../../../../../../cli/resource/export.md
[01]: ../../../../../../cli/config/set.md
