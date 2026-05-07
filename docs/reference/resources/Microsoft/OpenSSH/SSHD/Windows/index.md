---
description: Microsoft.OpenSSH.SSHD/Windows resource reference documentation
ms.date:     07/02/2025
ms.topic:    reference
title:       Microsoft.OpenSSH.SSHD/Windows
---

# Microsoft.OpenSSH.SSHD/Windows

## Synopsis

Manage SSH server global configuration settings on Windows.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [OpenSSH, Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.OpenSSH.SSHD/Windows
    properties:
      # Instance properties
      shell:
      cmdOption:
      escapeArguments:
```

## Condition

The resource only applies on systems where the `sshd` executable is available in PATH. DSC
evaluates this with the expression `[not(equals(tryWhich('sshd'), null()))]` and skips the
resource if `sshd` is not found.

## Description

The `Microsoft.OpenSSH.SSHD/Windows` resource enables you to idempotently manage the Windows
OpenSSH server global settings. These settings are stored in the Windows registry under
`HKLM\SOFTWARE\OpenSSH` and control the default shell behavior for SSH sessions:

- Set the default shell executable for SSH connections.
- Specify command-line options to pass to the default shell.
- Control whether shell arguments are escaped.

> [!NOTE]
> This resource is installed with DSC itself on systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource requires OpenSSH server and client to be installed on the Windows system.
- The resource must run in a process context that has permissions to manage the SSH server
  configuration settings.
- The resource must run at least under a Windows Server 2019 or Windows 10 (build 1809)
  operating system.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][00].

## Examples

1. [Configure default shell PowerShell][01] - Shows how to set the default shell to PowerShell.exe

## Properties

The following list describes the properties for the resource.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [shell](#shell) - The path to the default shell for SSH.
  - [cmdOption](#cmdOption) - Specifies command-line options for the shell.
  - [escapeArguments](#escapeArguments) - Specifies whether shell arguments should be escaped.

### shell

<details><summary>Expand for <code>shell</code> property metadata</summary>

```yaml
Type             : string, null
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the path to the default shell executable to use for SSH sessions.
When specified, the value must be a valid path to an executable on the system.

### cmdOption

<details><summary>Expand for <code>cmdOption</code> property metadata</summary>

```yaml
Type             : string, null
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Specifies optional command-line options to pass to the shell when it's launched.

### escapeArguments

<details><summary>Expand for <code>escapeArguments</code> property metadata</summary>

```yaml
Type             : boolean, null
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Determines whether shell arguments should be escaped. When set to `true`, the arguments will be
properly escaped before being passed to the shell.

## Instance validating schema

The resource generates its schema dynamically at runtime by running
`sshdconfig schema -s windows-global`. The following snippet shows the effective schema that
validates an instance of the resource.

```json
{
  "type": "object",
  "properties": {
    "shell": {
      "type": [
        "string",
        "null"
      ]
    },
    "cmdOption": {
      "type": [
        "string",
        "null"
      ]
    },
    "escapeArguments": {
      "type": [
        "boolean",
        "null"
      ]
    }
  }
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

- [Microsoft.DSC/PowerShell resource][02]
- For more information about OpenSSH, see [OpenSSH Documentation][03]

<!-- Link definitions -->
[00]: ../../../../../concepts/resources/capabilities.md
[01]: ./examples/configure-default-shell-powershell.md
[02]: ../../../DSC/PowerShell/index.md
[03]: /windowsserverdocs/WindowsServerDocs/administration/OpenSSH/openssh-overview
