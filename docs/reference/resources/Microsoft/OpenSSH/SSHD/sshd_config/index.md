---
description: Microsoft.OpenSSH.SSHD/sshd_config resource reference documentation
ms.date: 05/07/2026
ms.topic: reference
title: Microsoft.OpenSSH.SSHD/sshd_config
---

# Microsoft.OpenSSH.SSHD/sshd_config

## Synopsis

Manage SSH Server Configuration.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.OpenSSH.SSHD/sshd_config
    properties:
      # Any sshd_config directive as a key
      <directive>: <value>
```

## Condition

The resource only applies on systems where the `sshd` executable is available in PATH. DSC
evaluates this with the expression `[not(equals(tryWhich('sshd'), null()))]` and skips the
resource if `sshd` is not found.

## Description

The `Microsoft.OpenSSH.SSHD/sshd_config` resource enables you to idempotently manage SSH server
configuration settings stored in the `sshd_config` file. The resource can:

- Retrieve current SSH server configuration settings.
- Apply desired SSH server configuration settings.
- Export all current SSH server configuration settings as individual resource instances.

> [!NOTE]
> This resource is installed with DSC itself on systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource requires OpenSSH server to be installed on the system.
- The resource must run in a process context that has permissions to read and write the `sshd_config`
  file.
- On Windows, the default configuration file path is `%ProgramData%\ssh\sshd_config`.
- On Linux, the default configuration file path is `/etc/ssh/sshd_config`.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `export` - You can use the resource to export all current SSH server configuration settings as
  individual resource instances.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][00].

## Examples

1. [Export OpenSSH configuration][01] - Shows how to export current OpenSSH configuration.
2. [Manage SSH server configuration settings][02] - Shows how to get and set specific sshd_config
   directives.

## Properties

The `Microsoft.OpenSSH.SSHD/sshd_config` resource uses an open-object schema where each property
corresponds to an `sshd_config` directive. There are no fixed required or key properties. Any
valid `sshd_config` keyword can be used as a property name with its corresponding value.

For example:

```yaml
PermitRootLogin: 'no'
PasswordAuthentication: 'no'
Port: 22
```

For the full list of supported directives and their values, see the
[sshd_config man page][05] or the OpenSSH documentation.

## Instance validating schema

The resource uses an embedded open-object schema. Any `sshd_config` directive is a valid property.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "sshdconfig",
  "type": "object",
  "properties": {},
  "additionalProperties": true
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid parameter

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed due to an invalid parameter. When the resource returns this
exit code, it also emits an error message with details about the invalid parameter.

## See also

- [Microsoft.OpenSSH.SSHD/Windows resource][03]
- For more information about OpenSSH, see [OpenSSH Documentation][04]

<!-- Link definitions -->
[00]: ../../../../../concepts/resources/capabilities.md
[01]: examples/export-openssh-configuration.md
[02]: examples/manage-sshd-settings.md
[03]: ../Windows/index.md
[04]: /windowsserverdocs/WindowsServerDocs/administration/OpenSSH/openssh-overview
[05]: https://man.openbsd.org/sshd_config
