---
description: Microsoft.OpenSSH.SSHD/Windows resource reference documentation
ms.date:     07/02/2025
ms.topic:    reference
title:       Microsoft.OpenSSH.SSHD/Windows
---

# Microsoft.OpenSSH.SSHD/Windows

## Synopsis

Manage SSH client and server configuration.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.OpenSSH.SSHD/Windows
    properties:
      # Required properties
      # Instance properties
      _exist:
      # Add other properties as needed
```

## Description

The `Microsoft.OpenSSH.SSHD/Windows` resource enables you to idempotently manage SSH server
configuration. The resource can:

- Add, update, and remove SSH client and server configuration settings.

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
- `export` - You can use the resource to export the SSHD configuration of existing instances.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][00].

## Examples

<!-- Example definitions would need to be created as separate files -->

1. [Configure default shell PowerShell][03] - Shows how to set the default shell to PowerShell.exe

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of these
  properties is invalid. For more information, see the "Required resource properties" section in
  [DSC resource properties][01]

  - [shell](#shell) - The path to the default shell for SSH.

- **Key properties:** <a id="key-properties"> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][02].

  - [shell](#shell) (required) - The path to the default shell for SSH.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [cmd_option](#cmd_option) - Specifies command-line options for the shell.
  - [escape_arguments](#escape_arguments) - Specifies whether shell arguments should be escaped.
  - [shell_arguments](#shell_arguments) - Specifies the arguments to pass to the shell.

### shell

<details><summary>Expand for <code>shell</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : true
IsKey            : true
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the path to the default shell executable to use for SSH sessions.
This property is required and must specify a valid path to an executable on the system.

### cmd_option

<details><summary>Expand for <code>cmd_option</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Specifies optional command-line options to pass to the shell when it's launched.

### escape_arguments

<details><summary>Expand for <code>escape_arguments</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Determines whether shell arguments should be escaped. When set to `true`, the arguments provided
in `shell_arguments` will be properly escaped before being passed to the shell.

### shell_arguments

<details><summary>Expand for <code>shell_arguments</code> property metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : false
ItemsMinimumCount : 0
IsRequired        : false
IsKey             : false
IsReadOnly        : false
IsWriteOnly       : false
```

</details>

Specifies an array of arguments to pass to the shell when it's launched.
Each element in the array represents a separate argument.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
validating schema only includes schema keywords that affect how the instance is validated. All
non validating keywords are omitted.

```json
{
  "type": "object",
  "required": ["shell"],
  "additionalProperties": false,
  "properties": {
    "shell": {
      "type": "string"
    },
    "cmd_option": {
      "type": "string"
    },
    "escape_arguments": {
      "type": "boolean"
    },
    "shell_arguments": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid parameter
- [2](#exit-code-2) - Invalid input
- [3](#exit-code-3) - SSH configuration error
- [4](#exit-code-4) - Json serialization failed

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed due to an invalid parameter. When the resource returns this
exit code, it also emits an error message with details about the invalid parameter.

### Exit code 2

Indicates the resource operation failed because the input instance was invalid. When the resource
returns this exit code, it also emits one or more error messages with details describing how the
input instance was invalid.

### Exit code 3

Indicates the resource operation failed due to an error in the SSH server configuration. When the
resource returns this exit code, it also emits the error message related to the SSH configuration issue.

### Exit code 4

Indicates the resource operation failed because the result couldn't be serialized to JSON.

## See also

- [Microsoft.DSC/PowerShell resource][03]
- For more information about OpenSSH, see [OpenSSH Documentation][04]

<!-- Link definitions -->
[00]: ../../../../../concepts/resources/capabilities.md
[01]: ../../../../../concepts/resources/properties.md#required-resource-properties
[02]: ../../../../../concepts/resources/properties.md#key-resource-properties
[03]: ../../../DSC/PowerShell/index.md
[04]: /windowsserverdocs/WindowsServerDocs/administration/OpenSSH/openssh-overview
[05]: ./examples/configure-default-shell-powershell.md

