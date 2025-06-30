---
description: Microsoft.DSC.Transitional/RunCommandOnSet resource reference documentation
ms.date:     06/30/2025
ms.topic:    reference
title:       Microsoft.DSC.Transitional/RunCommandOnSet
---

# Microsoft.DSC.Transitional/RunCommandOnSet

## Synopsis

Execute a command during DSC **Set** operation.

> [!IMPORTANT]
> The `runcommandonset` command and `Microsoft.DSC.Transitional/RunCommandOnSet` resource
> is intended as a temporary transitional resource while migrating DSCv3 resources for
> your needs.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Transitional, Windows, Linux, MacOS]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.DSC.Transitional/RunCommandOnSet
    properties:
      # Required properties
      executable: string
      # Optional properties
      arguments: array
      exitCode: integer
```

## Description

The `Microsoft.DSC.Transitional/RunCommandOnSet` resource enables you to run a specified executable command
during the DSC **Set** operation. This is useful for commands that need to run as part of your configuration,
but haven't fully transitioned to a DSC resource.

The resource allows you to:

- Specify an executable to run
- Pass arguments to the executable
- Define a custom exit code to indicate success

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][00].

## Examples

1. [Run a simple command][01] - Shows how to create and delete registry keys with the
   `dsc resource` commands.
1. [Run a PowerShell command][02] - Shows how to create, modify, and delete registry values with the
   `dsc resource` commands.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of these
  properties is invalid.

  - [executable](#executable) - The executable to run on set.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [arguments](#arguments) - The argument(s), if any, to pass to the executable that runs on get or set.
  - [exitCode](#exitcode) - The expected exit code to indicate success, if non-zero. Default is zero for success.

### executable

<details><summary>Expand for <code>executable</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : true
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the executable program or command to run during the DSC **Set** operation.
This can be any valid executable file or command accessible from the system PATH.

### arguments

<details><summary>Expand for <code>arguments</code> property metadata</summary>

```yaml
Type              : array
ItemsType         : string
IsRequired        : false
IsKey             : false
IsReadOnly        : false
IsWriteOnly       : false
```

</details>

Defines the arguments to pass to the executable. Each element in the array represents a
separate argument that will be passed to the executable.

### exitCode

<details><summary>Expand for <code>exitCode</code> property metadata</summary>

```yaml
Type                  : integer
IsRequired            : false
IsKey                 : false
IsReadOnly            : false
IsWriteOnly           : false
DefaultValue          : 0
```

</details>

Defines the expected exit code to indicate success if not zero. By default, an exit code of 0 indicates
successful execution. If your executable returns a different exit code to indicate success, specify that value here.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "type": "object",
  "required": [
    "executable"
  ],
  "properties": {
    "arguments": {
      "title": "The argument(s), if any, to pass to the executable that runs on set",
      "type": "array"
    },
    "executable": {
      "title": "The executable to run on set",
      "type": "string"
    },
    "exitCode": {
      "title": "The expected exit code to indicate success, if non-zero. Default is zero for success.",
      "type": "integer"
    }
  },
  "additionalProperties": false
}
```

## See also

- [Microsoft.DSC.PowerShell](../../PowerShell/index.md)
- [Microsoft.Windows.WindowsPowerShell](../../../../Microsoft/Windows/WindowsPowerShell/index.md)

[00]: ../../../../concepts/dsc/resource-capabilities.md
[01]: ./examples/run-a-simple-command.md
[02]: ./examples/run-powershell-command.md
